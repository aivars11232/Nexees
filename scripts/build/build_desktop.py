#!/usr/bin/env python3
"""Check, build and install the Nexees Desktop application: the Rust host and the Theia window.

    build_desktop.py check   --build-dir DIR                  type-check the window's TypeScript
    build_desktop.py build   --build-dir DIR [--electron-headers VERSION]
    build_desktop.py install --build-dir DIR --prefix PREFIX  install what build made

Everything happens in DIR, a build folder outside the checkout (DS-09), from copies of the
checkout's sources; the checkout is only read. `check` compiles the TypeScript with every strict
option and emits nothing: the compiler is the TypeScript lint (CONVENTIONS.md section 4).

npm runs offline and installs exactly the committed lockfile, apps/desktop/package-lock.json.
`check` installs it without any install script. `build` lets npm run only the install scripts
that package.json's allowScripts approves (DS-06), rebuilds the native modules for Electron with
@electron/rebuild, bundles the window with `theia build`, and builds the host binary with Cargo.
Three environment variables name the caches and the home folder npm and the native builds use,
so the user's own home and caches are never read or written:

    NEXEES_NPM_CACHE       the npm cache (check and build)
    NEXEES_BUILD_HOME      the home folder for npm, node-gyp and @electron/rebuild (check and build)
    NEXEES_ELECTRON_CACHE  the Electron download cache (build)

The native modules are built against the Electron headers in NEXEES_BUILD_HOME/.electron-gyp,
by default for the installed Electron's version. --electron-headers names another version of the
same major, whose native module ABI Electron keeps the same.

`install` lays the built application out under PREFIX as packaging/desktop/package_definition.json
describes: the window and its host in one folder, a launcher, a desktop entry, and the
application's icon in each size derived from the logo. In the installed copy of the Electron
binary it sets the fuses the definition names, the switches Electron reads from its own file
before any setting or command line (SI-28, TH-03), and reads them back. PREFIX must be an
absolute folder outside the checkout. Nothing is installed anywhere else.

The logo reaches an application only as the icons scripts/build/brand_assets.py derived from the
bound source (B1): `check` and `build` copy the one the window shows into the application's
resources in their build folder, and `install` copies them all under PREFIX. Each refuses an icon
that is not the one assets/branding/manifest.json records.

Exit codes: 0 done, 1 a step failed, 2 a missing setting or a refused folder.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import mmap
import os
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
APP = ROOT / 'apps/desktop'
DEFINITION = ROOT / 'packaging/desktop/package_definition.json'
METADATA = APP / 'resources/application_metadata.json'
BRANDING = ROOT / 'assets/branding'
# The icon the window itself shows, in its About dialog (apps/desktop/src/shell/about_dialog), and
# where a build places it in the application.
WINDOW_ICON_SIZE = 128
WINDOW_ICON = 'resources/branding/nexees-128.png'
# The native modules the window needs for Electron, as TASK-003 rebuilt them.
NATIVE_MODULES = 'drivelist,keytar,native-keymap'
# The files of the window's sources that a build needs: the manifests and the TypeScript.
MANIFESTS = ['package.json', 'package-lock.json', 'src/package.json', 'src/tsconfig.json']
# Electron's fuse wire: this marker, the wire's version, the number of fuses, then one byte per
# fuse, '1' for on and '0' for off. The names are in the order of Electron's fuse schema.
FUSE_MARKER = b'dL7pKGdnNz796PbbjQWNKmHXBZaB9tsX'
FUSE_WIRE_VERSION = 1
FUSES = ['RunAsNode', 'EnableCookieEncryption', 'EnableNodeOptionsEnvironmentVariable',
         'EnableNodeCliInspectArguments', 'EnableEmbeddedAsarIntegrityValidation', 'OnlyLoadAppFromAsar',
         'LoadBrowserProcessSpecificV8Snapshot', 'GrantFileProtocolExtraPrivileges', 'WasmTrapHandlers']


class Refused(Exception):
    """A setting is missing or a folder is not acceptable; nothing was changed."""


def outside_checkout(path: Path, what: str) -> Path:
    path = path.resolve()
    if not path.is_absolute() or path == ROOT or ROOT in path.parents:
        raise Refused(f'{what} must be an absolute folder outside the checkout: {path}')
    return path


def setting(name: str) -> Path:
    value = os.environ.get(name)
    if not value or not Path(value).is_absolute():
        raise Refused(f'set {name} to an absolute folder (see the module documentation)')
    return Path(value)


def environment(build: bool) -> dict[str, str]:
    """The environment npm and the native builds run in: the build home, the caches, offline."""
    env = {key: value for key, value in os.environ.items() if not key.startswith(('npm_config_', 'ELECTRON_'))}
    env.update({
        'HOME': str(setting('NEXEES_BUILD_HOME')),
        'npm_config_cache': str(setting('NEXEES_NPM_CACHE')),
        'npm_config_offline': 'true',
        'npm_config_audit': 'false',
        'npm_config_fund': 'false',
        'npm_config_update_notifier': 'false',
    })
    if build:
        cache = str(setting('NEXEES_ELECTRON_CACHE'))
        env.update({'electron_config_cache': cache, 'ELECTRON_CACHE': cache})
    return env


def run(cmd: list[str], cwd: Path, env: dict[str, str]) -> None:
    print(f'$ {" ".join(cmd)}', flush=True)
    result = subprocess.run(cmd, cwd=cwd, env=env)
    if result.returncode != 0:
        raise RuntimeError(f'{cmd[0]} exited with {result.returncode}')


def sources() -> list[str]:
    """The window's source files a build copies, relative to apps/desktop."""
    typescript = sorted(str(p.relative_to(APP)) for p in (APP / 'src').rglob('*.ts'))
    return MANIFESTS + typescript


