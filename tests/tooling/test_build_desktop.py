"""Tests for scripts/build/build_desktop.py: setting the fuses of an Electron binary (SI-28, TH-03),
and taking the logo's icons into an application (B1).

The install step changes bytes inside the installed copy of the Electron binary, so the rule under
test is that it changes exactly the fuses it was asked for, reads the result back from the file,
and refuses any binary whose fuse wire it does not know exactly. The tests use small stand-in files
that hold a fuse wire; the real binary is exercised by tests/e2e/desktop/local_application.

An application carries the logo only as the icons the logo manifest records: the build refuses an
icon file that is anything else, so a substituted picture cannot reach an installation.

The window speaks the host's protocol in TypeScript, so it states the protocol's versions and bounds
a second time. The last test holds each of those numbers to the one the shared core defines.
"""
from __future__ import annotations

import hashlib
import importlib.util
import json
import re
import tempfile
import unittest
from pathlib import Path
from unittest import mock

BUILD = Path(__file__).resolve().parents[2] / 'scripts' / 'build' / 'build_desktop.py'
spec = importlib.util.spec_from_file_location('build_desktop', BUILD)
build_desktop = importlib.util.module_from_spec(spec)
spec.loader.exec_module(build_desktop)

DEFAULTS = b'101100011'  # Electron's defaults, in the order of build_desktop.FUSES


class FuseTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)

    def binary(self, wire: bytes = DEFAULTS, version: int = 1, markers: int = 1) -> Path:
        """A stand-in for an Electron binary: other bytes around `markers` copies of the fuse wire."""
        wires = (build_desktop.FUSE_MARKER + bytes([version, len(wire)]) + wire + b'\x00rest') * markers
        path = Path(self.tmp.name) / 'electron'
        path.write_bytes(b'\x7fELF' + b'\x00' * 64 + wires + b'\x00' * 64)
        return path

    def test_only_the_named_fuses_change_and_the_result_is_read_back(self) -> None:
        binary = self.binary()
        before = binary.read_bytes()
        wanted = {'EnableNodeOptionsEnvironmentVariable': False, 'EnableNodeCliInspectArguments': False}
        self.assertEqual(build_desktop.set_fuses(binary, wanted), ('101100011', '100000011'))
        after = binary.read_bytes()
        changed = [index for index, (old, new) in enumerate(zip(before, after)) if old != new]
        self.assertEqual((len(after), len(changed)), (len(before), 2))
        # Setting them again changes nothing, and a fuse can be switched on as well.
        self.assertEqual(build_desktop.set_fuses(binary, wanted), ('100000011', '100000011'))
        self.assertEqual(build_desktop.set_fuses(binary, {'OnlyLoadAppFromAsar': True}), ('100000011', '100001011'))

    def test_a_binary_without_exactly_one_known_fuse_wire_is_refused_unchanged(self) -> None:
        wanted = {'RunAsNode': False}
        cases = {
            'no marker': lambda: self.binary(markers=0),
            'two markers': lambda: self.binary(markers=2),
            'another wire version': lambda: self.binary(version=2),
            'another number of fuses': lambda: self.binary(wire=DEFAULTS + b'1'),
        }
        for name, make in cases.items():
            with self.subTest(name):
                binary = make()
                before = binary.read_bytes()
                with self.assertRaises(build_desktop.Refused):
                    build_desktop.set_fuses(binary, wanted)
                self.assertEqual(binary.read_bytes(), before)

    def test_an_unknown_fuse_a_state_that_is_not_a_boolean_or_a_removed_fuse_is_refused(self) -> None:
        for wanted in ({'NoSuchFuse': False}, {'RunAsNode': 0}, {'RunAsNode': 'off'}):
            with self.subTest(wanted=wanted), self.assertRaises(build_desktop.Refused):
                build_desktop.set_fuses(self.binary(), wanted)
        # Electron marks a fuse it removed with 'r'; such a fuse has no state to set, and the fuses
        # named beside it stay as they were.
        removed = self.binary(wire=b'101r00011')
        before = removed.read_bytes()
        with self.assertRaises(build_desktop.Refused):
            build_desktop.set_fuses(removed, {'RunAsNode': False, 'EnableNodeCliInspectArguments': False})
        self.assertEqual(removed.read_bytes(), before)


