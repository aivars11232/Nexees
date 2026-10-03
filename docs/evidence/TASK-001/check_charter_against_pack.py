#!/usr/bin/env python3
"""Cross-check the frozen charter against the specification pack it freezes.

Evidence tooling for TASK-001, not product code. It reads the charter's LCL
files and the pack as plain text and JSON, and compares facts the LCL engine
cannot see, because the engine never reads outside the charter project. It is
not an LCL evaluator: the charter's own structure is checked by `lcl check`,
`lcl validate` and `lcl run`.

Usage: check_charter_against_pack.py <pack directory>
Exit code 0 when every check passes, 1 otherwise. Nothing is written.
"""
from __future__ import annotations
import hashlib
import json
import re
import sys
from pathlib import Path

CHARTER = Path(__file__).resolve().parents[2] / 'charter'

# Acceptance families: charter key -> (document in the pack, identifier prefix, digits, expected count).
FAMILIES = {
    'import': ('architecture/lcl_import_pipeline.lcl.txt', 'IM-', 2, 10),
    'handoff': ('architecture/model_handoff.lcl.txt', 'HO-', 2, 10),
    'manuals': ('architecture/manuals_and_help.lcl.txt', 'MAN-', 2, 5),
    'readability': ('policies/code_readability_and_cleanup.lcl.txt', 'RC-', 2, 5),
    'android': ('architecture/android_operating_model.lcl.txt', 'AN-', 2, 12),
    'remote_requirements': ('architecture/remote_device_control.lcl.txt', 'RC-', 2, 26),
    'remote_scenarios': ('verification/remote_control_acceptance.lcl.txt', 'RC-T', 2, 18),
}


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def charter_text(name: str) -> str:
    return (CHARTER / name).read_text(encoding='utf-8')


def constant(text: str, identifier: str) -> str:
    """Literal VALUE of one DEFINE or DATA declaration, with typed constructors unwrapped."""
    block = re.search(rf'^    ID: {re.escape(identifier)}\n(?:    .*\n)*?    VALUE: (.*)$', text, flags=re.M)
    value = block.group(1)
    wrapped = re.fullmatch(r'(?:PATH|URI)\((".*")\)', value)
    return json.loads(wrapped.group(1) if wrapped else value)


def object_field(text: str, identifier: str, field: str):
    """One field of an OBJECT declaration written in block form."""
    block = re.search(rf'^    ID: {re.escape(identifier)}\n(?:    .*\n)*?(?=\n|\Z)', text, flags=re.M).group(0)
    return json.loads(re.search(rf'^        {field}: (.*)$', block, flags=re.M).group(1))


def pack_document(pack: Path, relative: str) -> str:
    """Prose of a pack document that is carried as one STRING VALUE; rule files are returned raw."""
    raw = (pack / relative).read_text(encoding='utf-8')
    value = re.search(r'^    VALUE: (".*")$', raw, flags=re.M)
    return json.loads(value.group(1)) if value else raw


