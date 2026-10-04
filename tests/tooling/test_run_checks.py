"""Tests for scripts/test/run_checks.py: the rules its stages apply, and the guarantee that a run
reads the checkout but never writes into it, so pre-existing untracked user work is left exactly
as it was (the TASK-005 verification gate).

The rule tests call the runner's functions directly. The fixture test runs the runner as a
separate process against a throwaway Git checkout and compares every file before and after.
"""
from __future__ import annotations

import hashlib
import importlib.util
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

RUNNER = Path(__file__).resolve().parents[2] / 'scripts' / 'test' / 'run_checks.py'
ENV = {**os.environ, 'GIT_CONFIG_GLOBAL': os.devnull, 'GIT_CONFIG_NOSYSTEM': '1'}
spec = importlib.util.spec_from_file_location('run_checks', RUNNER)
run_checks = importlib.util.module_from_spec(spec)
spec.loader.exec_module(run_checks)


class EditorConfigTest(unittest.TestCase):
    def sections(self, text: str) -> list:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / '.editorconfig'
            path.write_text(text)
            return run_checks.editorconfig(path)

    def test_later_sections_override_earlier_ones_and_braces_expand(self) -> None:
        sections = self.sections('root = true\n[*]\ntrim_trailing_whitespace = true\n'
                                 '[*.{md,txt}]\ntrim_trailing_whitespace = false\n[*.py]\nmax_line_length = 120\n')
        self.assertEqual(run_checks.properties(sections, 'docs/a.md'), {'trim_trailing_whitespace': 'false'})
        self.assertEqual(run_checks.properties(sections, 'x/tool.py'),
                         {'trim_trailing_whitespace': 'true', 'max_line_length': '120'})

    def test_a_glob_form_the_checker_cannot_read_fails_instead_of_being_ignored(self) -> None:
        with self.assertRaises(run_checks.Unavailable):
            self.sections('[src/**.rs]\nindent_size = 4\n')

    def test_each_enforced_property_reports_its_violation(self) -> None:
        props = {'end_of_line': 'lf', 'insert_final_newline': 'true', 'trim_trailing_whitespace': 'true',
                 'indent_style': 'space', 'max_line_length': '10'}
        problems = run_checks.format_problems('f.py', 'ok\r\n\tindented\ntrailing \nwaytoolongline', props)
        # The CR is reported once, as a line ending, not again as trailing whitespace.
        self.assertEqual(problems, ['has CR line endings', 'does not end with a newline', 'line 2 indents with a tab',
                                    'line 3 has trailing whitespace', 'line 4 is longer than 10 characters'])
        self.assertEqual(run_checks.format_problems('f.py', 'clean\n', props), [])


class MarkdownLinkTest(unittest.TestCase):
    def test_only_relative_targets_outside_code_are_checked(self) -> None:
        text = ('[a](docs/x.md#part) [b](https://example.org) [c](#top) [d](JaVaScRiPt:alert(1)) '
                '[e](//example.org/x) `[f](in-code.md)`\n```\n[g](in-block.md)\n```\n[h](../up.md "title")\n')
        self.assertEqual(run_checks.markdown_links(text), ['docs/x.md', '../up.md'])


class BrokenLinkTest(unittest.TestCase):
    def test_missing_targets_are_reported_except_an_open_tasks_receipt_and_fixtures(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / 'docs/evidence/TASK-009').mkdir(parents=True)
            (root / 'docs/present.md').write_text('# Present\n')
            (root / 'docs/evidence/TASK-009/TASK-009_EVIDENCE.md').write_text(
                '[result](receipt/RECEIPT_RESULT.md) [here](../../present.md) [gone](missing.md)\n')
            (root / 'tests/fixtures').mkdir(parents=True)
            (root / 'tests/fixtures/hostile.md').write_text('[broken](nowhere.md)\n')
            runner = run_checks.Runner(root, root.parent / 'target', online=False)
            files = ['docs/present.md', 'docs/evidence/TASK-009/TASK-009_EVIDENCE.md', 'tests/fixtures/hostile.md']
            problems, checked = runner.broken_links(files)
            self.assertEqual(problems,
                             ['docs/evidence/TASK-009/TASK-009_EVIDENCE.md links to missing.md, which does not exist'])
            self.assertEqual(checked, 3)  # the fixture's link is not even counted


