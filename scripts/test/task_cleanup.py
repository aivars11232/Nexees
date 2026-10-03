#!/usr/bin/env python3
"""Scoped task-close cleanup for the Nexees checkout.

A task records every temporary artifact it creates inside the checkout in a task-artifact
manifest, with the file's SHA-256 at the time it was recorded. At close, this tool removes
exactly those artifacts and nothing else (policies/code_readability_and_cleanup, Q3 and Q5
RC-02). It never removes:

- anything not listed in the manifest, so unknown and untracked user work stays put;
- a file Git tracks;
- a file whose content changed since it was recorded, which may now hold someone's work;
- anything reached through a symbolic link, or outside the checkout;
- a directory that is not empty (directories are removed one level at a time, never recursively).

The manifest authorizes deletions, so it is validated strictly before anything happens: an
unknown field, another schema version or an unsafe path rejects the whole manifest. Paths are
opened one component at a time without following links, so a component swapped for a link
between the check and the removal is refused rather than followed.

Without --apply the tool only reports its plan. Either way it writes a cleanup record: what
was removed, what was refused and why, and every untracked file it left in place, with its
SHA-256, as evidence of the user work it preserved.

Usage:
  task_cleanup.py record MANIFEST TASK-ID PATH...   add files (hashed now) or directories (PATH/)
  task_cleanup.py clean MANIFEST [--apply] [--record FILE]

Exit codes: 0 when every artifact was removed or already absent, 1 when an artifact was
refused or a directory kept, 2 when the manifest or the request is invalid.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import stat
import subprocess
import sys
from pathlib import Path

MANIFEST_SCHEMA = 'nexees-task-artifacts'
RECORD_SCHEMA = 'nexees-cleanup-record'
VERSION = 1
TASK_ID = re.compile(r'TASK-\d{3}')
SHA256 = re.compile(r'[0-9a-f]{64}')


class InvalidRequest(Exception):
    """A manifest or argument that must stop the run before anything is deleted."""


def checkout_root() -> Path:
    """The top of the Git worktree around the current directory."""
    result = subprocess.run(['git', 'rev-parse', '--show-toplevel'], capture_output=True, text=True)
    if result.returncode != 0:
        raise InvalidRequest('not inside a Git worktree')
    return Path(result.stdout.strip())


def git_paths(root: Path, *args: str) -> list[str]:
    """Paths Git lists for `git ls-files ARGS`, relative to the root."""
    out = subprocess.run(['git', '-C', str(root), 'ls-files', '-z', *args], capture_output=True, check=True).stdout
    return [p.decode('utf-8', 'surrogateescape') for p in out.split(b'\0') if p]


def check_path(path: object, directory: bool) -> str:
    """Return the path if it is a plain relative POSIX path inside the checkout, else refuse it.

    Directories end in '/', files do not. No absolute paths, backslashes, control characters,
    empty, '.' or '..' segments, so a path can only name what it literally says."""
    if not isinstance(path, str) or not path:
        raise InvalidRequest(f'path must be a non-empty string: {path!r}')
    body = path[:-1] if directory else path
    if directory != path.endswith('/') or not body:
        raise InvalidRequest(f'directories end in "/" and files do not: {path!r}')
    if body.startswith('/') or '\\' in path or any(ord(c) < 32 or ord(c) == 127 for c in path):
        raise InvalidRequest(f'not a plain relative path: {path!r}')
    if any(part in ('', '.', '..') for part in body.split('/')):
        raise InvalidRequest(f'empty, "." or ".." segment: {path!r}')
    return path


def load_manifest(file: Path) -> dict:
    """Read and strictly validate a task-artifact manifest (schema version 1)."""
    try:
        data = json.loads(file.read_text(encoding='utf-8'))
    except (OSError, ValueError) as error:
        raise InvalidRequest(f'cannot read the manifest: {error}') from error
    if not isinstance(data, dict) or set(data) != {'schema', 'version', 'task', 'artifacts'}:
        raise InvalidRequest('the manifest must have exactly: schema, version, task, artifacts')
    if data['schema'] != MANIFEST_SCHEMA or type(data['version']) is not int or data['version'] != VERSION:
        raise InvalidRequest(f'unsupported manifest: {data["schema"]!r} version {data["version"]!r}')
    if not isinstance(data['task'], str) or not TASK_ID.fullmatch(data['task']):
        raise InvalidRequest(f'task must look like TASK-005: {data["task"]!r}')
    if not isinstance(data['artifacts'], list):
        raise InvalidRequest('artifacts must be a list')
    seen = set()
    for artifact in data['artifacts']:
        if not isinstance(artifact, dict):
            raise InvalidRequest(f'an artifact must be an object: {artifact!r}')
        if set(artifact) == {'path', 'sha256'}:
            check_path(artifact['path'], directory=False)
            if not isinstance(artifact['sha256'], str) or not SHA256.fullmatch(artifact['sha256']):
                raise InvalidRequest(f'sha256 must be 64 lowercase hex digits: {artifact["path"]}')
        elif set(artifact) == {'path', 'directory'} and artifact['directory'] is True:
            check_path(artifact['path'], directory=True)
        else:
            raise InvalidRequest(f'an artifact has exactly path and sha256, or path and directory: true: {artifact!r}')
        if artifact['path'] in seen:
            raise InvalidRequest(f'listed twice: {artifact["path"]}')
        seen.add(artifact['path'])
    return data


def open_parent(root: Path, relative: str) -> tuple[int, str]:
    """Open the directory that holds RELATIVE, one component at a time without following links.

    Returns (directory fd, final name). A link or non-directory on the way raises OSError, so
    nothing outside the checkout or behind a link can be reached."""
    parts = relative.rstrip('/').split('/')
    flags = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC
    fd = os.open(root, flags)
    try:
        for part in parts[:-1]:
            next_fd = os.open(part, flags, dir_fd=fd)
            os.close(fd)
            fd = next_fd
    except OSError:
        os.close(fd)
        raise
    return fd, parts[-1]


def regular_file(dir_fd: int, name: str) -> tuple[str, os.stat_result] | None:
    """SHA-256 and identity of NAME if it is a regular file, opened without following a link.

    The type is checked on the opened descriptor, so a file swapped for a pipe, device or link
    after an earlier look is never read; O_NONBLOCK keeps a pipe from blocking the open."""
    fd = os.open(name, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK | os.O_CLOEXEC, dir_fd=dir_fd)
    with os.fdopen(fd, 'rb') as handle:
        info = os.fstat(handle.fileno())
        if not stat.S_ISREG(info.st_mode):
            return None
        digest = hashlib.sha256()
        for block in iter(lambda: handle.read(1 << 16), b''):
            digest.update(block)
    return digest.hexdigest(), info


def clean(root: Path, manifest: dict, apply: bool) -> dict:
    """Remove the manifest's artifacts that are still exactly what the task created.

    In a plan (apply False) nothing changes and `removed` lists what --apply would remove."""
    tracked = set(git_paths(root))
    removed, refused, absent, kept_directories = [], [], [], []

    for artifact in (a for a in manifest['artifacts'] if 'sha256' in a):
        path = artifact['path']
        if path in tracked:
            refused.append({'path': path, 'reason': 'tracked by Git'})
            continue
        try:
            dir_fd, name = open_parent(root, path)
        except FileNotFoundError:
            absent.append(path)
            continue
        except OSError:
            refused.append({'path': path, 'reason': 'a parent is a symbolic link or not a directory'})
            continue
        try:
            found = regular_file(dir_fd, name)
            if found is None:
                refused.append({'path': path, 'reason': 'not a regular file'})
            elif found[0] != artifact['sha256']:
                refused.append({'path': path, 'reason': 'changed since it was recorded'})
            else:
                if apply:
                    # The name must still lead to the file that was just hashed.
                    now = os.stat(name, dir_fd=dir_fd, follow_symlinks=False)
                    if (now.st_dev, now.st_ino) != (found[1].st_dev, found[1].st_ino):
                        refused.append({'path': path, 'reason': 'replaced while being checked'})
                        continue
                    os.unlink(name, dir_fd=dir_fd)
                removed.append(path)
        except FileNotFoundError:
            absent.append(path)
        except OSError:
            # O_NOFOLLOW refuses a symbolic link at the final component.
            refused.append({'path': path, 'reason': 'a symbolic link or unreadable'})
        finally:
            os.close(dir_fd)

    # Deepest directories first, so a directory emptied by its children can go too.
    directories = sorted((a['path'] for a in manifest['artifacts'] if 'directory' in a), key=lambda p: -p.count('/'))
    for path in directories:
        try:
            dir_fd, name = open_parent(root, path)
        except FileNotFoundError:
            absent.append(path)
            continue
        except OSError:
            refused.append({'path': path, 'reason': 'a parent is a symbolic link or not a directory'})
            continue
        try:
            child = os.open(name, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC, dir_fd=dir_fd)
            try:
                entries = os.listdir(child)
            finally:
                os.close(child)
            if not apply:
                # In a plan, what is planned for removal is still there.
                entries = [e for e in entries if f'{path}{e}' not in removed and f'{path}{e}/' not in removed]
            if entries:
                kept_directories.append({'path': path, 'reason': f'not empty ({len(entries)} entries)'})
                continue
            if apply:
                os.rmdir(name, dir_fd=dir_fd)
            removed.append(path)
        except FileNotFoundError:
            absent.append(path)
        except OSError:
            refused.append({'path': path, 'reason': 'a symbolic link or not a directory'})
        finally:
            os.close(dir_fd)

    # Everything untracked that stays is reported with its hash: the user work left in place.
    preserved = []
    for path in sorted(git_paths(root, '--others', '--exclude-standard')):
        if path in removed:
            continue
        digest = None
        try:
            dir_fd, name = open_parent(root, path)
            try:
                found = regular_file(dir_fd, name)
                digest = found[0] if found else None
            finally:
                os.close(dir_fd)
        except OSError:
            pass
        preserved.append({'path': path, 'sha256': digest})
    ignored = len(git_paths(root, '--others', '--ignored', '--exclude-standard'))
    return {
        'schema': RECORD_SCHEMA, 'version': VERSION, 'task': manifest['task'], 'applied': apply,
        'removed': removed, 'refused': refused, 'absent': absent, 'kept_directories': kept_directories,
        'preserved_untracked': preserved, 'ignored_left_in_place': ignored,
    }


def record(root: Path, manifest_file: Path, task: str, paths: list[str]) -> dict:
    """Add artifacts to a manifest, hashing files now. Refuses tracked files and links.

    The tool cannot know who created a path: by recording it, the task states that it did."""
    if manifest_file.exists():
        manifest = load_manifest(manifest_file)
        if manifest['task'] != task:
            raise InvalidRequest(f'the manifest belongs to {manifest["task"]}, not {task}')
    else:
        if not TASK_ID.fullmatch(task):
            raise InvalidRequest(f'task must look like TASK-005: {task!r}')
        manifest = {'schema': MANIFEST_SCHEMA, 'version': VERSION, 'task': task, 'artifacts': []}
    tracked = set(git_paths(root))
    listed = {a['path'] for a in manifest['artifacts']}
    for path in paths:
        directory = path.endswith('/')
        check_path(path, directory)
        if path in listed:
            continue
        if path in tracked:
            raise InvalidRequest(f'tracked by Git, so never a task artifact: {path}')
        try:
            dir_fd, name = open_parent(root, path)
        except OSError as error:
            raise InvalidRequest(f'cannot reach {path} without following a link: {error}') from error
        try:
            if directory:
                info = os.stat(name, dir_fd=dir_fd, follow_symlinks=False)
                if not stat.S_ISDIR(info.st_mode):
                    raise InvalidRequest(f'not a directory: {path}')
                manifest['artifacts'].append({'path': path, 'directory': True})
            else:
                found = regular_file(dir_fd, name)
                if found is None:
                    raise InvalidRequest(f'not a regular file: {path}')
                manifest['artifacts'].append({'path': path, 'sha256': found[0]})
        except OSError as error:
            raise InvalidRequest(f'cannot read {path} without following a link: {error}') from error
        finally:
            os.close(dir_fd)
        listed.add(path)
    manifest_file.write_text(json.dumps(manifest, indent=2) + '\n', encoding='utf-8')
    return manifest


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split('\n\n')[0])
    commands = parser.add_subparsers(dest='command', required=True)
    add = commands.add_parser('record', help='add artifacts to a task-artifact manifest')
    add.add_argument('manifest', type=Path)
    add.add_argument('task')
    add.add_argument('paths', nargs='+')
    run = commands.add_parser('clean', help='remove the recorded artifacts (a plan unless --apply)')
    run.add_argument('manifest', type=Path)
    run.add_argument('--apply', action='store_true', help='delete; without it nothing is changed')
    run.add_argument('--record', type=Path, help='write the cleanup record here as well as to stdout')
    args = parser.parse_args(argv)
    try:
        root = checkout_root()
        if args.command == 'record':
            record(root, args.manifest, args.task, args.paths)
            return 0
        result = clean(root, load_manifest(args.manifest), args.apply)
    except InvalidRequest as error:
        print(f'refused, nothing changed: {error}', file=sys.stderr)
        return 2
    text = json.dumps(result, indent=2) + '\n'
    sys.stdout.write(text)
    if args.record:
        args.record.write_text(text, encoding='utf-8')
    return 1 if result['refused'] or result['kept_directories'] else 0


if __name__ == '__main__':
    raise SystemExit(main(sys.argv[1:]))
