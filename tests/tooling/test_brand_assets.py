"""Tests for scripts/build/brand_assets.py: the logo reaches the repository only as the bound
source and as icons scaled down from it, never redrawn or substituted (B1).

The reader and the scaling are tested on small images made here, whose right answers can be
worked out by hand. The import and its check run against a throwaway folder with a stand-in
source. One test checks the repository itself: the committed logo files are the recorded source
and exactly the icons this script derives from it.
"""
from __future__ import annotations

import contextlib
import importlib.util
import io
import json
import struct
import tempfile
import unittest
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('brand_assets', ROOT / 'scripts' / 'build' / 'brand_assets.py')
brand_assets = importlib.util.module_from_spec(spec)
spec.loader.exec_module(brand_assets)

SIDE = 256  # the smallest stand-in source every icon size can be derived from


def chunk(kind: bytes, body: bytes) -> bytes:
    return struct.pack('>I', len(body)) + kind + body + struct.pack('>I', zlib.crc32(kind + body))


def png(side: int, rows: list[bytes], *, height: int | None = None, depth: int = 8, colour: int = 6,
        interlace: int = 0, filters: list[int] | None = None, extra: bytes = b'', pixels: bytes | None = None) -> bytes:
    """A PNG built by hand, so that a test can give it any header, row filter or extra chunk."""
    header = struct.pack('>IIBBBBB', side, side if height is None else height, depth, colour, 0, 0, interlace)
    if pixels is None:
        pixels = b''.join(bytes([kind]) + filtered(kind, row, rows[y - 1] if y else bytes(len(row)))
                          for y, (kind, row) in enumerate(zip(filters or [0] * len(rows), rows)))
    return (brand_assets.PNG_SIGNATURE + chunk(b'IHDR', header) + extra + chunk(b'IDAT', zlib.compress(pixels))
            + chunk(b'IEND', b''))