class LogoIconTest(unittest.TestCase):
    def setUp(self) -> None:
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.branding = Path(tmp.name)
        (self.branding / 'derived').mkdir()
        self.icons = {16: b'sixteen', 128: b'the window icon'}
        for size, data in self.icons.items():
            (self.branding / f'derived/nexees-{size}.png').write_bytes(data)
        manifest = {'derived': [{'file': f'derived/nexees-{size}.png', 'size': size,
                                 'sha256': hashlib.sha256(data).hexdigest()} for size, data in self.icons.items()]}
        (self.branding / 'manifest.json').write_text(json.dumps(manifest))
        patch = mock.patch.object(build_desktop, 'BRANDING', self.branding)
        patch.start()
        self.addCleanup(patch.stop)

    def test_the_icons_the_manifest_records_are_taken_by_size(self) -> None:
        self.assertEqual(build_desktop.logo_icons(), self.icons)

    def test_an_icon_that_is_not_the_recorded_one_is_refused(self) -> None:
        (self.branding / 'derived/nexees-128.png').write_bytes(b'another picture')
        with self.assertRaises(build_desktop.Refused):
            build_desktop.logo_icons()



class RepositoryLogoTest(unittest.TestCase):
    def test_the_icon_the_build_places_for_the_window_is_one_the_real_manifest_records(self) -> None:
        self.assertIn(build_desktop.WINDOW_ICON_SIZE, build_desktop.logo_icons())
        self.assertTrue(build_desktop.WINDOW_ICON.endswith(f'nexees-{build_desktop.WINDOW_ICON_SIZE}.png'))

    def test_the_window_names_the_file_the_build_places_and_names_it_once(self) -> None:
        """The window reads the logo two folders above its page, lib/frontend: the application's own
        folder. The About dialog names the file, and the title row shows the same one by that name."""
        shell = build_desktop.APP / 'src/shell'
        dialog = (shell / 'about_dialog.ts').read_text(encoding='utf-8')
        named = re.search(r"^export const LOGO = '\.\./\.\./(.+)';$", dialog, re.M)
        self.assertIsNotNone(named)
        self.assertEqual(named.group(1), build_desktop.WINDOW_ICON)
        naming = [path.name for path in sorted(shell.glob('*.ts')) if '.png' in path.read_text(encoding='utf-8')]
        self.assertEqual(naming, ['about_dialog.ts'])
        self.assertIn("import { LOGO } from './about_dialog';", (shell / 'title_bar.ts').read_text(encoding='utf-8'))


class WindowProtocolTest(unittest.TestCase):
    """What the window's TypeScript states of the host's protocol is what the shared core defines."""

    ROOT = build_desktop.APP.parents[1]

    def number(self, path: str, pattern: str) -> int:
        """The one number `pattern` finds in the file `path`, with or without digit separators."""
        text = (self.ROOT / path).read_text(encoding='utf-8')
        found = re.findall(pattern, text, re.M)
        self.assertEqual(len(found), 1, f'{path}: {pattern} found {len(found)} times')
        return int(found[0].replace('_', ''))

    def test_the_window_states_the_versions_and_bounds_of_the_shared_core(self) -> None:
        def rust(path: str, name: str, kind: str) -> int:
            return self.number(path, rf'^pub const {name}: {kind} = ([\d_]+);$')

        def typescript(path: str, name: str) -> int:
            return self.number(path, rf'^(?:export )?const {name} = ([\d_]+);$')

        versions = 'core/protocol/version_negotiation.rs'
        window, contract = 'apps/desktop/src/main.ts', 'apps/desktop/src/main.protocol.ts'
        core = {
            'newest version': rust(versions, 'CURRENT_VERSION', 'u32'),
            'minimum secure version': rust(versions, 'MIN_SECURE_VERSION', 'u32'),
            'first version with layouts': rust(versions, 'LAYOUT_SINCE_VERSION', 'u32'),
            'largest message': rust('core/protocol/serialization.rs', 'MAX_IPC_MESSAGE_BYTES', 'usize'),
            'largest panel size': rust('core/domain/client.rs', 'MAX_PANEL_SIZE', 'u32'),
            'longest view name': rust('core/domain/ids.rs', 'MAX_ID_BYTES', 'usize'),
        }
        # The pattern of a view's name allows one first character and then at most so many more.
        view_name = r'^const VIEW_ID = /\^\[A-Za-z0-9\]\[A-Za-z0-9_\.:-\]\{0,(\d+)\}\$/;$'
        stated = {
            'newest version': self.number(window, r'^const VERSIONS = \{ min: \d+, max: (\d+) \};$'),
            'minimum secure version': typescript(window, 'MIN_SECURE_VERSION'),
            'first version with layouts': typescript(window, 'LAYOUT_SINCE_VERSION'),
            'largest message': typescript(window, 'MAX_MESSAGE_BYTES'),
            'largest panel size': typescript(contract, 'MAX_PANEL_SIZE'),
            'longest view name': 1 + self.number(contract, view_name),
        }
        self.assertEqual(stated, core)
        # The oldest version the window speaks is the minimum secure one: it never offers an older one.
        oldest = self.number(window, r'^const VERSIONS = \{ min: (\d+), max: \d+ \};$')
        self.assertEqual(oldest, core['minimum secure version'])


if __name__ == '__main__':
    unittest.main()