class GradleOutputTest(unittest.TestCase):
    """Ignored files under docs/evidence are excused only as Gradle or IDE output beside a Gradle build file."""

    PROTOTYPE = 'docs/evidence/TASK-003/prototypes/android-app'
    FILES = {f'{PROTOTYPE}/settings.gradle.kts', f'{PROTOTYPE}/app/build.gradle.kts'}

    def test_output_beside_a_gradle_build_file_is_excused(self) -> None:
        for path in (f'{self.PROTOTYPE}/.gradle/8.9/checksums/checksums.lock', f'{self.PROTOTYPE}/.kotlin/errors/x',
                     f'{self.PROTOTYPE}/app/build/outputs/app.apk'):
            self.assertTrue(run_checks.gradle_output(path, self.FILES), path)

    def test_anything_else_ignored_under_evidence_stays_a_finding(self) -> None:
        for path in ('docs/evidence/TASK-006/logs/run.log', 'docs/evidence/TASK-006/logs/build/out.txt',
                     f'{self.PROTOTYPE}/app/src/build.log', f'{self.PROTOTYPE}/.gradle'):
            self.assertFalse(run_checks.gradle_output(path, self.FILES), path)
        # A Gradle folder only counts beside a build file that the repository itself holds.
        self.assertFalse(run_checks.gradle_output(f'{self.PROTOTYPE}/.gradle/x', set()))


