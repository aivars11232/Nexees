#!/usr/bin/env python3
"""Measure the approved compact layout, and compare the measurements with the layout's design tokens.

    measure_approved_layout.py [ROOT]

TASK-011 takes the sizes and shares of the Desktop layout from the approved picture,
assets/design/desktop/approved-compact-layout.png, and keeps them as design tokens
(assets/theme/design_tokens.json, groups `desktop` and `desktop_percent`). This script repeats
the measurement, so that the numbers can be checked against the picture by anyone: it reads the
picture, finds the edges of its regions, and prints each measurement beside the token taken from it.

The picture is 1536 pixels wide and shows a window larger than its pixels: its status bar, which
is 22 pixels high in the window, is 30 high in the picture. That ratio turns every length of the
picture into a length of the window. Shares need no ratio.

An edge is found where the brightness along a line of the picture changes; the lines are chosen
where no text or icon crosses them. The picture is an illustration with soft edges: a boundary
shows as two or three neighbouring rows or columns, of which the script takes the middle, and it
is good to about a pixel of the picture. A token is a whole number. So the script reports the
difference, and fails only where a token is further from its measurement than the tolerance
printed with it: a pixel and a half of the window for a size, one point for a share.

Not measured: the box of the logo in the title row (desktop.title_logo_size). The picture shows
another mark than the Nexees logo, and the box is the size of the row's other icons.

Standard library only. Exit codes: 0 every token is within its tolerance, 1 one is not, 2 the
picture cannot be read as expected.
"""
from __future__ import annotations

import json
import struct
import sys
import zlib
from pathlib import Path

PICTURE = Path('assets/design/desktop/approved-compact-layout.png')
TOKENS = Path('assets/theme/design_tokens.json')
# The background of the title row, and how far a pixel must be from it to count as drawn on it.
ROW_BACKGROUND = (10, 17, 23)
DRAWN = 24


class Unreadable(Exception):
    """The picture is not the kind of PNG this script reads."""


def read_png(path: Path) -> tuple[int, int, list[bytearray]]:
    """The width, the height and the rows of an 8-bit RGB PNG that is not interlaced."""
    data = path.read_bytes()
    if data[:8] != b'\x89PNG\r\n\x1a\n':
        raise Unreadable(f'{path} is not a PNG')
    position, packed, header = 8, b'', None
    while position < len(data):
        length, kind = struct.unpack('>I4s', data[position:position + 8])
        body = data[position + 8:position + 8 + length]
        position += 12 + length
        if kind == b'IHDR':
            header = struct.unpack('>IIBBBBB', body)
        elif kind == b'IDAT':
            packed += body
    if header is None or header[2:] != (8, 2, 0, 0, 0):
        raise Unreadable(f'{path} is not an 8-bit RGB PNG without interlacing: {header}')
    width, height = header[:2]
    raw, stride = zlib.decompress(packed), width * 3
    rows, previous, position = [], bytearray(stride), 0
    for _ in range(height):
        kind, row = raw[position], bytearray(raw[position + 1:position + 1 + stride])
        position += 1 + stride
        for i in range(stride):
            left = row[i - 3] if i >= 3 else 0
            above = previous[i]
            corner = previous[i - 3] if i >= 3 else 0
            if kind == 1:
                row[i] = (row[i] + left) & 255
            elif kind == 2:
                row[i] = (row[i] + above) & 255
            elif kind == 3:
                row[i] = (row[i] + ((left + above) >> 1)) & 255
            elif kind == 4:
                nearest = min((abs(above - corner), 0, left), (abs(left - corner), 1, above),
                              (abs(left + above - 2 * corner), 2, corner))[2]
                row[i] = (row[i] + nearest) & 255
            elif kind != 0:
                raise Unreadable(f'{path} has a row with the unknown filter {kind}')
        rows.append(row)
        previous = row
    return width, height, rows


class Picture:
    """The approved picture, with the two kinds of look-up the measurements need."""

    def __init__(self, path: Path) -> None:
        self.width, self.height, self.rows = read_png(path)

    def pixel(self, x: int, y: int) -> tuple[int, int, int]:
        return tuple(self.rows[y][3 * x:3 * x + 3])

    def brightness(self, x: int, y: int) -> float:
        red, green, blue = self.pixel(x, y)
        return 0.2126 * red + 0.7152 * green + 0.0722 * blue

    def edges_down(self, x: int, threshold: float = 5) -> list[int]:
        """The rows at which the brightness changes along the column `x`."""
        return [y for y in range(1, self.height) if abs(self.brightness(x, y) - self.brightness(x, y - 1)) > threshold]

    def edges_across(self, y: int, threshold: float = 4) -> list[int]:
        """The columns at which the brightness changes along the row `y`."""
        return [x for x in range(1, self.width) if abs(self.brightness(x, y) - self.brightness(x - 1, y)) > threshold]


def boundary(edges: list[int], low: int, high: int, what: str) -> float:
    """The middle of the first run of neighbouring edges from `low` to `high`: where one region ends and the next begins."""
    found = [edge for edge in edges if low <= edge <= high]
    if not found:
        raise Unreadable(f'no edge found for {what} between {low} and {high}')
    run = [found[0]]
    for edge in found[1:]:
        if edge - run[-1] > 1:
            break
        run.append(edge)
    return (run[0] + run[-1]) / 2


