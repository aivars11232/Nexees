#!/usr/bin/env python3
"""Run the Nexees repository checks: the one command a task, a developer or CI runs before
calling the checkout healthy.

Stages, in this order (all of them by default, or the ones named with --stage):

  layout    placeholder markers follow CONV-MARKER-01, and every product and test file has its
            place in the architecture catalogue (docs/architecture/subsystems.lcl.txt)
  format    the .editorconfig rules for every text file outside docs/evidence/, and rustfmt
  lint      clippy with warnings denied; every repository script parses and states its purpose
  build     the Rust workspace, locked to Cargo.lock
  test      the Rust tests and the repository tooling tests in tests/tooling/
  docs      rustdoc with warnings denied, relative Markdown links, the four LCL documentation
            projects, FILE_TREE.txt and the manual sources
  security  no credential or private key in any file a commit would hold, and no evidence ignored
  deps      exact version pins, cargo-deny's license, ban, source and advisory gate, and no
            lockfile that no gate covers
  evidence  every closed task's receipt is complete, and from TASK-005 on carries a valid
            closure record

The runner fails closed: a stage whose tool or setting is missing fails rather than being
skipped, and the exit code is 1 when any stage failed. It never writes into the checkout:
Cargo builds go to a target folder outside it and temporary files to the system temporary
folder. The one exception is --write-file-tree, which regenerates FILE_TREE.txt on request,
listing the receipt files of an open task in advance.

The docs stage needs the LCL engine `lcl` and the canonical Core packages, named by the
environment variables NEXEES_LCL_CORE_01 and NEXEES_LCL_CORE_03. The deps stage needs
cargo-deny; it uses the advisory database already in CARGO_HOME unless --online is given.
docs/engineering/CONVENTIONS.md explains every rule.
"""
from __future__ import annotations

import argparse
import ast
import fnmatch
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path

STAGES = ['layout', 'format', 'lint', 'build', 'test', 'docs', 'security', 'deps', 'evidence']

# The commit that imported the source layout and added one marker line to every text file.
IMPORT_COMMIT = '0d6d3e5ee422182a38b793756e001ddd75df507f'
MARKER = re.compile(r'^\s*(//|#|<!--|--|/\*|;)?\s*"?(_comment"\s*:\s*")?start here"?,?\s*(-->|\*/)?\s*$', re.M)

# Roots whose every file belongs to one architecture slot, and the layers allowed under them (AD-01).
PRODUCT_ROOTS = ('apps/', 'core/', 'platform/', 'integrations/', 'assets/', 'config/', 'packaging/', 'scripts/',
                 'docs/manuals/')
ROOT_LAYERS = {'core/': {'shared_core'}, 'platform/': {'platform_adapter'}, 'apps/': {'client_ui', 'local_host'}}

LCL_PROJECTS = ['docs/charter/charter.lcl.txt', 'docs/architecture/architecture.lcl.txt',
                'docs/dependencies/dependencies.lcl.txt', 'docs/security/security.lcl.txt']
MANUALS = ['docs/manuals/LCL_IN_NEXEES_USER_MANUAL.md', 'docs/manuals/NEXEES_USER_MANUAL.md']
RECEIPT_FILES = ['RECEIPT_RESULT.md', 'corroboration.txt', 'evaluation.record.json', 'final_verification.txt',
                 'receipt.json', 'snapshot_files.sha256']
