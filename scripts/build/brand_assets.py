#!/usr/bin/env python3
"""Bring the Nexees logo into the repository and derive the icon sizes packaging needs (B1).

    brand_assets.py import   copy the logo source into assets/branding/ and derive the icons from it
    brand_assets.py check    verify the copy and every derived icon; change nothing

The logo has one source: the file the owner bound, LOGO_SOURCE below. `import` reads only that
file and offers no way to name another, because binding B1 forbids substituting the logo. It
writes three things under assets/branding/:

    source/nexees-logo.png    the source, byte for byte
    derived/nexees-<n>.png    one square icon per size in ICON_SIZES, scaled down from the source
    manifest.json             the source's path, size and SHA-256, and each derived icon's SHA-256

Builds, packages and tests use those files, so only the owner's machine needs the bound file.
`check` is what keeps them honest: the files under source/ and derived/ are exactly the
manifest's, each has the recorded SHA-256, each icon holds exactly the pixels this script
derives from the source copy, and, where the bound file exists, the copy still equals it.

Nothing is redrawn. An icon is the source scaled down by area averaging: every icon pixel is the
average of the source pixels it covers, weighted by how much of each it covers. Colours are
averaged in proportion to their opacity, so a transparent pixel adds no colour of its own to an
edge. The arithmetic is in whole numbers, which makes the result the same on every machine. The
script never scales up: a size larger than the source is refused.

The script reads one kind of PNG, the kind the logo is: 8 bits per channel, RGBA, not
interlaced, square, between 64 and 4096 pixels wide. Anything else is refused before a file is
written.

Exit codes: 0 done, 1 `check` found a difference, 2 refused or wrong usage.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import struct
import sys
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
# The logo source the owner bound (B1). The character between "Logo" and "Icon" is U+2044
# FRACTION SLASH, part of the file's name: the file lies directly in the Pictures folder.
LOGO_SOURCE = Path('/home/aivars/Pictures/Nexees Logo⁄Icon.png')
BRANDING = Path('assets/branding')
SOURCE_COPY = 'source/nexees-logo.png'
MANIFEST = 'manifest.json'
# The sizes the Desktop package installs into the hicolor icon theme. The window shows the
# 128-pixel icon in its About dialog.
ICON_SIZES = (16, 24, 32, 48, 64, 128, 256)

PNG_SIGNATURE = b'\x89PNG\r\n\x1a\n'
MIN_SIDE, MAX_SIDE = 64, 4096
MAX_FILE_BYTES = 16 * 1024 * 1024
DESCRIPTION = ('The Nexees logo in the repository: the byte-identical copy of the source the owner bound (B1) and '
               'the icons derived from it. Written by scripts/build/brand_assets.py import, which reads only the '
               'bound source; brand_assets.py check verifies every file named here.')


class Refused(Exception):
    """The input is not acceptable; nothing was written."""


def derived_name(size: int) -> str:
    return f'derived/nexees-{size}.png'


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def decode_png(data: bytes, smallest: int = MIN_SIDE) -> tuple[int, list[bytearray]]:
    """The side length and the rows of a square 8-bit RGBA PNG, four bytes per pixel. Refuses any
    other PNG, a damaged one, one whose pixel data is not exactly the size its header states, and
    one narrower than `smallest`: a source must be at least MIN_SIDE wide, a derived icon need not."""
    if len(data) > MAX_FILE_BYTES:
        raise Refused(f'the file is larger than {MAX_FILE_BYTES} bytes')
    if not data.startswith(PNG_SIGNATURE):
        raise Refused('the file is not a PNG image')
    header, compressed, at, ended = None, bytearray(), len(PNG_SIGNATURE), False
    while not ended:
        if at + 12 > len(data):
            raise Refused('the PNG ends before its last chunk')
        length, kind = struct.unpack('>I4s', data[at:at + 8])
        body = data[at + 8:at + 8 + length]
        (crc,) = struct.unpack('>I', data[at + 8 + length:at + 12 + length].ljust(4, b'\0'))
        if len(body) != length or zlib.crc32(kind + body) != crc:
            raise Refused(f'the PNG chunk {kind!r} is damaged')
        at += 12 + length
        if header is None and kind != b'IHDR':
            raise Refused('the PNG does not start with its header')
        if kind == b'IHDR':
            if header is not None or length != 13:
                raise Refused('the PNG header is not valid')
            header = struct.unpack('>IIBBBBB', body)
        elif kind == b'IDAT':
            compressed += body
        elif kind == b'IEND':
            ended = True
        elif not kind[:1].islower():
            # A critical chunk this reader does not know would change what the pixels mean.
            raise Refused(f'the PNG holds the chunk {kind!r}, which this reader does not support')
    width, height, depth, colour, compression, filtering, interlace = header
    if (depth, colour, compression, filtering, interlace) != (8, 6, 0, 0, 0):
        raise Refused('the PNG is not 8 bits per channel, RGBA and not interlaced')
    if width != height or not smallest <= width <= MAX_SIDE:
        raise Refused(f'the image is {width} x {height}; a square between {smallest} and {MAX_SIDE} pixels is needed')
    stride = 4 * width
    expected = height * (1 + stride)
    inflater = zlib.decompressobj()
    try:
        raw = inflater.decompress(bytes(compressed), expected + 1)
    except zlib.error as error:
        raise Refused(f'the PNG pixel data does not decompress: {error}') from error
    if len(raw) != expected or not inflater.eof or inflater.unused_data:
        raise Refused('the PNG pixel data is not the size its header states')
    rows: list[bytearray] = []
    previous = bytearray(stride)
    for y in range(height):
        start = y * (1 + stride)
        row = unfilter(raw[start], bytearray(raw[start + 1:start + 1 + stride]), previous)
        rows.append(row)
        previous = row
    return width, rows


def unfilter(kind: int, row: bytearray, previous: bytearray) -> bytearray:
    """Undoes one row's PNG filter in place; `previous` is the row above, already unfiltered."""
    if kind == 0:
        return row
    if kind == 2:
        return bytearray((value + above) & 255 for value, above in zip(row, previous))
    if kind not in (1, 3, 4):
        raise Refused(f'the PNG uses the unknown row filter {kind}')
    for i, value in enumerate(row):
        left = row[i - 4] if i >= 4 else 0
        above = previous[i]
        if kind == 1:
            predicted = left
        elif kind == 3:
            predicted = (left + above) // 2
        else:
            upper_left = previous[i - 4] if i >= 4 else 0
            estimate = left + above - upper_left
            distances = (abs(estimate - left), abs(estimate - above), abs(estimate - upper_left))
            predicted = (left, above, upper_left)[distances.index(min(distances))]
        row[i] = (value + predicted) & 255
    return row


