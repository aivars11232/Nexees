#!/usr/bin/env python3
"""Check the Nexees design tokens and write each client's copy of them.

    design_tokens.py           write the Desktop copy
    design_tokens.py --check   write nothing; report whether the committed copy is current

assets/theme/design_tokens.json and assets/theme/icon_mapping.json are the one source of the
visual system both clients share (R19). A client never reads them at run time and never keeps
values of its own: this script turns them into source code of the client's language, and the
copy is committed beside the code that uses it. The Desktop copy is
apps/desktop/src/shell/design_tokens.ts. TASK-058 adds the Android copy here.

The script fails closed. Before it writes anything it refuses a source that is not exactly what
it understands:

- design_tokens.json holds `description`, `color` and groups of numbers. A colour is `#rrggbb`
  in lower case, in a group under `color`. Every other value is a whole number from 1 to 1024.
- icon_mapping.json holds `description` and `icons`, a map from a concept to the name of a
  Codicon.
- Names are lower-case words joined by underscores, and no name appears twice.
- Text stays readable: every text, accent, status and syntax colour has a contrast of at least
  4.5 to 1 (WCAG 2 AA) against every surface it may stand on, and the text on a filled accent
  has it against the fills.

Exit codes: 0 done or current, 1 the committed copy is not current, 2 a source was refused.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
TOKENS = Path('assets/theme/design_tokens.json')
ICONS = Path('assets/theme/icon_mapping.json')
DESKTOP_COPY = Path('apps/desktop/src/shell/design_tokens.ts')

NAME = re.compile(r'[a-z][a-z0-9]*(_[a-z0-9]+)*')
COLOUR = re.compile(r'#[0-9a-f]{6}')
CODICON = re.compile(r'[a-z][a-z0-9]*(-[a-z0-9]+)*')
LARGEST_NUMBER = 1024
MINIMUM_CONTRAST = 4.5
# The colour groups that are drawn on a surface, and so must be readable on every one of them.
ON_SURFACES = ('text', 'accent', 'status', 'syntax')
# The exceptions: fills are backgrounds themselves, and this text stands only on the fills.
FILL_PREFIX = 'fill'
ON_FILLS = ('text', 'on_accent')


class Refused(Exception):
    """A source is not what this script understands; nothing was written."""


def read(path: Path) -> dict:
    """The JSON object in `path`, refusing a name that appears twice in one object."""
    def unique(pairs: list[tuple[str, object]]) -> dict:
        names = [name for name, _ in pairs]
        for name in names:
            if names.count(name) > 1:
                raise Refused(f'{path.name} names {name!r} twice')
        return dict(pairs)

    try:
        value = json.loads(path.read_text(encoding='utf-8'), object_pairs_hook=unique)
    except (OSError, ValueError) as error:
        raise Refused(f'{path.name} cannot be read as JSON: {error}') from error
    if not isinstance(value, dict):
        raise Refused(f'{path.name} does not hold an object')
    return value


def group(where: str, value: object) -> dict:
    """`value` as a group: a non-empty object whose names are well formed."""
    if not isinstance(value, dict) or not value:
        raise Refused(f'{where} is not a group of named values')
    for name in value:
        if not NAME.fullmatch(name):
            raise Refused(f'{where} has the name {name!r}; names are lower-case words joined by underscores')
    return value


def luminance(colour: str) -> float:
    """The relative luminance of an #rrggbb colour (WCAG 2)."""
    channels = [int(colour[i:i + 2], 16) / 255 for i in (1, 3, 5)]
    red, green, blue = [c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4 for c in channels]
    return 0.2126 * red + 0.7152 * green + 0.0722 * blue


def contrast(one: str, other: str) -> float:
    """The contrast ratio of two #rrggbb colours (WCAG 2), from 1 to 21."""
    lighter, darker = sorted((luminance(one), luminance(other)), reverse=True)
    return (lighter + 0.05) / (darker + 0.05)