FIRST_PROFILE_TASK = 2  # receipts corroborated under the continuation profile have the six files
FIRST_CLOSURE_TASK = 5  # tasks from TASK-005 on carry a closure record in their receipt
BINARY_SUFFIXES = ('.png', '.jpg', '.jpeg', '.gif', '.webp', '.ico', '.zip', '.jar', '.so', '.apk', '.aab')
KEY_SUFFIXES = ('.pem', '.key', '.p12', '.pfx', '.jks', '.keystore', '.der')
GATED_LOCKFILES = {'Cargo.lock'}  # lockfile names the deps stage has a gate for
LOCKFILE = re.compile(r'(\.lock|\.lockfile|-lock\.json|-lock\.yaml|lock\.json)$')
CREDENTIALS = {
    'private key block': r'-----BEGIN [A-Z ]*PRIVATE KEY-----',
    'GitHub token': r'\b(gh[pousr]_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{30,})',
    'OpenAI key': r'\bsk-(proj-)?[A-Za-z0-9_-]{30,}',
    'Anthropic key': r'\bsk-ant-[A-Za-z0-9_-]{20,}',
    'AWS access key': r'\bAKIA[0-9A-Z]{16}\b',
    'Google API key': r'\bAIza[0-9A-Za-z_-]{35}\b',
    'Slack token': r'\bxox[baprs]-[A-Za-z0-9-]{10,}',
    'credentials in a URL': r'[a-z][a-z0-9+.-]*://[^\s/:@]+:[^\s/@]+@',
    'assigned password or token': (r'(?i)\b(password|passwd|secret|api_key|apikey|access_token|auth_token)\b'
                                   r'\s*[:=]\s*["\'][^"\'\s]{8,}["\']'),
}


class Unavailable(Exception):
    """A tool or setting the stage needs is missing; the stage fails instead of being skipped."""


