"""Tests for scripts/build/build_desktop.py: setting the fuses of an Electron binary (SI-28, TH-03).

The install step changes bytes inside the installed copy of the Electron binary, so the rule under
test is that it changes exactly the fuses it was asked for, reads the result back from the file,
and refuses any binary whose fuse wire it does not know exactly. The tests use small stand-in files
that hold a fuse wire; the real binary is exercised by tests/e2e/desktop/local_application.
"""
from __future__ import annotations

import importlib.util
import tempfile
import unittest
from pathlib import Path

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


if __name__ == '__main__':
    unittest.main()