def checked_tokens(tokens: dict) -> dict:
    """The tokens without their description, after every rule of the module documentation held."""
    if not isinstance(tokens.get('description'), str) or not tokens['description'].strip():
        raise Refused('design_tokens.json does not describe itself in `description`')
    groups = {name: value for name, value in tokens.items() if name != 'description'}
    group('design_tokens.json', groups)
    if 'color' not in groups:
        raise Refused('design_tokens.json has no `color`')
    for name, colours in group('color', groups['color']).items():
        for token, value in group(f'color.{name}', colours).items():
            if not isinstance(value, str) or not COLOUR.fullmatch(value):
                raise Refused(f'color.{name}.{token} is {value!r}; a colour is #rrggbb in lower case')
    for name, numbers in groups.items():
        if name == 'color':
            continue
        for token, value in group(name, numbers).items():
            if type(value) is not int or not 1 <= value <= LARGEST_NUMBER:
                raise Refused(f'{name}.{token} is {value!r}; a size is a whole number from 1 to {LARGEST_NUMBER}')
    colour = groups['color']
    for needed in ('surface', 'accent', *ON_SURFACES):
        if needed not in colour:
            raise Refused(f'design_tokens.json has no color.{needed}')
    fills = {name: value for name, value in colour['accent'].items() if name.startswith(FILL_PREFIX)}
    pairs = [(f'color.{ON_FILLS[0]}.{ON_FILLS[1]}', colour[ON_FILLS[0]].get(ON_FILLS[1]), 'color.accent', fills)]
    for name in ON_SURFACES:
        for token, value in colour[name].items():
            if (name, token) != ON_FILLS and not (name == 'accent' and token in fills):
                pairs.append((f'color.{name}.{token}', value, 'color.surface', colour['surface']))
    for foreground, value, behind, backgrounds in pairs:
        if value is None or not backgrounds:
            raise Refused(f'{foreground} or the colours of {behind} it stands on are missing')
        for background, other in backgrounds.items():
            if contrast(value, other) < MINIMUM_CONTRAST:
                raise Refused(f'{foreground} {value} has a contrast of only {contrast(value, other):.2f} to 1 '
                              f'against {behind}.{background} {other}; text needs {MINIMUM_CONTRAST} to 1')
    return groups


def checked_icons(mapping: dict) -> dict:
    """The icon of each concept, after every rule of the module documentation held."""
    if set(mapping) != {'description', 'icons'} or not isinstance(mapping['description'], str):
        raise Refused('icon_mapping.json must hold exactly `description` and `icons`')
    icons = group('icons', mapping['icons'])
    for concept, icon in icons.items():
        if not isinstance(icon, str) or not CODICON.fullmatch(icon):
            raise Refused(f'icons.{concept} is {icon!r}; an icon is the name of a Codicon')
    return icons


def typescript(value: object, depth: int = 0) -> str:
    """A checked value as a TypeScript literal in the window's style."""
    if isinstance(value, dict):
        inner = '    ' * (depth + 1)
        lines = [f'{inner}{name}: {typescript(item, depth + 1)},' for name, item in value.items()]
        return '{\n' + '\n'.join(lines) + '\n' + '    ' * depth + '}'
    return str(value) if isinstance(value, int) else f"'{value}'"


def desktop_copy(root: Path) -> str:
    """The Desktop copy of the checked sources under `root`."""
    tokens = checked_tokens(read(root / TOKENS))
    icons = checked_icons(read(root / ICONS))
    return (f'// Generated by scripts/build/design_tokens.py from {TOKENS} and\n'
            f'// {ICONS}, the one source both clients share (R19). Do not edit this\n'
            '// file: change the source and run the script again.\n'
            '\n'
            '/** The Nexees design tokens: colours as #rrggbb, every other value in CSS pixels. */\n'
            f'export const TOKENS = {typescript(tokens)} as const;\n'
            '\n'
            '/** The Nexees icon language: the name of the Codicon that stands for each concept. */\n'
            f'export const ICONS = {typescript(icons)} as const;\n')


def main(arguments: list[str] | None = None, root: Path = ROOT) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    parser.add_argument('--check', action='store_true',
                        help='write nothing; exit 1 when the committed copy is not current')
    args = parser.parse_args(arguments)
    try:
        wanted = desktop_copy(root)
    except Refused as refused:
        print(f'refused: {refused}', file=sys.stderr)
        return 2
    target = root / DESKTOP_COPY
    current = target.read_text(encoding='utf-8') if target.is_file() else None
    if args.check:
        if current != wanted:
            print(f'{DESKTOP_COPY} is not current; run scripts/build/design_tokens.py')
            return 1
        print(f'{DESKTOP_COPY} is current')
        return 0
    if current == wanted:
        print(f'{DESKTOP_COPY} is already current')
    else:
        target.write_text(wanted, encoding='utf-8')
        print(f'wrote {DESKTOP_COPY}')
    return 0


if __name__ == '__main__':
    sys.exit(main())