class Runner:
    """Runs stages against one checkout and collects their findings."""

    def __init__(self, root: Path, target_dir: Path, online: bool) -> None:
        self.root = root
        self.target_dir = target_dir
        self.online = online
        self.findings: list[str] = []

    # ------------------------------------------------------------------ helpers

    def run(self, cmd: list[str], env: dict | None = None) -> int:
        """Run a command in the checkout, show it with its output and exit code, and record a
        non-zero exit as a finding."""
        print(f'$ {" ".join(cmd)}', flush=True)
        try:
            result = subprocess.run(cmd, cwd=self.root, env={**os.environ, **(env or {})},
                                    capture_output=True, text=True)
        except FileNotFoundError as error:
            raise Unavailable(f'{cmd[0]} is not installed') from error
        output = (result.stdout + result.stderr).rstrip()
        if output:
            print(output)
        print(f'exit code: {result.returncode}', flush=True)
        if result.returncode != 0:
            name = 'python3' if cmd[0] == sys.executable else cmd[0]
            self.find(f'{" ".join([name, *cmd[1:4]])} exited with {result.returncode}')
        return result.returncode

    def cargo(self, *args: str, env: dict | None = None) -> int:
        """A Cargo command against the workspace, building outside the checkout."""
        if not shutil.which('cargo'):
            raise Unavailable('cargo is not installed (rustup reads rust-toolchain.toml)')
        return self.run(['cargo', *args], env={'CARGO_TARGET_DIR': str(self.target_dir), **(env or {})})

    def git(self, *args: str) -> str:
        return subprocess.run(['git', '-C', str(self.root), *args], capture_output=True, text=True, check=True).stdout

    def files(self) -> list[str]:
        """The files a commit of this checkout would hold: tracked and untracked but not ignored."""
        listed = self.git('ls-files', '-z', '--cached', '--others', '--exclude-standard').split('\0')
        deleted = set(self.git('ls-files', '-z', '--deleted').split('\0'))
        return sorted({p for p in listed if p and p not in deleted})

    def text(self, relative: str) -> str | None:
        """A file's text, or None for a binary or non-UTF-8 file."""
        if relative.endswith(BINARY_SUFFIXES):
            return None
        data = (self.root / relative).read_bytes()
        if b'\0' in data:
            return None
        try:
            return data.decode('utf-8')
        except UnicodeDecodeError:
            return None

    def find(self, message: str) -> None:
        self.findings.append(message)
        print(f'FINDING: {message}', flush=True)

    # ------------------------------------------------------------------ layout

    def stage_layout(self) -> None:
        """Placeholder markers (CONV-MARKER-01) and the architecture's file catalogue."""
        files = self.files()
        try:
            imported = set(self.git('ls-tree', '-r', '--name-only', IMPORT_COMMIT).split('\n'))
        except subprocess.CalledProcessError as error:
            raise Unavailable(f'the history back to the layout import {IMPORT_COMMIT[:7]} is missing '
                              '(a shallow clone needs a full fetch)') from error

        def original(path: str) -> str | None:
            if path not in imported:
                return None
            return subprocess.run(['git', '-C', str(self.root), 'show', f'{IMPORT_COMMIT}:{path}'],
                                  capture_output=True, text=True, check=True).stdout

        for path in files:
            text = self.text(path)
            if text is None:
                continue
            if MARKER.search(text):
                # A marked file must still be an untouched placeholder: as the import created it,
                # or, when a later task added it, nothing but the marker line.
                untouched = original(path) == text if path in imported else MARKER.fullmatch(text.strip()) is not None
                if not untouched:
                    self.find(f'{path} was edited but kept its "start here" marker (CONV-MARKER-01)')
            elif path.endswith('.source'):
                self.find(f'{path} is a placeholder without its marker; implement it under its real extension instead')
            elif (old := original(path)) is not None and MARKER.search(old):
                if MARKER.sub('', old, count=1).lstrip('\n') == text.lstrip('\n'):
                    self.find(f'{path} lost its marker without any other edit (CONV-MARKER-01)')

        subsystems = architecture_records(self.root / 'docs/architecture/subsystems.lcl.txt')
        product = [p for p in files if p.startswith(PRODUCT_ROOTS)]
        owners: dict[str, list[str]] = {p: [] for p in product}
        for sid, record in subsystems.items():
            for slot in record['slots']:
                path = slot.split(' | ', 1)[0]
                hits = [f for f in product if (f.startswith(path) if path.endswith('/') else stem(f) == path)]
                if not hits:
                    self.find(f'{sid} lists the slot {path}, which names no file')
                for f in hits:
                    owners[f].append(sid)
                    allowed = next((layers for root, layers in ROOT_LAYERS.items() if f.startswith(root)), None)
                    if allowed and record['layer'] not in allowed:
                        self.find(f'{f} belongs to {sid}, whose layer {record["layer"]} is not allowed '
                                  f'under {f.split("/")[0]}/')
        for f, found in owners.items():
            if len(found) != 1:
                self.find(f'{f} belongs to {len(found)} subsystems instead of one: {found}')
        tests = sorted(stem(p) for p in files if p.startswith('tests/'))
        attributed = sorted(t for r in subsystems.values() for t in r['test_slots'])
        for missing in sorted(set(tests) - set(attributed)):
            self.find(f'{missing} is not attributed to a subsystem')
        for phantom in sorted(set(attributed) - set(tests)):
            self.find(f'{phantom} is attributed but does not exist')
        for duplicate in sorted({t for t in attributed if attributed.count(t) > 1}):
            self.find(f'{duplicate} is attributed to more than one subsystem')
        slots = sum(len(r['slots']) for r in subsystems.values())
        print(f'{len(files)} files; {len(product)} product files in {slots} slots; {len(tests)} test files')

    # ------------------------------------------------------------------ format

    def stage_format(self) -> None:
        """EditorConfig rules for every non-evidence text file, then rustfmt."""
        sections = editorconfig(self.root / '.editorconfig')
        checked = 0
        for path in self.files():
            if path.startswith('docs/evidence/'):
                continue  # frozen task records keep the bytes they were accepted with
            text = self.text(path)
            if text is None:
                if not path.endswith(BINARY_SUFFIXES):
                    self.find(f'{path} is not UTF-8 text')
                continue
            checked += 1
            for problem in format_problems(path, text, properties(sections, path)):
                self.find(f'{path}: {problem}')
        print(f'{checked} text files checked against .editorconfig')
        self.cargo('fmt', '--all', '--check')

    # ------------------------------------------------------------------ lint

    def stage_lint(self) -> None:
        """Clippy with warnings denied, and every repository script parses and states its purpose."""
        self.cargo('clippy', '--workspace', '--all-targets', '--locked', '--offline', '--', '-D', 'warnings')
        scripts = [p for p in self.files() if p.endswith('.py') and p.startswith(('scripts/', 'tests/'))]
        for path in scripts:
            try:
                tree = ast.parse((self.root / path).read_text(encoding='utf-8'), filename=path)
            except SyntaxError as error:
                self.find(f'{path} does not parse: {error}')
                continue
            if not ast.get_docstring(tree):
                self.find(f'{path} has no module docstring stating its purpose')
        print(f'{len(scripts)} repository scripts parsed')

    # ------------------------------------------------------------------ build and test

    def stage_build(self) -> None:
        self.cargo('build', '--workspace', '--all-targets', '--locked', '--offline')

    def stage_test(self) -> None:
        self.cargo('test', '--workspace', '--locked', '--offline')
        self.run([sys.executable, '-B', '-m', 'unittest', 'discover', '-s', 'tests/tooling', '-p', 'test_*.py'])

    # ------------------------------------------------------------------ docs

    def stage_docs(self) -> None:
        """Rustdoc, Markdown links, FILE_TREE.txt, the manual sources and the LCL documentation projects."""
        self.cargo('doc', '--workspace', '--no-deps', '--locked', '--offline', env={'RUSTDOCFLAGS': '-D warnings'})
        files = self.files()
        problems, links = self.broken_links(files)
        for problem in problems:
            self.find(problem)
        print(f'{links} relative Markdown links checked')

        written = (self.root / 'FILE_TREE.txt').read_text(encoding='utf-8')
        if written != render_file_tree(sorted(set(files) | self.planned_receipt_files())):
            self.find('FILE_TREE.txt does not match the files; regenerate it with --write-file-tree')

        for manual in MANUALS:
            text = self.text(manual) if manual in files else None
            if text is None:
                self.find(f'the manual source {manual} is missing')
            elif not re.search(r'^\*\*Status: ', text, re.M):
                self.find(f'{manual} does not state its status')
        copies = [p for p in files if p.lower().endswith('user_manual.md') and p not in MANUALS
                  and not p.startswith('docs/evidence/')]
        for copy in copies:
            self.find(f'{copy} looks like a second manual source; docs/manuals/ is the only one')

        # Last, because a missing engine ends the stage; the checks above have already run.
        core01, core03 = os.environ.get('NEXEES_LCL_CORE_01'), os.environ.get('NEXEES_LCL_CORE_03')
        if not (core01 and core03 and Path(core01).is_dir() and Path(core03).is_dir()):
            raise Unavailable('set NEXEES_LCL_CORE_01 and NEXEES_LCL_CORE_03 to the canonical LCL Core packages')
        if not shutil.which('lcl'):
            raise Unavailable('the LCL engine `lcl` is not installed')
        for project in LCL_PROJECTS:
            for verb in ('check', 'validate', 'run'):
                self.run(['lcl', verb, '--spec', core01, '--project-spec', core03, project])

    def broken_links(self, files: list[str]) -> tuple[list[str], int]:
        """Relative Markdown links whose target does not exist, and how many links were checked.

        Fixtures are skipped, because they may hold deliberately broken or hostile links, such as
        TASK-003's help fixture. A link to an open task's receipt is fine: the receipt is written
        last, and the task's evidence record links to it before it exists."""
        planned = self.planned_receipt_files()
        problems, links = [], 0
        for path in (p for p in files if p.endswith('.md') and '/fixtures/' not in f'/{p}'):
            for target in markdown_links(self.text(path) or ''):
                links += 1
                resolved = os.path.normpath(os.path.join(os.path.dirname(path), target))
                if not (self.root / resolved).exists() and resolved not in planned:
                    problems.append(f'{path} links to {target}, which does not exist')
        return problems, links

    def planned_receipt_files(self) -> set[str]:
        """The six receipt files of every open task, which FILE_TREE.txt lists before they exist.

        A task writes its receipt last, after the snapshot that FILE_TREE.txt is part of, so the
        tree names the receipt files in advance. A task is open while its evidence folder has no
        receipt folder."""
        evidence = self.root / 'docs/evidence'
        planned: set[str] = set()
        for folder in (evidence.iterdir() if evidence.is_dir() else []):
            if folder.is_dir() and re.fullmatch(r'TASK-\d{3}', folder.name) and not (folder / 'receipt').exists():
                planned |= {f'docs/evidence/{folder.name}/receipt/{name}' for name in RECEIPT_FILES}
        return planned

    # ------------------------------------------------------------------ security

    def stage_security(self) -> None:
        """No credential or private key in what a commit would hold; evidence is never ignored."""
        files = self.files()
        for path in files:
            if path.lower().endswith(KEY_SUFFIXES):
                self.find(f'{path} has a private-key file name')
            text = self.text(path)
            if text is None:
                continue
            for name, pattern in CREDENTIALS.items():
                for match in re.finditer(pattern, text):
                    line = text.count('\n', 0, match.start()) + 1
                    self.find(f'{path}:{line}: {name}')
        ignored = [p for p in self.git('ls-files', '-z', '--others', '--ignored', '--exclude-standard', '--',
                                       'docs/evidence').split('\0') if p]
        for path in ignored:
            self.find(f'{path} is evidence but Git ignores it, so a commit would miss it')
        print(f'{len(files)} files scanned for credentials and keys; ignored evidence files: {len(ignored)}')

    # ------------------------------------------------------------------ deps

    def stage_deps(self) -> None:
        """Exact pins (DS-05), cargo-deny (DS-07, DS-08), and a gate for every lockfile."""
        files = self.files()
        for manifest in (p for p in files if p.endswith('Cargo.toml') and not p.startswith('docs/evidence/')):
            for problem in pin_problems(tomllib.loads((self.root / manifest).read_text(encoding='utf-8'))):
                self.find(f'{manifest}: {problem}')
        for lockfile in (p for p in files if LOCKFILE.search(p) and not p.startswith('docs/evidence/')):
            if lockfile.rsplit('/', 1)[-1] not in GATED_LOCKFILES:
                self.find(f'{lockfile} has no advisory and license gate yet; add it to the deps stage (DS-08)')
        if not shutil.which('cargo-deny'):
            raise Unavailable('cargo-deny is not installed')
        self.run(['cargo', 'deny', *([] if self.online else ['--offline']), '--locked', 'check'])

    # ------------------------------------------------------------------ evidence

    def stage_evidence(self) -> None:
        """Closed tasks have a complete receipt folder, and from TASK-005 on a valid closure record."""
        evidence = self.root / 'docs/evidence'
        for folder in sorted(p for p in evidence.iterdir() if p.is_dir() and re.fullmatch(r'TASK-\d{3}', p.name)):
            receipt = folder / 'receipt'
            if not receipt.exists():
                print(f'{folder.name}: open (no receipt yet)')
                continue
            number = int(folder.name[5:])
            if number < FIRST_PROFILE_TASK:
                print(f'{folder.name}: closed under the original procedure, before the continuation profile')
                continue
            for name in RECEIPT_FILES:
                if not (receipt / name).is_file():
                    self.find(f'{folder.name}: receipt/{name} is missing')
            if number < FIRST_CLOSURE_TASK:
                print(f'{folder.name}: closed before closure records existed')
                continue
            try:
                data = json.loads((receipt / 'receipt.json').read_text(encoding='utf-8'))
            except (OSError, ValueError) as error:
                self.find(f'{folder.name}: receipt.json is unreadable: {error}')
                continue
            for problem in closure_problems(data, folder):
                self.find(f'{folder.name}: {problem}')
            print(f'{folder.name}: closure record checked')