def filtered(kind: int, row: bytes, above: bytes) -> bytes:
    """One row with a PNG row filter applied, the inverse of what the reader undoes."""
    out = bytearray()
    for i, value in enumerate(row):
        left = row[i - 4] if i >= 4 else 0
        upper_left = above[i - 4] if i >= 4 else 0
        estimate = left + above[i] - upper_left
        nearest = min((left, above[i], upper_left), key=lambda candidate: abs(estimate - candidate))
        predicted = (0, left, above[i], (left + above[i]) // 2, nearest)[kind]
        out.append((value - predicted) & 255)
    return bytes(out)


def picture(side: int) -> list[bytes]:
    """A square test picture: colours that change in both directions, and a transparent quarter."""
    return [bytes(value for x in range(side)
                  for value in ((x * 7) & 255, (y * 5) & 255, (x + y) & 255, 0 if x < side // 2 > y else 255))
            for y in range(side)]


class ReaderTest(unittest.TestCase):
    def test_every_row_filter_reads_back_to_the_same_pixels(self) -> None:
        rows = picture(64)
        for kind in range(5):
            with self.subTest(filter=kind):
                side, decoded = brand_assets.decode_png(png(64, rows, filters=[kind] * 64))
                self.assertEqual((side, [bytes(row) for row in decoded]), (64, rows))
        # What the script writes, it reads back.
        self.assertEqual([bytes(row) for row in brand_assets.decode_png(brand_assets.encode_png(64, rows))[1]], rows)

    def test_anything_but_a_sound_square_8_bit_rgba_png_is_refused(self) -> None:
        rows = picture(64)
        good = png(64, rows)
        raw = b''.join(b'\0' + row for row in rows)
        cases = {
            'not a PNG': b'GIF89a' + good,
            'cut off': good[:-20],
            'a damaged chunk': good[:40] + bytes([good[40] ^ 1]) + good[41:],
            '16 bits per channel': png(64, rows, depth=16),
            'RGB without alpha': png(64, rows, colour=2),
            'a palette': png(64, rows, colour=3),
            'interlaced': png(64, rows, interlace=1),
            'not square': png(64, rows, height=32),
            'too small': png(32, picture(32)),
            'too large': png(brand_assets.MAX_SIDE + 1, [], pixels=b''),
            'an unknown critical chunk': png(64, rows, extra=chunk(b'XTRA', b'')),
            'an unknown row filter': png(64, rows, pixels=b'\x05' + raw[1:]),
            'fewer pixels than the header states': png(64, rows, pixels=raw[:-1]),
            'more pixels than the header states': png(64, rows, pixels=raw + b'\0'),
            'data after the pixels': good.replace(b'IEND', b'IDAT', 1),
            'no header first': brand_assets.PNG_SIGNATURE + chunk(b'IDAT', zlib.compress(raw)) + chunk(b'IEND', b''),
        }
        for name, data in cases.items():
            with self.subTest(name), self.assertRaises(brand_assets.Refused):
                brand_assets.decode_png(data)
        # An ancillary chunk does not change the pixels and is passed over.
        self.assertEqual(brand_assets.decode_png(png(64, rows, extra=chunk(b'tEXt', b'Comment\0x')))[0], 64)


class ScalingTest(unittest.TestCase):
    def test_an_icon_pixel_is_the_average_of_what_it_covers_weighted_by_how_much(self) -> None:
        # Three grey pixels a side become two: each icon pixel covers one source pixel fully and
        # half of the middle one, so the averages are (0*2 + 90)/3 = 30 and (90 + 180*2)/3 = 150.
        row = bytes([0, 0, 0, 255, 90, 90, 90, 255, 180, 180, 180, 255])
        scaled = brand_assets.scale_down(3, [bytearray(row)] * 3, 2)
        self.assertEqual(scaled, [bytes([30, 30, 30, 255, 150, 150, 150, 255])] * 2)
        self.assertEqual(brand_assets.coverage(3, 2), [[(0, 2), (1, 1)], [(1, 1), (2, 2)]])

    def test_a_transparent_pixel_adds_no_colour_of_its_own(self) -> None:
        # Transparent red beside opaque blue: the result is half-transparent blue, with no red in it.
        row = bytearray([255, 0, 0, 0, 0, 0, 255, 255])
        self.assertEqual(brand_assets.scale_down(2, [row, row], 1), [bytes([0, 0, 255, 128])])
        # Fully transparent stays fully transparent, whatever colour the source stored there.
        clear = bytearray([200, 100, 50, 0] * 4)
        self.assertEqual(brand_assets.scale_down(4, [clear] * 4, 2), [bytes(8)] * 2)

    def test_one_colour_stays_one_colour_at_every_size(self) -> None:
        colour = bytes([61, 157, 247, 200])
        rows = [bytearray(colour * 64) for _ in range(64)]
        for target in (1, 7, 16, 24, 63, 64):
            with self.subTest(target=target):
                self.assertEqual(brand_assets.scale_down(64, rows, target), [colour * target] * target)

    def test_scaling_up_is_refused(self) -> None:
        rows = [bytearray(4 * 64) for _ in range(64)]
        for target in (65, 128, 0, -1):
            with self.subTest(target=target), self.assertRaises(brand_assets.Refused):
                brand_assets.scale_down(64, rows, target)


class ImportTest(unittest.TestCase):
    def setUp(self) -> None:
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.root = Path(tmp.name) / 'checkout'
        self.branding = self.root / brand_assets.BRANDING
        self.source = Path(tmp.name) / 'logo.png'
        self.source.write_bytes(png(SIDE, picture(SIDE), filters=[y % 5 for y in range(SIDE)]))

    def test_import_writes_the_source_byte_for_byte_its_icons_and_a_manifest_that_checks_clean(self) -> None:
        manifest = brand_assets.import_logo(self.root, self.source)
        self.assertEqual((self.branding / brand_assets.SOURCE_COPY).read_bytes(), self.source.read_bytes())
        self.assertEqual(json.loads((self.branding / brand_assets.MANIFEST).read_text()), manifest)
        self.assertEqual([entry['size'] for entry in manifest['derived']], list(brand_assets.ICON_SIZES))
        for entry in manifest['derived']:
            side, rows = brand_assets.decode_png((self.branding / entry['file']).read_bytes(), smallest=1)
            self.assertEqual((side, len(rows)), (entry['size'], entry['size']))
        self.assertEqual(brand_assets.check(self.root, self.source), [])
        # A machine without the bound source, such as CI, checks the files against the manifest alone.
        self.assertEqual(brand_assets.check(self.root, self.source.with_name('absent.png')), [])

    def test_a_source_that_is_missing_or_refused_writes_nothing(self) -> None:
        bad = self.source.with_name('bad.png')
        bad.write_bytes(png(SIDE, picture(SIDE), colour=2))
        for source in (self.source.with_name('absent.png'), bad):
            with self.subTest(source=source.name), self.assertRaises(brand_assets.Refused):
                brand_assets.import_logo(self.root, source)
            self.assertFalse(self.root.exists())

    def test_check_reports_every_way_the_files_can_stop_being_the_recorded_logo(self) -> None:
        def rewrite_manifest(change) -> None:
            manifest = json.loads((self.branding / brand_assets.MANIFEST).read_text())
            change(manifest)
            (self.branding / brand_assets.MANIFEST).write_text(json.dumps(manifest))

        def another_icon(manifest: dict) -> None:
            # A different picture of the right size, with its hash recorded: a substituted icon.
            other = brand_assets.encode_png(48, [bytes([1, 2, 3, 255] * 48)] * 48)
            (self.branding / brand_assets.derived_name(48)).write_bytes(other)
            next(e for e in manifest['derived'] if e['size'] == 48)['sha256'] = brand_assets.sha256(other)

        def another_source(manifest: dict) -> None:
            other = png(SIDE, [bytes(reversed(row)) for row in picture(SIDE)])
            (self.branding / brand_assets.SOURCE_COPY).write_bytes(other)
            manifest['source']['sha256'] = brand_assets.sha256(other)

        cases = {
            'an icon changed': (lambda: (self.branding / brand_assets.derived_name(16)).write_bytes(b'x'),
                                'does not have the SHA-256'),
            'the source copy changed': (lambda: (self.branding / brand_assets.SOURCE_COPY).write_bytes(b'x'),
                                        'does not have the SHA-256'),
            'an icon is missing': ((self.branding / brand_assets.derived_name(32)).unlink, 'is missing'),
            'a file nobody recorded': (lambda: (self.branding / 'derived/extra.png').write_bytes(b'x'),
                                       'is not in the manifest'),
            'a substituted icon, recorded': (lambda: rewrite_manifest(another_icon), 'is not the source scaled down'),
            'a substituted source, recorded': (lambda: rewrite_manifest(another_source), 'is not the source scaled'),
            'a size dropped from the manifest': (lambda: rewrite_manifest(lambda m: m['derived'].pop()),
                                                 'exactly the icon sizes'),
            'a manifest that is not one': (lambda: (self.branding / brand_assets.MANIFEST).write_text('[]'),
                                           'cannot be read as a logo manifest'),
            'no manifest': ((self.branding / brand_assets.MANIFEST).unlink, 'cannot be read as a logo manifest'),
            'the bound source changed': (lambda: self.source.write_bytes(png(SIDE, picture(SIDE))),
                                         'is no longer the file'),
        }
        for name, (damage, expected) in cases.items():
            with self.subTest(name):
                brand_assets.import_logo(self.root, self.source)
                original = self.source.read_bytes()
                damage()
                problems = brand_assets.check(self.root, self.source)
                self.assertTrue(problems and any(expected in problem for problem in problems), problems)
                self.source.write_bytes(original)
                for stray in self.branding.glob('derived/extra.png'):
                    stray.unlink()

    def test_the_command_line_offers_no_way_to_name_another_source(self) -> None:
        for arguments in (['import', str(self.source)], ['import', '--source', str(self.source)], []):
            with self.subTest(arguments=arguments), contextlib.redirect_stderr(io.StringIO()):
                with self.assertRaises(SystemExit) as stopped:
                    brand_assets.main(arguments)
                self.assertEqual(stopped.exception.code, 2)


class RepositoryTest(unittest.TestCase):
    def test_the_committed_logo_files_are_the_recorded_source_and_exactly_its_icons(self) -> None:
        self.assertEqual(brand_assets.check(ROOT), [])
        manifest = json.loads((ROOT / brand_assets.BRANDING / brand_assets.MANIFEST).read_text(encoding='utf-8'))
        self.assertEqual(manifest['source']['bound_path'], str(brand_assets.LOGO_SOURCE))


if __name__ == '__main__':
    unittest.main()
