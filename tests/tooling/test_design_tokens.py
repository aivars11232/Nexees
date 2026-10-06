"""Tests for scripts/build/design_tokens.py: the shared design tokens are checked before any
client copy is written, and the committed Desktop copy is exactly what the sources give (R19).

The refusals are tested on small token files made here. Two tests check the repository
itself: the committed Desktop copy is current, and no token or icon is left unused by the window.
"""
from __future__ import annotations

import contextlib
import copy
import importlib.util
import io
import json
import re
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('design_tokens', ROOT / 'scripts' / 'build' / 'design_tokens.py')
design_tokens = importlib.util.module_from_spec(spec)
spec.loader.exec_module(design_tokens)

TOKENS = {
    'description': 'Test tokens.',
    'color': {
        'surface': {'window': '#101010', 'raised': '#202020'},
        'text': {'primary': '#f0f0f0', 'on_accent': '#ffffff'},
        'accent': {'primary': '#60a0ff', 'fill': '#2050c0'},
        'status': {'error': '#ff7070'},
        'syntax': {'keyword': '#d090e0'},
    },
    'font': {'base': 13},
    'layout_percent': {'side': 20},
}
ICONS = {'description': 'Test icons.', 'icons': {'host_attached': 'plug', 'saved': 'check-all'}}


def changed(source: dict, path: str, value: object) -> dict:
    """A copy of `source` with the value at the dotted `path` replaced, or removed for `...`."""
    result = copy.deepcopy(source)
    *parents, last = path.split('.')
    target = result
    for name in parents:
        target = target[name]
    if value is ...:
        del target[last]
    else:
        target[last] = value
    return result


class RulesTest(unittest.TestCase):
    def test_sound_sources_pass_and_keep_their_order(self) -> None:
        self.assertEqual(list(design_tokens.checked_tokens(TOKENS)), ['color', 'font', 'layout_percent'])
        # A hundred is a size like any other; only as a share is it refused (the cases below).
        self.assertEqual(design_tokens.checked_tokens(changed(TOKENS, 'font.base', 100))['font'], {'base': 100})
        self.assertEqual(design_tokens.checked_icons(ICONS), ICONS['icons'])

    def test_contrast_is_the_wcag_ratio(self) -> None:
        self.assertAlmostEqual(design_tokens.contrast('#000000', '#ffffff'), 21.0)
        self.assertAlmostEqual(design_tokens.contrast('#3d9df7', '#3d9df7'), 1.0)
        self.assertAlmostEqual(design_tokens.contrast('#777777', '#ffffff'), 4.48, places=2)

    def test_tokens_that_break_a_rule_are_refused(self) -> None:
        cases = {
            'no description': changed(TOKENS, 'description', ...),
            'no colours': changed(TOKENS, 'color', ...),
            'no surfaces': changed(TOKENS, 'color.surface', ...),
            'an empty group': changed(TOKENS, 'color.status', {}),
            'an upper-case colour': changed(TOKENS, 'color.text.primary', '#F0F0F0'),
            'a three-digit colour': changed(TOKENS, 'color.text.primary', '#fff'),
            'a named colour': changed(TOKENS, 'color.text.primary', 'white'),
            'a colour with opacity': changed(TOKENS, 'color.text.primary', '#f0f0f0ff'),
            'a colour outside a group': changed(TOKENS, 'color.stray', '#f0f0f0'),
            'a size that is not whole': changed(TOKENS, 'font.base', 13.5),
            'a size of zero': changed(TOKENS, 'font.base', 0),
            'a size too large': changed(TOKENS, 'font.base', 1025),
            'a size as text': changed(TOKENS, 'font.base', '13px'),
            'a truth value for a size': changed(TOKENS, 'font.base', True),
            'a name with a capital': changed(TOKENS, 'font.Base', 13),
            'a name with a hyphen': changed(TOKENS, 'font.line-height', 20),
            'a number outside a group': changed(TOKENS, 'stray', 4),
            'a share of the whole': changed(TOKENS, 'layout_percent.side', 100),
            'a share of nothing': changed(TOKENS, 'layout_percent.side', 0),
            'a share that is not whole': changed(TOKENS, 'layout_percent.side', 19.5),
            'text too faint on a surface': changed(TOKENS, 'color.text.primary', '#606060'),
            'an accent too faint on a surface': changed(TOKENS, 'color.accent.primary', '#303a50'),
            'a status too faint on one surface only': changed(TOKENS, 'color.surface.raised', '#a05050'),
            'text unreadable on the fill': changed(TOKENS, 'color.accent.fill', '#a0c8ff'),
            'no text for the fills': changed(TOKENS, 'color.text.on_accent', ...),
        }
        for name, tokens in cases.items():
            with self.subTest(name), self.assertRaises(design_tokens.Refused):
                design_tokens.checked_tokens(tokens)

    def test_an_icon_mapping_that_breaks_a_rule_is_refused(self) -> None:
        cases = {
            'no icons': changed(ICONS, 'icons', ...),
            'an extra field': {**ICONS, 'sizes': {}},
            'no description': changed(ICONS, 'description', ...),
            'an empty map': changed(ICONS, 'icons', {}),
            'an icon that is no Codicon name': changed(ICONS, 'icons.saved', '$(check)'),
            'an icon with a modifier': changed(ICONS, 'icons.saved', 'sync~spin'),
            'an icon that is not text': changed(ICONS, 'icons.saved', 7),
            'a concept with a capital': changed(ICONS, 'icons.hostAttached', 'plug'),
        }
        for name, mapping in cases.items():
            with self.subTest(name), self.assertRaises(design_tokens.Refused):
                design_tokens.checked_icons(mapping)