# ---------------------------------------------------------------------- helpers without state

def stem(relative: str) -> str:
    """Path without its extension, so a slot survives the real extension a task gives its file."""
    folder, _, name = relative.rpartition('/')
    return f'{folder}/{name.split(".")[0]}' if folder else name.split('.')[0]


def architecture_records(path: Path) -> dict[str, dict]:
    """The subsystem records of the architecture catalogue, keyed by subsystem ID."""
    text = path.read_text(encoding='utf-8')
    found = {}
    record = r'^    ID: data\.arch_ss_\w+\n    TYPE: OBJECT\[REF\(type\.\w+\)\]\n    VALUE:\n((?:        .*\n)+)'
    for block in re.findall(record, text, flags=re.M):
        fields = {m.group(1): json.loads(m.group(2)) for m in re.finditer(r'^        (\w+): (.*)$', block, flags=re.M)}
        found[fields['subsystem_id']] = fields
    return found


def editorconfig(path: Path) -> list[tuple[str, dict]]:
    """Sections of .editorconfig as (glob, properties), in file order.

    Only the glob forms this repository uses are supported: '*', '*.ext' and '*.{a,b}'. Any
    other form fails, so a rule cannot be silently ignored."""
    sections: list[tuple[str, dict]] = []
    for raw in path.read_text(encoding='utf-8').splitlines():
        line = raw.strip()
        if not line or line.startswith(('#', ';')):
            continue
        if line.startswith('[') and line.endswith(']'):
            glob = line[1:-1]
            if not re.fullmatch(r'\*(\.[\w.]+|\.\{[\w.]+(,[\w.]+)*\})?', glob):
                raise Unavailable(f'.editorconfig section [{glob}] uses a glob form the checker does not support')
            sections.append((glob, {}))
        elif '=' in line and sections:
            key, value = (part.strip().lower() for part in line.split('=', 1))
            sections[-1][1][key] = value
    return sections


