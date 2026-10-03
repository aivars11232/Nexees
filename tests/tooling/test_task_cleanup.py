"""Fixture tests for scripts/test/task_cleanup.py: task-close cleanup removes only the task's own
unchanged artifacts and leaves pre-existing untracked user work byte-for-byte as it was
(code_readability_and_cleanup Q5 RC-02, the TASK-005 verification gate).

Each test builds a throwaway Git checkout in a temporary folder, runs the tool as a separate
process exactly as a task would, and compares every file's bytes before and after. Global and
system Git configuration are switched off so the results do not depend on the machine.
"""
from __future__ import annotations

import hashlib
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

TOOL = Path(__file__).resolve().parents[2] / 'scripts' / 'test' / 'task_cleanup.py'
ENV = {**os.environ, 'GIT_CONFIG_GLOBAL': os.devnull, 'GIT_CONFIG_NOSYSTEM': '1'}


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


class TaskCleanupTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        base = Path(self.tmp.name)
        self.repo = base / 'checkout'
        self.outside = base / 'outside'
        self.manifest = base / 'artifacts.json'
        self.repo.mkdir()
        self.outside.mkdir()
        subprocess.run(['git', 'init', '-q'], cwd=self.repo, env=ENV, check=True)
        self.write('README.md', 'tracked project file\n')
        subprocess.run(['git', 'add', 'README.md'], cwd=self.repo, env=ENV, check=True)
        # Pre-existing user work: untracked, never recorded by the task.
        self.write('notes.txt', 'the user\'s own notes\n')
        self.write('drafts/idea.md', '# An idea the user has not committed yet\n')
        (self.outside / 'victim.txt').write_text('outside the checkout\n')
        # The task's own temporary artifacts, recorded as it creates them.
        self.write('tmp/out.bin', 'build output\n')
        self.write('tmp/sub/log.txt', 'debug log\n')
        self.write('scratch.txt', 'task scratch\n')
        self.run_tool('record', str(self.manifest), 'TASK-099', 'tmp/out.bin', 'tmp/sub/log.txt', 'tmp/sub/', 'tmp/',
                      'scratch.txt', expect=0)
        self.user_files = ['README.md', 'notes.txt', 'drafts/idea.md']
        self.before = {p: digest(self.repo / p) for p in self.user_files}
        self.victim = digest(self.outside / 'victim.txt')

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def write(self, relative: str, text: str) -> None:
        path = self.repo / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)

    def run_tool(self, *args: str, expect: int) -> subprocess.CompletedProcess:
        result = subprocess.run([sys.executable, '-B', str(TOOL), *args], cwd=self.repo, env=ENV,
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, expect, result.stdout + result.stderr)
        return result

    def write_manifest(self, artifacts: list, **overrides) -> None:
        manifest = {'schema': 'nexees-task-artifacts', 'version': 1, 'task': 'TASK-099', 'artifacts': artifacts}
        manifest.update(overrides)
        self.manifest.write_text(json.dumps(manifest))

    def assert_user_work_untouched(self) -> None:
        self.assertEqual({p: digest(self.repo / p) for p in self.user_files}, self.before)
        self.assertEqual(digest(self.outside / 'victim.txt'), self.victim)

    def test_apply_removes_the_recorded_artifacts_and_keeps_user_work(self) -> None:
        """Every recorded, unchanged artifact goes, including the folders they emptied; the
        user's untracked files, the tracked file and anything outside stay byte-for-byte."""
        record = json.loads(self.run_tool('clean', str(self.manifest), '--apply', expect=0).stdout)
        for gone in ('tmp/out.bin', 'tmp/sub/log.txt', 'scratch.txt', 'tmp/sub', 'tmp'):
            self.assertFalse((self.repo / gone).exists(), gone)
        self.assertEqual(sorted(record['removed']),
                         sorted(['tmp/out.bin', 'tmp/sub/log.txt', 'scratch.txt', 'tmp/sub/', 'tmp/']))
        self.assert_user_work_untouched()
        preserved = {item['path']: item['sha256'] for item in record['preserved_untracked']}
        self.assertEqual(preserved, {path: self.before[path] for path in ('notes.txt', 'drafts/idea.md')})

    def test_a_plan_changes_nothing(self) -> None:
        """Without --apply the tool only reports what it would remove."""
        record = json.loads(self.run_tool('clean', str(self.manifest), expect=0).stdout)
        self.assertFalse(record['applied'])
        self.assertIn('tmp/', record['removed'])
        for kept in ('tmp/out.bin', 'tmp/sub/log.txt', 'scratch.txt'):
            self.assertTrue((self.repo / kept).exists(), kept)
        self.assert_user_work_untouched()

    def test_an_artifact_changed_after_recording_is_kept(self) -> None:
        """A recorded file whose content changed may now hold someone's work, so it stays."""
        self.write('scratch.txt', 'the user started using this file\n')
        record = json.loads(self.run_tool('clean', str(self.manifest), '--apply', expect=1).stdout)
        self.assertEqual(record['refused'], [{'path': 'scratch.txt', 'reason': 'changed since it was recorded'}])
        self.assertEqual((self.repo / 'scratch.txt').read_text(), 'the user started using this file\n')
        self.assert_user_work_untouched()

    def test_a_folder_holding_unrecorded_files_is_kept(self) -> None:
        """A recorded folder is removed only when empty; it is never removed with its contents."""
        self.write('tmp/user-kept.txt', 'put here by the user\n')
        record = json.loads(self.run_tool('clean', str(self.manifest), '--apply', expect=1).stdout)
        self.assertEqual([d['path'] for d in record['kept_directories']], ['tmp/'])
        self.assertEqual((self.repo / 'tmp/user-kept.txt').read_text(), 'put here by the user\n')
        self.assert_user_work_untouched()

    def test_a_tracked_file_is_never_removed(self) -> None:
        """Even listed with its exact hash, a file Git tracks is refused."""
        self.write_manifest([{'path': 'README.md', 'sha256': self.before['README.md']}])
        record = json.loads(self.run_tool('clean', str(self.manifest), '--apply', expect=1).stdout)
        self.assertEqual(record['refused'], [{'path': 'README.md', 'reason': 'tracked by Git'}])
        self.assert_user_work_untouched()

    def test_nothing_is_reached_through_a_symbolic_link(self) -> None:
        """A link inside the checkout cannot lead the cleanup outside it, as a folder or as the file."""
        os.symlink(self.outside, self.repo / 'linked-folder')
        os.symlink(self.outside / 'victim.txt', self.repo / 'linked-file.txt')
        self.write_manifest([{'path': 'linked-folder/victim.txt', 'sha256': self.victim},
                             {'path': 'linked-file.txt', 'sha256': self.victim}])
        record = json.loads(self.run_tool('clean', str(self.manifest), '--apply', expect=1).stdout)
        self.assertEqual(sorted(r['path'] for r in record['refused']), ['linked-file.txt', 'linked-folder/victim.txt'])
        self.assertTrue((self.repo / 'linked-file.txt').is_symlink())
        self.assert_user_work_untouched()

    def test_an_invalid_manifest_is_rejected_before_anything_is_deleted(self) -> None:
        """The manifest authorizes deletion, so any doubt about it stops the whole run."""
        good = {'path': 'tmp/out.bin', 'sha256': digest(self.repo / 'tmp/out.bin')}
        cases = {
            'an unknown field': ([good], {'note': 'extra'}),
            'another schema version': ([good], {'version': 2}),
            'a boolean version': ([good], {'version': True}),
            'another schema': ([good], {'schema': 'something-else'}),
            'a malformed task': ([good], {'task': 'TASK-5'}),
            'a parent segment': ([{'path': '../outside/victim.txt', 'sha256': self.victim}, good], {}),
            'an absolute path': ([{'path': str(self.outside / 'victim.txt'), 'sha256': self.victim}, good], {}),
            'a dot segment': ([{'path': 'tmp/./out.bin', 'sha256': good['sha256']}], {}),
            'a duplicate': ([good, good], {}),
            'an unknown artifact field': ([{**good, 'force': True}], {}),
            'an uppercase hash': ([{'path': good['path'], 'sha256': good['sha256'].upper()}], {}),
            'a folder without its slash': ([{'path': 'tmp', 'directory': True}], {}),
        }
        for name, (artifacts, overrides) in cases.items():
            with self.subTest(name):
                self.write_manifest(artifacts, **overrides)
                result = self.run_tool('clean', str(self.manifest), '--apply', expect=2)
                self.assertIn('refused, nothing changed', result.stderr)
                self.assertTrue((self.repo / 'tmp/out.bin').exists())
                self.assert_user_work_untouched()

    def test_recording_refuses_tracked_files_and_links(self) -> None:
        """A tracked file or a link can never be recorded as a task artifact."""
        os.symlink(self.outside / 'victim.txt', self.repo / 'linked-file.txt')
        for path in ('README.md', 'linked-file.txt'):
            with self.subTest(path):
                self.run_tool('record', str(self.manifest), 'TASK-099', path, expect=2)


if __name__ == '__main__':
    unittest.main()