def main(pack: Path) -> int:
    failures: list[str] = []

    def check(name: str, ok: bool, detail: str = '') -> None:
        print(f'{"PASS" if ok else "FAIL"}  {name}' + (f'  [{detail}]' if detail else ''))
        if not ok:
            failures.append(name)

    baseline, scope = charter_text('baseline.lcl.txt'), charter_text('scope.lcl.txt')
    trace = charter_text('traceability.lcl.txt')

    # 1. The charter names exactly the pack that is on disk, and that pack is intact.
    manifest = json.loads((pack / 'MANIFEST.json').read_text(encoding='utf-8'))
    ledger = json.loads((pack / 'CONTENT_IDENTITY.json').read_text(encoding='utf-8'))
    damaged = [f['path'] for f in manifest['files'] if sha256(pack / f['path']) != f['sha256']]
    check('pack files match the pack manifest', not damaged, f'{len(manifest["files"])} files')
    rehashed = [{'path': e['path'], 'sha256': sha256(pack / e['path'])} for e in ledger['active_lcl_sources']]
    identity = hashlib.sha256(json.dumps(rehashed, sort_keys=True, separators=(',', ':')).encode('utf-8')).hexdigest()
    check('content identity recomputed from the pack files', identity == constant(baseline, 'binding.charter_pack_content_identity'), identity)
    check('manifest hash', sha256(pack / 'MANIFEST.json') == constant(baseline, 'binding.charter_pack_manifest_sha256'))
    check('archive revision', manifest['version'] == constant(baseline, 'binding.charter_pack_archive_revision'), manifest['version'])
    check('specification version', manifest['project_specification_version'] == constant(baseline, 'binding.charter_pack_specification_version'))
    check('language core', manifest['language_core'] == constant(baseline, 'binding.charter_language_core'))

    # 2. Exactly 75 tasks, grouped into the phases of the pack's build order.
    task_files = sorted((pack / 'tasks').glob('task_*.lcl.txt'))
    task_count = int(re.search(r'^    ID: binding\.charter_task_count\n(?:    .*\n)*?    VALUE: (\d+)$', baseline, flags=re.M).group(1))
    check('task count', len(task_files) == task_count == manifest['implementation_task_count'] == 75, str(len(task_files)))
    phases: list[list] = []
    for number, phase in re.findall(r'^(\d{3})\. \*\*TASK-\d{3} — .*?\*\* — (.+)$', pack_document(pack, 'guidance/build_order.lcl.txt'), flags=re.M):
        if phases and phases[-1][0] == phase:
            phases[-1][2] = int(number)
        else:
            phases.append([phase, int(number), int(number)])
    check('execution phases', constant(trace, 'data.charter_execution_phases') == [f'{a:03}-{b:03} | {n}' for n, a, b in phases], f'{len(phases)} phases')
    check('phase task counts', constant(trace, 'data.charter_phase_task_counts') == [b - a + 1 for _, a, b in phases])
    phase_names = {name for name, _, _ in phases}

    # 3. Task-ID map: every v0.3 task is either carried over from v0.2 or listed as added.
    migration = pack_document(pack, 'guidance/task_id_migration_v0.2_to_v0.3.lcl.txt')
    carried = re.findall(r'^\| TASK-\d{3} \| TASK-(\d{3}) \|', migration, flags=re.M)
    added = [f'TASK-{n}' for n in re.findall(r'^- TASK-(\d{3}):', migration, flags=re.M)]
    check('task-ID map carried over', len(carried) == object_field(trace, 'data.charter_task_id_map', 'tasks_carried_over_from_v0_2') == 70)
    check('task-ID map added tasks', added == object_field(trace, 'data.charter_task_id_map', 'tasks_added_in_v0_3'), ' '.join(added))
    check('task-ID map covers 001-075', sorted(carried + [a[5:] for a in added]) == [f'{i:03}' for i in range(1, 76)])

    # 4. Acceptance families exist in their documents with the stated number of identifiers.
    valid_tasks = {f'TASK-{i:03}' for i in range(1, 76)}
    for key, (document, prefix, digits, expected) in FAMILIES.items():
        found = sorted(set(re.findall(rf'(?<![A-Z-]){re.escape(prefix)}(\d{{{digits}}})(?!\d)', (pack / document).read_text(encoding='utf-8'))))
        identifier = f'data.charter_family_{key}'
        complete = found == [f'{i:02}' for i in range(1, expected + 1)]
        check(f'family {key}: identifiers in {document}', complete and object_field(trace, identifier, 'count') == expected, f'{len(found)}/{expected}')
        owners = object_field(trace, identifier, 'primary_owner_tasks') + object_field(trace, identifier, 'verification_tasks')
        check(f'family {key}: owner task identifiers', set(owners) <= valid_tasks and object_field(trace, identifier, 'authoritative_document').startswith(document))

    # 5. Remote ownership equals the pack's requirement map, turned from per-task to per-requirement.
    owners, scenario_owners = {}, {}
    for task in json.loads((pack / 'TASK_REQUIREMENT_MAP.json').read_text(encoding='utf-8'))['tasks']:
        for requirement in task['remote_requirements']:
            owners.setdefault(requirement, []).append(task['task_id'])
        for scenario in task['remote_scenarios']:
            scenario_owners.setdefault(scenario, []).append(task['task_id'])
    check('remote requirement owners', constant(trace, 'data.charter_remote_requirement_owners') == [f"{k} | {' '.join(owners[k])}" for k in sorted(owners)], f'{len(owners)} requirements')
    check('remote scenario owners', constant(trace, 'data.charter_remote_scenario_owners') == [f"{k} | {' '.join(scenario_owners[k])}" for k in sorted(scenario_owners)], f'{len(scenario_owners)} scenarios')
    check('TASK-001 owns all 26 remote requirements and no scenario', all('TASK-001' in v for v in owners.values()) and not any('TASK-001' in v for v in scenario_owners.values()))

    # 6. Every clause cites pack documents that exist and phases that exist.
    clauses = re.findall(r'^    ID: (data\.charter_clause_\d\d)$', scope, flags=re.M)
    missing, unknown = [], []
    for clause in clauses:
        for entry in object_field(scope, clause, 'authority'):
            if not (pack / entry.split(' ', 1)[0]).is_file():
                missing.append(f'{clause}: {entry}')
        unknown += [f'{clause}: {p}' for p in object_field(scope, clause, 'primary_phases') if p not in phase_names]
    check('clause authorities exist in the pack', not missing, f'{len(clauses)} clauses' if not missing else '; '.join(missing))
    check('clause phases exist in the build order', not unknown, '; '.join(unknown))
    covered = {p for clause in clauses for p in object_field(scope, clause, 'primary_phases')}
    check('every phase is covered by a clause', covered == phase_names, ', '.join(sorted(phase_names - covered)))

    print(f'\n{"ALL CHECKS PASSED" if not failures else "FAILED: " + ", ".join(failures)}')
    return 1 if failures else 0


if __name__ == '__main__':
    if len(sys.argv) != 2 or not Path(sys.argv[1]).is_dir():
        raise SystemExit(__doc__)
    raise SystemExit(main(Path(sys.argv[1])))