def properties(sections: list[tuple[str, dict]], relative: str) -> dict:
    """The EditorConfig properties for one file; later sections override earlier ones."""
    name = relative.rsplit('/', 1)[-1]
    result: dict = {}
    for glob, values in sections:
        braces = re.fullmatch(r'\*\.\{(.+)\}', glob)
        patterns = [f'*.{alternative}' for alternative in braces.group(1).split(',')] if braces else [glob]
        if any(fnmatch.fnmatchcase(name, pattern) for pattern in patterns):
            result.update(values)
    return result


def format_problems(relative: str, text: str, props: dict) -> list[str]:
    """Violations of the EditorConfig properties the checker enforces."""
    problems = []
    if props.get('end_of_line') == 'lf' and '\r' in text:
        problems.append('has CR line endings')
    if props.get('insert_final_newline') == 'true' and text and not text.endswith('\n'):
        problems.append('does not end with a newline')
    limit = props.get('max_line_length')
    for number, line in enumerate(text.split('\n'), 1):
        if props.get('trim_trailing_whitespace') == 'true' and line != line.rstrip(' \t'):
            problems.append(f'line {number} has trailing whitespace')
        if props.get('indent_style') == 'space' and '\t' in line[:len(line) - len(line.lstrip())]:
            problems.append(f'line {number} indents with a tab')
        if limit and limit.isdigit() and len(line) > int(limit):
            problems.append(f'line {number} is longer than {limit} characters')
    return problems