def logo_icons() -> dict[int, bytes]:
    """The icons derived from the logo, by size, each with the SHA-256 the logo manifest records."""
    manifest = json.loads((BRANDING / 'manifest.json').read_text(encoding='utf-8'))
    icons = {}
    for entry in manifest['derived']:
        data = (BRANDING / entry['file']).read_bytes()
        if hashlib.sha256(data).hexdigest() != entry['sha256']:
            raise Refused(f'{entry["file"]} is not the icon the logo manifest records '
                          '(scripts/build/brand_assets.py check)')
        icons[entry['size']] = data
    return icons


def stage(folder: Path, env: dict[str, str], scripts: bool) -> None:
    """Copies the sources and the window's icon into `folder` and installs the lockfile's packages
    there, unless the manifests are unchanged since the last install. Files a build made there
    (lib/, src-gen/) stay until they are rebuilt; sources that no longer exist are removed."""
    folder.mkdir(parents=True, exist_ok=True)
    icon = folder / WINDOW_ICON
    icon.parent.mkdir(parents=True, exist_ok=True)
    icon.write_bytes(logo_icons()[WINDOW_ICON_SIZE])
    wanted = set(sources())
    for old in (folder / 'src').rglob('*.ts') if (folder / 'src').is_dir() else []:
        if str(old.relative_to(folder)) not in wanted:
            old.unlink()
    for relative in sorted(wanted):
        target = folder / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(APP / relative, target)
    digest = hashlib.sha256()
    for relative in MANIFESTS:
        digest.update((APP / relative).read_bytes())
    stamp = folder / '.nexees-installed'
    wanted_stamp = f'{digest.hexdigest()} scripts={scripts}\n'
    if (folder / 'node_modules').is_dir() and stamp.is_file() and stamp.read_text() == wanted_stamp:
        print('packages: unchanged since the last install', flush=True)
        return
    stamp.unlink(missing_ok=True)
    run(['npm', 'ci', '--offline', *([] if scripts else ['--ignore-scripts'])], folder, env)
    stamp.write_text(wanted_stamp)


def check(build_dir: Path) -> None:
    folder = build_dir / 'check'
    env = environment(build=False)
    stage(folder, env, scripts=False)
    run(['node_modules/.bin/tsc', '-p', 'src/tsconfig.json', '--noEmit'], folder, env)


def build(build_dir: Path, headers: str | None, cargo_target: Path) -> None:
    folder = build_dir / 'app'
    env = environment(build=True)
    stage(folder, env, scripts=True)
    run(['node_modules/.bin/tsc', '-p', 'src/tsconfig.json'], folder, env)
    run(['node_modules/.bin/theia', 'build', '--mode', 'production'], folder, env)
    electron = json.loads((folder / 'node_modules/electron/package.json').read_text())['version']
    version = headers or electron
    if version.split('.')[0] != electron.split('.')[0]:
        raise Refused(f'Electron headers {version} are not of the installed Electron {electron}\'s major version')
    run(['node_modules/.bin/electron-rebuild', '--version', version, '--force', '--which-module', NATIVE_MODULES,
         '--only', NATIVE_MODULES], folder, env)
    cargo_env = {**os.environ, 'CARGO_TARGET_DIR': str(cargo_target)}
    run(['cargo', 'build', '--release', '--locked', '--offline', '-p', 'nexees-desktop-host'], ROOT, cargo_env)
    (folder / 'bin').mkdir(exist_ok=True)
    shutil.copy2(cargo_target / 'release/nexees-host', folder / 'bin/nexees-host')


def set_fuses(binary: Path, wanted: dict[str, bool]) -> tuple[str, str]:
    """Sets the named fuses in an Electron binary and returns its fuse wire before and after, the
    second read back from the file. Refuses, before it changes anything, a binary whose wire it
    does not know exactly: another version, a missing or repeated marker, an unknown name, or a
    fuse that is not a plain on or off."""
    for name, on in wanted.items():
        if name not in FUSES or not isinstance(on, bool):
            raise Refused(f'unknown fuse or state in the package definition: {name}={on!r}')
    with open(binary, 'r+b') as file, mmap.mmap(file.fileno(), 0) as data:
        at = data.find(FUSE_MARKER)
        if at < 0 or data.find(FUSE_MARKER, at + 1) >= 0:
            raise Refused(f'{binary} does not hold exactly one fuse wire')
        version, count = data[at + len(FUSE_MARKER)], data[at + len(FUSE_MARKER) + 1]
        first = at + len(FUSE_MARKER) + 2
        if version != FUSE_WIRE_VERSION or count != len(FUSES):
            raise Refused(f'{binary} has fuse wire version {version} with {count} fuses, not the known one')
        before = data[first:first + count].decode('ascii', 'replace')
        for name in wanted:
            if before[FUSES.index(name)] not in '01':
                raise Refused(f'the fuse {name} of {binary} cannot be set: its state is {before[FUSES.index(name)]!r}')
        for name, on in wanted.items():
            data[first + FUSES.index(name)] = ord('1' if on else '0')
        data.flush()
    with open(binary, 'rb') as file:
        file.seek(first)
        return before, file.read(count).decode('ascii', 'replace')