def drawn_spans(picture: Picture, top: int, bottom: int) -> list[tuple[int, int]]:
    """The stretches of columns, between the rows `top` and `bottom`, in which something is drawn on the row's background."""
    def drawn(x: int) -> bool:
        return any(max(abs(c - b) for c, b in zip(picture.pixel(x, y), ROW_BACKGROUND)) > DRAWN for y in range(top, bottom))

    spans, start = [], None
    for x in range(picture.width):
        if drawn(x) and start is None:
            start = x
        elif not drawn(x) and start is not None:
            spans.append((start, x - 1))
            start = None
    merged: list[tuple[int, int]] = []
    for span in spans:
        if merged and span[0] - merged[-1][1] <= 7:
            merged[-1] = (merged[-1][0], span[1])
        else:
            merged.append(span)
    return merged


def main(root: Path) -> int:
    picture = Picture(root / PICTURE)
    tokens = json.loads((root / TOKENS).read_text(encoding='utf-8'))
    desktop, shares = tokens['desktop'], tokens['desktop_percent']

    # The window in the picture, down a column free of text. The title row ends with its border, a
    # line one pixel thick: the line shows as two neighbouring edges, and its far side lies half a
    # pixel after their middle.
    column = picture.edges_down(5)
    top = boundary(column, 100, 130, 'the top of the window')
    row_end = boundary(column, top + 20, top + 50, 'the border under the title row') + 0.5
    status_top = boundary(column, 850, 890, 'the top of the status bar')
    bottom = boundary(column, status_top + 10, status_top + 45, 'the bottom of the window')
    panel_top = boundary(picture.edges_down(950), 640, 700, 'the top of the bottom panel')
    scale = (bottom - status_top) / desktop['status_bar_height']

    # The regions across the window, which fills the picture's width, in a row free of text.
    across = picture.edges_across(205)
    window = picture.width
    activity = boundary(across, 50, 80, 'the right edge of the activity bar')
    sidebar = boundary(across, 250, 320, 'the right edge of the left sidebar')
    right = boundary(across, 1000, 1060, 'the left edge of the right sidebar')

    # The title row: what is drawn in it, from left to right. The last three are the window controls,
    # and before them stands the frame the picture draws around the two toggles.
    spans = drawn_spans(picture, int(top) + 4, int(row_end) - 2)
    close, maximise, minimise, toggles = spans[-1], spans[-2], spans[-3], spans[-4]
    centre = lambda span: (span[0] + span[1]) / 2  # noqa: E731
    control_pitch = (centre(close) - centre(minimise)) / 2

    print(f'The approved layout, {PICTURE}: {picture.width} x {picture.height} pixels.')
    print(f'Rows: the window begins at {top} and ends at {bottom}; its title row ends at {row_end}, the bottom panel begins at '
          f'{panel_top}, the status bar at {status_top}.')
    print(f'Columns: the activity bar ends at {activity}, the left sidebar at {sidebar}; the right sidebar begins at {right}.')
    print(f'Scale: the status bar is {bottom - status_top} pixels of the picture for {desktop["status_bar_height"]} of the window: '
          f'{scale:.3f} to 1.')
    print(f'Title row, drawn stretches: the two toggles within {toggles}; the window controls {minimise}, {maximise}, {close}.')
    print()

    measured = [
        # (token, the token's value, the measurement in the token's unit, tolerance, how it was measured)
        ('desktop.title_bar_height', desktop['title_bar_height'], (row_end - top) / scale, 1.5,
         'the title row with its border, to the scale'),
        ('desktop.activity_bar_width', desktop['activity_bar_width'], activity / scale, 1.5,
         'the activity bar, to the scale'),
        ('desktop.window_control_width', desktop['window_control_width'], control_pitch / scale, 1.5,
         'the distance from one window control to the next, to the scale'),
        ('desktop.sidebar_toggle_width', desktop['sidebar_toggle_width'], (toggles[1] - toggles[0] + 1) / 2 / scale, 1.5,
         'half the frame that holds the two toggles, to the scale'),
        ('desktop_percent.left_sidebar_width', shares['left_sidebar_width'], 100 * sidebar / window, 1.0,
         'the activity bar and the left sidebar, of the window\'s width'),
        ('desktop_percent.right_sidebar_width', shares['right_sidebar_width'], 100 * (window - right) / (window - sidebar), 1.0,
         'the right sidebar, of the width beside the left sidebar'),
        ('desktop_percent.bottom_panel_height', shares['bottom_panel_height'], 100 * (status_top - panel_top) / (status_top - row_end), 1.0,
         'the bottom panel, of the height between the title row and the status bar'),
    ]
    failed = 0
    for name, token, value, tolerance, how in measured:
        within = abs(token - value) <= tolerance
        failed += not within
        print(f'{"OK  " if within else "FAIL"}  {name} = {token}; measured {value:.2f} ({how}); '
              f'difference {abs(token - value):.2f}, tolerance {tolerance}')
    print()
    print('EVERY TOKEN IS WITHIN ITS TOLERANCE' if not failed else f'{failed} TOKEN(S) OUTSIDE THE TOLERANCE')
    return 1 if failed else 0


if __name__ == '__main__':
    try:
        sys.exit(main(Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).resolve().parents[3]))
    except (Unreadable, OSError, KeyError, IndexError) as error:
        print(f'unreadable: {error}', file=sys.stderr)
        sys.exit(2)