def markdown_links(text: str) -> list[str]:
    """Relative link targets of a Markdown text, without anchors; code, URLs with a scheme and
    protocol-relative links are skipped."""
    text = re.sub(r'```.*?```', '', text, flags=re.S)
    text = re.sub(r'`[^`\n]*`', '', text)
    targets = []
    for target in re.findall(r'\]\(([^)\s]+)(?:\s+"[^"]*")?\)', text):
        if re.match(r'[a-z][a-z0-9+.-]*:', target, re.I) or target.startswith(('#', '//')):
            continue
        targets.append(target.split('#', 1)[0])
    return [t for t in targets if t]


def render_file_tree(files: list[str]) -> str:
    """FILE_TREE.txt for these files: folders first, then files, each in byte order."""
    tree: dict = {}
    for path in files:
        if path.rsplit('/', 1)[-1] == '.directory':
            continue
        node = tree
        for part in path.split('/')[:-1]:
            node = node.setdefault(part + '/', {})
        node[path.split('/')[-1]] = None

    def render(node: dict, prefix: str) -> list[str]:
        folders = sorted((k for k in node if node[k] is not None), key=lambda s: s[:-1].encode())
        names = sorted((k for k in node if node[k] is None), key=lambda s: s.encode())
        entries = folders + names
        lines = []
        for i, name in enumerate(entries):
            last = i == len(entries) - 1
            lines.append(prefix + ('└── ' if last else '├── ') + name)
            if node[name] is not None:
                lines += render(node[name], prefix + ('    ' if last else '│   '))
        return lines

    return '\n'.join(['Nexees/'] + render(tree, '')) + '\n'