def encode_png(side: int, rows: list[bytes]) -> bytes:
    """A square 8-bit RGBA PNG of the rows, four bytes per pixel."""
    def chunk(kind: bytes, body: bytes) -> bytes:
        return struct.pack('>I', len(body)) + kind + body + struct.pack('>I', zlib.crc32(kind + body))

    pixels = b''.join(b'\0' + bytes(row) for row in rows)
    return (PNG_SIGNATURE + chunk(b'IHDR', struct.pack('>IIBBBBB', side, side, 8, 6, 0, 0, 0))
            + chunk(b'IDAT', zlib.compress(pixels, 9)) + chunk(b'IEND', b''))


def coverage(source: int, target: int) -> list[list[tuple[int, int]]]:
    """For each of `target` pixels along one axis, the source pixels it covers and how much of
    each, in units of 1/target of a source pixel. The weights of one target pixel add up to
    `source`."""
    result = []
    for j in range(target):
        first, last = j * source // target, ((j + 1) * source - 1) // target
        result.append([(i, min((i + 1) * target, (j + 1) * source) - max(i * target, j * source))
                       for i in range(first, last + 1)])
    return result


def scale_down(side: int, rows: list[bytearray], target: int) -> list[bytes]:
    """The image scaled down to `target` pixels a side by area averaging, in whole numbers.

    Colour is weighted by opacity: each channel is summed as colour times alpha, and divided by
    the summed alpha at the end, so transparent pixels do not darken or tint the edges."""
    if not 0 < target <= side:
        raise Refused(f'a {side}-pixel source cannot be scaled to {target} pixels: icons are only scaled down')
    cover = coverage(side, target)
    # Horizontally: per source row, four planes (red, green and blue times alpha, and alpha).
    narrowed = []
    for row in rows:
        alpha = row[3::4]
        planes = [[c * a for c, a in zip(row[channel::4], alpha)] for channel in range(3)] + [list(alpha)]
        narrowed.append([[sum(weight * plane[i] for i, weight in taps) for taps in cover] for plane in planes])
    result = []
    for taps in cover:
        sums = [[sum(weight * narrowed[i][plane][x] for i, weight in taps) for x in range(target)]
                for plane in range(4)]
        out = bytearray(4 * target)
        for x in range(target):
            total_alpha = sums[3][x]
            if total_alpha:
                for channel in range(3):
                    out[4 * x + channel] = (2 * sums[channel][x] + total_alpha) // (2 * total_alpha)
                out[4 * x + 3] = (2 * total_alpha + side * side) // (2 * side * side)
        result.append(bytes(out))
    return result