class CopyTest(unittest.TestCase):
    def setUp(self) -> None:
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.root = Path(tmp.name)
        for path, value in ((design_tokens.TOKENS, TOKENS), (design_tokens.ICONS, ICONS)):
            (self.root / path).parent.mkdir(parents=True, exist_ok=True)
            (self.root / path).write_text(json.dumps(value))
        (self.root / design_tokens.DESKTOP_COPY).parent.mkdir(parents=True)

    def run_main(self, *arguments: str) -> int:
        with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
            return design_tokens.main(list(arguments), self.root)

    def test_the_copy_holds_every_value_and_check_mode_tells_a_stale_copy_without_writing(self) -> None:
        target = self.root / design_tokens.DESKTOP_COPY
        self.assertEqual(self.run_main('--check'), 1)
        self.assertFalse(target.exists())
        self.assertEqual(self.run_main(), 0)
        text = target.read_text()
        self.assertTrue(text.startswith('// Generated by scripts/build/design_tokens.py'))
        values = ("window: '#101010',", 'base: 13,', 'side: 20,', "host_attached: 'plug',", "saved: 'check-all',")
        for expected in values:
            self.assertIn(expected, text)
        self.assertEqual(self.run_main('--check'), 0)
        (self.root / design_tokens.TOKENS).write_text(json.dumps(changed(TOKENS, 'font.base', 14)))
        self.assertEqual(self.run_main('--check'), 1)
        self.assertEqual(target.read_text(), text)

    def test_a_refused_source_or_a_name_given_twice_writes_nothing(self) -> None:
        target = self.root / design_tokens.DESKTOP_COPY
        twice = json.dumps(ICONS).replace('"saved"', '"host_attached"')
        for path, text in ((design_tokens.TOKENS, json.dumps(changed(TOKENS, 'font.base', 0))),
                           (design_tokens.TOKENS, '{"description": "x", "color": '),
                           (design_tokens.TOKENS, '[]'),
                           (design_tokens.ICONS, twice)):
            with self.subTest(text=text[:40]):
                original = (self.root / path).read_text()
                (self.root / path).write_text(text)
                self.assertEqual(self.run_main(), 2)
                self.assertFalse(target.exists())
                (self.root / path).write_text(original)


class RepositoryTest(unittest.TestCase):
    def test_the_committed_desktop_copy_is_current(self) -> None:
        self.assertEqual((ROOT / design_tokens.DESKTOP_COPY).read_text(encoding='utf-8'),
                         design_tokens.desktop_copy(ROOT))

    def test_the_window_uses_every_token_and_every_icon(self) -> None:
        """A token or icon nothing uses is dead weight (C17). The window names a colour as
        `group.token` after taking the groups out of TOKENS.color, and everything else in full,
        in any of its TypeScript sources: the shell's and those of its other folders."""
        window = (ROOT / design_tokens.DESKTOP_COPY).parents[1]
        code = '\n'.join(path.read_text(encoding='utf-8') for path in sorted(window.rglob('*.ts'))
                         if path.name != design_tokens.DESKTOP_COPY.name)
        tokens = design_tokens.checked_tokens(design_tokens.read(ROOT / design_tokens.TOKENS))
        icons = design_tokens.checked_icons(design_tokens.read(ROOT / design_tokens.ICONS))
        names = [f'{group}.{token}' for group, colours in tokens.pop('color').items() for token in colours]
        names += [f'TOKENS.{group}.{token}' for group, numbers in tokens.items() for token in numbers]
        names += [f'ICONS.{concept}' for concept in icons]
        unused = [name for name in names if not re.search(rf'(?<![\w.]){re.escape(name)}\b', code)]
        self.assertEqual(unused, [])


if __name__ == '__main__':
    unittest.main()