def pin_problems(manifest: dict) -> list[str]:
    """Dependencies that are not exact pins (DS-05) or come from an unrecorded source."""
    problems = []

    def check(table: dict, where: str) -> None:
        for name, spec in table.items():
            if isinstance(spec, str):
                version = spec
            elif isinstance(spec, dict):
                if spec.get('workspace') is True or ('path' in spec and 'version' not in spec):
                    continue
                if 'git' in spec or 'registry' in spec:
                    problems.append(f'{where}.{name} comes from a git or alternative registry source')
                    continue
                version = spec.get('version', '')
            else:
                problems.append(f'{where}.{name} has an unreadable specification')
                continue
            if not re.fullmatch(r'=\d+\.\d+\.\d+(-[\w.]+)?', version):
                problems.append(f'{where}.{name} = "{version}" is not an exact pin ("=x.y.z")')

    for section in ('dependencies', 'dev-dependencies', 'build-dependencies'):
        check(manifest.get(section, {}), section)
    for target, tables in manifest.get('target', {}).items():
        for section in ('dependencies', 'dev-dependencies', 'build-dependencies'):
            check(tables.get(section, {}), f'target.{target}.{section}')
    check(manifest.get('workspace', {}).get('dependencies', {}), 'workspace.dependencies')
    return problems


def closure_problems(receipt: dict, folder: Path) -> list[str]:
    """Problems with the closure record of a closed task (schema nexees-task-closure version 1).

    The record names the four things every closure must record (code_readability_and_cleanup
    Q5 RC-05): the readability review, the cleanup, the manual impact and the post-cleanup
    verification of the final revision. Unknown or missing fields are problems."""
    closure = receipt.get('closure')
    if not isinstance(closure, dict):
        return ['the receipt has no closure record']
    problems = []

    def exact(obj: object, keys: set[str], where: str) -> bool:
        if not isinstance(obj, dict) or set(obj) != keys:
            problems.append(f'{where} must have exactly: {", ".join(sorted(keys))}')
            return False
        return True

    def text(value: object, where: str) -> None:
        if not isinstance(value, str) or not value.strip():
            problems.append(f'{where} must be a non-empty string')

    fields = {'schema', 'version', 'task', 'readability', 'cleanup', 'manual_impact', 'final_verification'}
    if not exact(closure, fields, 'closure'):
        return problems
    if closure['schema'] != 'nexees-task-closure' or type(closure['version']) is not int or closure['version'] != 1:
        problems.append('closure must be schema nexees-task-closure version 1')
    if closure['task'] != folder.name:
        problems.append(f'closure.task is {closure["task"]!r}, not {folder.name}')
    if exact(closure['readability'], {'reviewer', 'summary'}, 'closure.readability'):
        text(closure['readability']['reviewer'], 'closure.readability.reviewer')
        text(closure['readability']['summary'], 'closure.readability.summary')
    cleanup = closure['cleanup']
    if exact(cleanup, {'summary', 'record', 'removed', 'preserved_untracked'}, 'closure.cleanup'):
        text(cleanup['summary'], 'closure.cleanup.summary')
        if not isinstance(cleanup['removed'], list) or not all(isinstance(p, str) for p in cleanup['removed']):
            problems.append('closure.cleanup.removed must be a list of paths')
        if type(cleanup['preserved_untracked']) is not int or cleanup['preserved_untracked'] < 0:
            problems.append('closure.cleanup.preserved_untracked must be a count')
        if cleanup['record'] is not None:
            record_file = folder / str(cleanup['record'])
            try:
                record = json.loads(record_file.read_text(encoding='utf-8'))
            except (OSError, ValueError):
                problems.append(f'closure.cleanup.record {cleanup["record"]} is not a readable cleanup record')
            else:
                applied = (record.get('schema') == 'nexees-cleanup-record' and record.get('version') == 1
                           and record.get('applied') is True)
                preserved = len(record.get('preserved_untracked', []))
                if not applied:
                    problems.append('the cleanup record is not an applied nexees-cleanup-record version 1')
                elif record.get('removed') != cleanup['removed'] or preserved != cleanup['preserved_untracked']:
                    problems.append('closure.cleanup disagrees with its cleanup record')
        elif cleanup['removed']:
            problems.append('closure.cleanup lists removals without a cleanup record')
    manual = closure['manual_impact']
    if exact(manual, {'status', 'detail'}, 'closure.manual_impact'):
        if manual['status'] not in ('updated', 'not_applicable'):
            problems.append('closure.manual_impact.status must be updated or not_applicable')
        text(manual['detail'], 'closure.manual_impact.detail')
    final = closure['final_verification']
    if exact(final, {'log', 'snapshot', 'result'}, 'closure.final_verification'):
        if final['log'] != 'receipt/final_verification.txt' or not (folder / final['log']).is_file():
            problems.append('closure.final_verification.log must be receipt/final_verification.txt and exist')
        if final['snapshot'] != receipt.get('inputs', {}).get('current_snapshot_sha256'):
            problems.append('closure.final_verification.snapshot differs from the receipt snapshot')
        if final['result'] != 'passed':
            problems.append('closure.final_verification.result must be passed')
    return problems