class EnforcementPointTest(unittest.TestCase):
    """The threat model's enforcement points must name slots, whether still placeholders or implemented."""

    def test_points_are_read_from_every_security_record(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            Path(tmp, 'threats.lcl.txt').write_text(
                '        enforcement_points: ["core/protocol/messages", "core/remote/command_dispatch"]\n')
            Path(tmp, 'invariants.lcl.txt').write_text('        enforcement_points: ["core/tools/"]\n')
            self.assertEqual(run_checks.enforcement_points(Path(tmp)),
                             {'core/protocol/messages', 'core/remote/command_dispatch', 'core/tools/'})

    def test_a_point_resolves_as_a_placeholder_or_as_implemented_source(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            Path(tmp, 'core/tools').mkdir(parents=True)
            files = ['core/protocol/messages.rs', 'core/remote/command_dispatch.source']
            points = {'core/protocol/messages', 'core/remote/command_dispatch', 'core/tools/', 'core/protocol/gone',
                      'core/missing/'}
            self.assertEqual(run_checks.unresolved_points(points, files, Path(tmp)),
                             ['core/missing/', 'core/protocol/gone'])


class PinTest(unittest.TestCase):
    def test_only_exact_pins_and_workspace_or_path_references_pass(self) -> None:
        manifest = {
            'dependencies': {'exact': '=1.2.3', 'caret': '1.2.3', 'table': {'version': '=0.4.0', 'features': ['x']},
                             'tilde': {'version': '~1.0'}, 'shared': {'workspace': True}, 'local': {'path': '../a'},
                             'remote': {'git': 'https://example.org/repo'}},
            'workspace': {'dependencies': {'star': '*'}},
            'target': {'cfg(unix)': {'dependencies': {'unix': '>=1'}}},
        }
        self.assertEqual(run_checks.pin_problems(manifest), [
            'dependencies.caret = "1.2.3" is not an exact pin ("=x.y.z")',
            'dependencies.tilde = "~1.0" is not an exact pin ("=x.y.z")',
            'dependencies.remote comes from a git or alternative registry source',
            'target.cfg(unix).dependencies.unix = ">=1" is not an exact pin ("=x.y.z")',
            'workspace.dependencies.star = "*" is not an exact pin ("=x.y.z")',
        ])


class NpmGateTest(unittest.TestCase):
    """The npm lockfile gate (DS-05, DS-06, DS-08) on a small package and its lockfile."""

    def setUp(self) -> None:
        self.manifest = {
            'dependencies': {'left-pad': '1.3.0', 'shell': 'file:src'},
            'devDependencies': {'builder': '2.0.0'},
            'overrides': {'left-pad': '1.3.0'},
            'allowScripts': {'builder@2.0.0': True},
        }
        registry = run_checks.NPM_REGISTRY
        self.lock = {'lockfileVersion': 3, 'packages': {
            '': {'name': 'app'},
            'node_modules/left-pad': {'version': '1.3.0', 'resolved': f'{registry}left-pad/-/left-pad-1.3.0.tgz',
                                      'integrity': 'sha512-AAAA'},
            'node_modules/builder': {'version': '2.0.0', 'resolved': f'{registry}builder/-/builder-2.0.0.tgz',
                                     'integrity': 'sha512-BBBB', 'hasInstallScript': True, 'dev': True},
            'node_modules/shell': {'resolved': 'src', 'link': True},
            'src': {'name': 'shell', 'version': '0.0.0'},
        }}
        packages = run_checks.npm_package_set(self.lock)
        self.review = {'package_set_sha256': hashlib.sha256(''.join(f'{p}\n' for p in packages).encode()).hexdigest(),
                       'advisories': [{'package': 'npm:builder@2.0.0', 'id': 'GHSA-1111'}]}
        self.dispositions = 'VALUE: ["builder 2.0.0 | GHSA-1111 (low) | build only | accepted"]'

    def problems(self) -> list:
        return run_checks.npm_problems(self.manifest, self.lock, self.review, self.dispositions)

    def test_a_reviewed_lockfile_of_exact_registry_packages_passes(self) -> None:
        self.assertEqual(run_checks.npm_package_set(self.lock), ['builder@2.0.0', 'left-pad@1.3.0'])
        self.assertEqual(self.problems(), [])

    def test_ranges_and_outside_folders_are_not_pins(self) -> None:
        self.manifest['dependencies'].update({'caret': '^1.0.0', 'up': 'file:../other', 'git': 'github:a/b'})
        self.manifest['overrides']['shell'] = 'file:src'
        self.assertEqual(len([p for p in self.problems() if 'neither an exact version' in p]), 4)

    def test_another_source_a_missing_hash_or_a_link_outside_is_refused(self) -> None:
        self.lock['packages']['node_modules/left-pad'].update(resolved='https://example.org/left-pad.tgz')
        del self.lock['packages']['node_modules/builder']['integrity']
        self.lock['packages']['node_modules/shell']['resolved'] = '../elsewhere'
        problems = self.problems()
        self.assertIn('node_modules/left-pad comes from https://example.org/left-pad.tgz, not the npm registry',
                      problems)
        self.assertIn('node_modules/builder has no sha512 integrity hash', problems)
        self.assertIn('node_modules/shell links outside the package', problems)

    def test_an_install_script_needs_an_allowscripts_decision(self) -> None:
        self.manifest['allowScripts'] = {}
        self.assertEqual(self.problems(),
                         ['builder@2.0.0 has an install script without an allowScripts decision (DS-06)'])
        self.manifest['allowScripts'] = {'builder': False}
        self.assertEqual(self.problems(), [], 'a denial is a decision too')

    def test_a_changed_lockfile_needs_a_new_review(self) -> None:
        self.lock['packages']['node_modules/left-pad']['version'] = '1.3.1'
        self.assertTrue(any('differ from the reviewed set' in p for p in self.problems()))
        self.assertIn('no advisory review is recorded in docs/dependencies/npm_review.json (DS-08)',
                      run_checks.npm_problems(self.manifest, self.lock, None, self.dispositions))

    def test_every_reviewed_advisory_needs_a_disposition(self) -> None:
        self.review['advisories'].append({'package': 'npm:left-pad@1.3.0', 'id': 'GHSA-2222'})
        self.assertEqual(self.problems(),
                         ['left-pad@1.3.0 GHSA-2222 has no recorded disposition in docs/dependencies/strategy.lcl.txt'])


class ClosureTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.folder = Path(self.tmp.name) / 'TASK-007'
        (self.folder / 'receipt').mkdir(parents=True)
        (self.folder / 'receipt' / 'final_verification.txt').write_text('Result: passed\n')
        self.snapshot = 'a' * 64

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def receipt(self, **changes) -> dict:
        closure = {
            'schema': 'nexees-task-closure', 'version': 1, 'task': 'TASK-007',
            'readability': {'reviewer': 'the coding agent', 'summary': 'comments match the code'},
            'cleanup': {'summary': 'nothing task-owned was left', 'record': None, 'removed': [],
                        'preserved_untracked': 0},
            'manual_impact': {'status': 'not_applicable', 'detail': 'no user-visible change'},
            'final_verification': {'log': 'receipt/final_verification.txt', 'snapshot': self.snapshot,
                                   'result': 'passed'},
        }
        closure.update(changes)
        return {'inputs': {'current_snapshot_sha256': self.snapshot}, 'closure': closure}

    def test_a_complete_record_passes(self) -> None:
        self.assertEqual(run_checks.closure_problems(self.receipt(), self.folder), [])

    def test_each_missing_or_doubtful_item_is_a_problem(self) -> None:
        cases = {
            'no closure': ({'inputs': {}}, 'the receipt has no closure record'),
            'an unknown field': (self.receipt(extra=1), 'closure must have exactly'),
            'another version': (self.receipt(version=2), 'schema nexees-task-closure version 1'),
            'another task': (self.receipt(task='TASK-008'), 'not TASK-007'),
            'an unknown manual status': (self.receipt(manual_impact={'status': 'later', 'detail': 'x'}),
                                         'updated or not_applicable'),
            'removals without a record': (self.receipt(cleanup={'summary': 's', 'record': None, 'removed': ['tmp/x'],
                                                                'preserved_untracked': 0}), 'without a cleanup record'),
            'another snapshot': (self.receipt(final_verification={'log': 'receipt/final_verification.txt',
                                                                  'snapshot': 'b' * 64, 'result': 'passed'}),
                                 'differs from the receipt snapshot'),
            'an empty review': (self.receipt(readability={'reviewer': '', 'summary': 'x'}), 'non-empty string'),
        }
        for name, (receipt, expected) in cases.items():
            with self.subTest(name):
                problems = run_checks.closure_problems(receipt, self.folder)
                self.assertTrue(any(expected in p for p in problems), problems)

    def test_a_cleanup_record_must_agree_with_the_closure(self) -> None:
        record = {'schema': 'nexees-cleanup-record', 'version': 1, 'applied': True, 'removed': ['tmp/x'],
                  'preserved_untracked': [{'path': 'notes.txt', 'sha256': 'c' * 64}]}
        (self.folder / 'cleanup.json').write_text(json.dumps(record))
        agreeing = {'summary': 's', 'record': 'cleanup.json', 'removed': ['tmp/x'], 'preserved_untracked': 1}
        self.assertEqual(run_checks.closure_problems(self.receipt(cleanup=agreeing), self.folder), [])
        disagreeing = {**agreeing, 'preserved_untracked': 3}
        self.assertIn('closure.cleanup disagrees with its cleanup record',
                      run_checks.closure_problems(self.receipt(cleanup=disagreeing), self.folder))


class FileTreeTest(unittest.TestCase):
    def test_the_tree_lists_folders_first_then_files_and_leaves_out_folder_settings(self) -> None:
        files = ['b.txt', 'a/z.md', 'a/b/c.rs', '.gitignore', 'a/.directory']
        tree = run_checks.render_file_tree(files)
        self.assertEqual(tree, 'Nexees/\n├── a/\n│   ├── b/\n│   │   └── c.rs\n│   └── z.md\n'
                               '├── .gitignore\n└── b.txt\n')

    def test_an_open_task_has_its_receipt_files_planned_and_a_closed_one_does_not(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / 'docs/evidence/TASK-008/receipt').mkdir(parents=True)
            (root / 'docs/evidence/TASK-009/logs').mkdir(parents=True)
            runner = run_checks.Runner(root, root.parent / 'target', online=False)
            self.assertEqual(runner.planned_receipt_files(),
                             {f'docs/evidence/TASK-009/receipt/{name}' for name in run_checks.RECEIPT_FILES})

    def test_stems_drop_the_extension_but_keep_the_folder(self) -> None:
        self.assertEqual(run_checks.stem('scripts/test/run_checks.py'), 'scripts/test/run_checks')
        self.assertEqual(run_checks.stem('core/domain/lib.rs'), 'core/domain/lib')


class ReadOnlyRunTest(unittest.TestCase):
    """A run against a checkout full of user work changes nothing in it."""

    def snapshot(self, root: Path) -> dict:
        return {str(p.relative_to(root)): (hashlib.sha256(p.read_bytes()).hexdigest() if p.is_file() else 'dir')
                for p in root.rglob('*') if '.git' not in p.relative_to(root).parts}

    def test_the_runner_reads_but_never_writes_the_checkout(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / 'checkout'
            root.mkdir()
            subprocess.run(['git', 'init', '-q'], cwd=root, env=ENV, check=True)
            (root / '.editorconfig').write_text('root = true\n[*]\ntrim_trailing_whitespace = true\n')
            (root / 'tracked.txt').write_text('tracked\n')
            subprocess.run(['git', 'add', '.'], cwd=root, env=ENV, check=True)
            # Pre-existing untracked user work, including findings the runner must report but never fix.
            (root / 'notes.md').write_text('the user\'s untracked notes\n')
            (root / 'drafts').mkdir()
            (root / 'drafts' / 'messy.txt').write_text('trailing spaces   \n')
            (root / 'drafts' / 'token.txt').write_text('token = ' + 'gh' + 'p_' + 'x' * 36 + '\n')
            before = self.snapshot(root)
            result = subprocess.run([sys.executable, '-B', str(RUNNER), '--root', str(root), '--stage', 'format',
                                     '--stage', 'security', '--target-dir', str(Path(tmp) / 'target')],
                                    env=ENV, capture_output=True, text=True)
            self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
            self.assertIn('drafts/messy.txt: line 1 has trailing whitespace', result.stdout)
            self.assertIn('drafts/token.txt:1: GitHub token', result.stdout)
            self.assertEqual(self.snapshot(root), before)

    def test_a_target_folder_inside_the_checkout_is_refused(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            result = subprocess.run([sys.executable, '-B', str(RUNNER), '--root', tmp, '--target-dir',
                                     str(Path(tmp) / 'target')], env=ENV, capture_output=True, text=True)
            self.assertEqual(result.returncode, 2)
            self.assertFalse((Path(tmp) / 'target').exists())


if __name__ == '__main__':
    unittest.main()