def import_logo(root: Path, source: Path) -> dict:
    """Copies `source` into the checkout at `root`, derives the icons and writes the manifest,
    which it returns. Everything is read and derived before the first file is written."""
    if not source.is_file():
        raise Refused(f'the logo source {source} does not exist (binding B1); logo work stops here')
    data = source.read_bytes()
    side, rows = decode_png(data)
    icons = {size: encode_png(size, scale_down(side, rows, size)) for size in ICON_SIZES}
    manifest = {
        'description': DESCRIPTION,
        'source': {'bound_path': str(source), 'copy': SOURCE_COPY, 'sha256': sha256(data), 'width': side,
                   'height': side},
        'derived': [{'file': derived_name(size), 'size': size, 'sha256': sha256(icon)}
                    for size, icon in icons.items()],
    }
    branding = root / BRANDING
    files = {SOURCE_COPY: data, **{derived_name(size): icon for size, icon in icons.items()},
             MANIFEST: (json.dumps(manifest, indent=2, ensure_ascii=False) + '\n').encode()}
    for relative, content in files.items():
        (branding / relative).parent.mkdir(parents=True, exist_ok=True)
        (branding / relative).write_bytes(content)
    return manifest


def check(root: Path, bound_source: Path | None = LOGO_SOURCE) -> list[str]:
    """Everything that is wrong with the logo files in the checkout at `root`; empty when they are
    exactly what `import` writes from the recorded source."""
    branding = root / BRANDING
    try:
        manifest = json.loads((branding / MANIFEST).read_text(encoding='utf-8'))
        source, derived = manifest['source'], manifest['derived']
        recorded = {source['copy']: source['sha256'], **{entry['file']: entry['sha256'] for entry in derived}}
        sizes = {entry['file']: entry['size'] for entry in derived}
    except (OSError, ValueError, KeyError, TypeError) as error:
        return [f'{BRANDING / MANIFEST} cannot be read as a logo manifest: {error!r}']
    problems = []
    if source['copy'] != SOURCE_COPY or sizes != {derived_name(size): size for size in ICON_SIZES}:
        problems.append(f'the manifest does not name the source copy and exactly the icon sizes {ICON_SIZES}')
    present = {str(path.relative_to(branding)) for folder in ('source', 'derived')
               for path in (branding / folder).glob('*') if path.is_file()}
    for extra in sorted(present - set(recorded)):
        problems.append(f'{BRANDING / extra} is not in the manifest')
    for missing in sorted(set(recorded) - present):
        problems.append(f'{BRANDING / missing} is missing')
    contents = {name: (branding / name).read_bytes() for name in sorted(set(recorded) & present)}
    for name, data in contents.items():
        if sha256(data) != recorded[name]:
            problems.append(f'{BRANDING / name} does not have the SHA-256 the manifest records')
    if problems:
        return problems
    try:
        side, rows = decode_png(contents[SOURCE_COPY])
        if (side, side) != (source['width'], source['height']):
            problems.append('the manifest records another size for the source than the source copy has')
        for name, size in sizes.items():
            icon_side, icon_rows = decode_png(contents[name], smallest=1)
            if (icon_side, [bytes(row) for row in icon_rows]) != (size, scale_down(side, rows, size)):
                problems.append(f'{BRANDING / name} is not the source scaled down to {size} pixels')
    except Refused as refused:
        problems.append(f'a logo file cannot be read: {refused}')
    if bound_source is not None and bound_source.is_file() and sha256(bound_source.read_bytes()) != source['sha256']:
        problems.append(f'the bound logo source {bound_source} is no longer the file the repository copy was made from')
    return problems


def main(arguments: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    parser.add_argument('command', choices=['import', 'check'])
    args = parser.parse_args(arguments)
    if args.command == 'check':
        problems = check(ROOT)
        for problem in problems:
            print(f'FINDING: {problem}')
        where = 'compared with the bound source' if LOGO_SOURCE.is_file() else 'the bound source is not on this machine'
        print(f'logo files checked ({where}): {len(problems)} finding(s)')
        return 1 if problems else 0
    try:
        manifest = import_logo(ROOT, LOGO_SOURCE)
    except Refused as refused:
        print(f'refused: {refused}', file=sys.stderr)
        return 2
    source = manifest['source']
    print(f'imported {source["bound_path"]} ({source["width"]} x {source["height"]}, SHA-256 {source["sha256"]})')
    print(f'derived {len(manifest["derived"])} icons: {", ".join(str(size) for size in ICON_SIZES)} pixels')
    return 0


if __name__ == '__main__':
    sys.exit(main())