def install(build_dir: Path, prefix: Path) -> None:
    app = build_dir / 'app'
    if not (app / 'bin/nexees-host').is_file() or not (app / 'lib/backend').is_dir():
        raise Refused(f'nothing built in {app}; run build first')
    definition = json.loads(DEFINITION.read_text())
    metadata = json.loads(METADATA.read_text())
    layout, window = definition['layout'], definition['window']
    target = prefix / layout['app']
    if target.exists():
        shutil.rmtree(target)
    target.parent.mkdir(parents=True, exist_ok=True)
    # The sources and the manifests come along: the window's own extension is linked from them.
    shutil.copytree(app, target, symlinks=True, ignore=shutil.ignore_patterns('.nexees-installed'))
    # Only the installed copy changes; the build folder keeps Electron's binary as it was fetched.
    before, after = set_fuses(target / window['electron'], window['fuses'])
    changed = ', '.join(f'{name} {"on" if on else "off"}' for name, on in window['fuses'].items())
    print(f'fuses: {before} -> {after} ({changed})', flush=True)
    for size, data in logo_icons().items():
        icon = prefix / layout['icons'].format(size=size, icon=metadata['icon'])
        icon.parent.mkdir(parents=True, exist_ok=True)
        icon.write_bytes(data)
    launcher = prefix / layout['launcher']
    launcher.parent.mkdir(parents=True, exist_ok=True)
    arguments = ' '.join(window['arguments'])
    up = '/'.join(['..'] * (len(Path(layout['launcher']).parts) - 1))
    launcher.write_text(
        '#!/bin/sh\n'
        '# Starts the installed Nexees Desktop window. Written by scripts/build/build_desktop.py from\n'
        '# packaging/desktop/package_definition.json.\n'
        'here=$(dirname "$(readlink -f "$0")")\n'
        f'app=$(readlink -f "$here/{up}/{layout["app"]}")\n'
        '# A shell started from an editor may ask Electron to act as Node; the window needs Electron.\n'
        'unset ELECTRON_RUN_AS_NODE\n'
        f'NEXEES_HOST_PROGRAM="$app/bin/nexees-host"\n'
        'export NEXEES_HOST_PROGRAM\n'
        # Electron takes the application's name, and with it the folder of its own settings, from
        # the package.json of the folder it starts. Theia reads the folder to open as the first
        # argument after the application, so the window's own switches follow it.
        f'exec "$app/{window["electron"]}" "$app" {arguments} "$@"\n')
    launcher.chmod(0o755)
    entry = prefix / layout['desktop_entry']
    entry.parent.mkdir(parents=True, exist_ok=True)
    if any(c in str(launcher) for c in '"`$\\%\n'):
        raise Refused(f'the launcher path {launcher} cannot be written into a desktop entry')
    entry.write_text(
        '[Desktop Entry]\n'
        'Type=Application\n'
        f'Name={metadata["name"]}\n'
        f'Comment={metadata["summary"]}\n'
        f'Exec="{launcher}" %F\n'
        f'Icon={metadata["icon"]}\n'
        'Terminal=false\n'
        f'Categories={";".join(metadata["categories"])};\n'
        f'StartupWMClass={metadata["window_class"]}\n')
    print(f'installed under {prefix}', flush=True)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    parser.add_argument('command', choices=['check', 'build', 'install'])
    parser.add_argument('--build-dir', type=Path, required=True)
    parser.add_argument('--prefix', type=Path)
    parser.add_argument('--electron-headers', help='the Electron version whose headers the native modules use')
    parser.add_argument('--cargo-target-dir', type=Path, default=os.environ.get('CARGO_TARGET_DIR'))
    args = parser.parse_args()
    try:
        build_dir = outside_checkout(args.build_dir, 'the build folder')
        if args.command == 'check':
            check(build_dir)
        elif args.command == 'build':
            if args.cargo_target_dir is None:
                raise Refused('set CARGO_TARGET_DIR or --cargo-target-dir to a folder outside the checkout')
            build(build_dir, args.electron_headers, outside_checkout(args.cargo_target_dir, 'the Cargo target folder'))
        else:
            if args.prefix is None:
                raise Refused('install needs --prefix')
            install(build_dir, outside_checkout(args.prefix, 'the prefix'))
    except Refused as refused:
        print(f'refused: {refused}', file=sys.stderr)
        return 2
    except (RuntimeError, OSError) as error:
        print(f'failed: {error}', file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())