def default_target_dir(root: Path) -> Path:
    """CARGO_TARGET_DIR when it is outside the checkout, else a folder in the system temp folder."""
    configured = os.environ.get('CARGO_TARGET_DIR')
    if configured:
        path = Path(configured).resolve()
        if path != root and root not in path.parents:
            return path
    return Path(tempfile.gettempdir()) / 'nexees-run-checks-target'


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split('\n\n')[0])
    parser.add_argument('--stage', action='append', choices=STAGES, help='run only this stage (repeatable)')
    parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parents[2], help='the checkout to check')
    parser.add_argument('--target-dir', type=Path, help='Cargo target folder; never inside the checkout')
    parser.add_argument('--online', action='store_true', help='let cargo-deny fetch the advisory database')
    parser.add_argument('--write-file-tree', action='store_true', help='regenerate FILE_TREE.txt and stop')
    args = parser.parse_args(argv)
    root = args.root.resolve()
    target = (args.target_dir or default_target_dir(root)).resolve()
    if target == root or root in target.parents:
        print(f'refused: the Cargo target folder {target} is inside the checkout', file=sys.stderr)
        return 2
    runner = Runner(root, target, args.online)
    if args.write_file_tree:
        files = sorted(set(runner.files()) | runner.planned_receipt_files())
        (root / 'FILE_TREE.txt').write_text(render_file_tree(files), encoding='utf-8')
        print('FILE_TREE.txt written')
        return 0
    results = {}
    for stage in args.stage or STAGES:
        print(f'\n== {stage} ==', flush=True)
        runner.findings = []
        try:
            getattr(runner, f'stage_{stage}')()
            results[stage] = 'PASS' if not runner.findings else 'FAIL'
        except Unavailable as error:
            print(f'UNAVAILABLE: {error}')
            results[stage] = 'FAIL'
            runner.findings.append(str(error))
        print(f'-- {stage}: {results[stage]}' + (f' ({len(runner.findings)} findings)' if runner.findings else ''))
    print('\n' + '  '.join(f'{stage} {result}' for stage, result in results.items()))
    failed = [s for s, r in results.items() if r != 'PASS']
    print('ALL STAGES PASSED' if not failed else f'FAILED: {", ".join(failed)}')
    return 1 if failed else 0


if __name__ == '__main__':
    raise SystemExit(main(sys.argv[1:]))
