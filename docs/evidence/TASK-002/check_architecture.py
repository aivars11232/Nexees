#!/usr/bin/env python3
"""Cross-check the architecture against the repository layout and the pack.

Evidence tooling for TASK-002, not product code. It reads the architecture's
LCL files, the repository tree and the specification pack as plain text and
JSON, and checks facts the LCL engine cannot see, because the engine never
reads outside the architecture project. It is not an LCL evaluator: the
project's own structure is checked by `lcl check`, `lcl validate` and
`lcl run` of docs/architecture/architecture.lcl.txt.

Rerun it whenever the layout, the architecture or the pack changes.

Usage: check_architecture.py <pack directory>
Exit code 0 when every check passes, 1 otherwise. Nothing is written.
"""
from __future__ import annotations
import hashlib
import json
import re
import sys
import zipfile
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
ARCH = REPO / 'docs' / 'architecture'

# Product source roots whose every file must belong to exactly one subsystem slot.
PRODUCT_ROOTS = ['apps', 'core', 'platform', 'integrations', 'assets', 'config', 'packaging', 'scripts', 'docs/manuals']

# Layers allowed under each root (decision AD-01): the core is shared, platform code is adapters,
# and apps hold only clients and the host composition roots.
ROOT_LAYERS = {'core/': {'shared_core'}, 'platform/': {'platform_adapter'}, 'apps/': {'client_ui', 'local_host'}}

# Requirement documents the architecture refines. Each must still be byte-identical to the
# pack revision 0.5.2 that the TASK-001 charter froze, so that the charter's baseline applies.
REFINED = ['policies/project_charter.lcl.txt', 'policies/master_rules.lcl.txt', 'policies/global_contracts.lcl.txt',
           'bindings/bindings.lcl.txt', 'policies/reuse_policy.lcl.txt', 'policies/no_unnecessary_code.lcl.txt',
           'policies/security_baseline.lcl.txt', 'checks/stop_conditions.lcl.txt',
           'architecture/remote_device_control.lcl.txt', 'verification/remote_control_acceptance.lcl.txt',
           'architecture/android_operating_model.lcl.txt', 'architecture/android_ui.lcl.txt',
           'architecture/authority_hierarchy.lcl.txt', 'architecture/auth_and_settings.lcl.txt',
           'architecture/desktop_ui.lcl.txt', 'architecture/lcl_import_pipeline.lcl.txt',
           'architecture/lcl_integration.lcl.txt', 'architecture/manuals_and_help.lcl.txt',
           'architecture/model_handoff.lcl.txt', 'architecture/persistent_state_model.lcl.txt',
           'architecture/provider_and_agent_model.lcl.txt', 'architecture/system_architecture.lcl.txt',
           'architecture/workspace_model.lcl.txt', 'reuse/reuse_matrix.lcl.txt',
           'reuse/import_handoff_sources.lcl.txt', 'templates/decision_log_template.lcl.txt']
BASELINE_ZIP = 'history/task_001_baseline_v0.5.2.zip'
BASELINE_PREFIX = 'Nexees_LCL_Implementation_Pack_v0.5.2/'

STATE_VOCABULARY = {'authority_class': {'authoritative', 'derived', 'release_content'},
                    'sync_policy': {'never', 'selected_content', 'portable_record', 'explicit_preference', 'view_only'},
                    'sensitivity': {'secret', 'security_critical', 'private', 'ordinary'}}


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def stem(relative: str) -> str:
    """Path without extension, so that a slot survives the real file extensions TASK-005 chooses."""
    folder, _, name = relative.rpartition('/')
    return f'{folder}/{name.split(".")[0]}'


def records(part: str, key: str) -> dict[str, dict]:
    """The typed OBJECT records of one architecture part that carry the identifier field `key`."""
    text = (ARCH / part).read_text(encoding='utf-8')
    found = {}
    pattern = r'^    ID: data\.arch_\w+\n    TYPE: OBJECT\[REF\(type\.\w+\)\]\n    VALUE:\n((?:        .*\n)+)'
    for block in re.findall(pattern, text, flags=re.M):
        fields = {m.group(1): json.loads(m.group(2)) for m in re.finditer(r'^        (\w+): (.*)$', block, flags=re.M)}
        if key in fields:
            found[fields[key]] = fields
    return found


def data_value(part: str, data_id: str):
    """Literal VALUE of one DATA or DEFINE declaration, with typed constructors unwrapped."""
    text = (ARCH / part).read_text(encoding='utf-8')
    value = re.search(rf'^    ID: {re.escape(data_id)}\n(?:    .*\n)*?    VALUE: (.*)$', text, flags=re.M).group(1)
    wrapped = re.fullmatch(r'(?:PATH|URI)\((".*")\)', value)
    return json.loads(wrapped.group(1) if wrapped else value)


def cycles(graph: dict[str, list[str]]) -> list[str]:
    """Each dependency cycle found by depth-first search, as 'A -> B -> A'."""
    found, done, active = [], set(), []

    def visit(node: str) -> None:
        if node in active:
            found.append(' -> '.join(active[active.index(node):] + [node]))
            return
        if node in done:
            return
        active.append(node)
        for nxt in graph.get(node, []):
            visit(nxt)
        active.pop()
        done.add(node)

    for node in graph:
        visit(node)
    return found


def main(pack: Path) -> int:
    failures: list[str] = []

    def check(name: str, ok: bool, detail: str = '') -> None:
        print(f'{"PASS" if ok else "FAIL"}  {name}' + (f'  [{detail}]' if detail else ''))
        if not ok:
            failures.append(name)

    subsystems = records('subsystems.lcl.txt', 'subsystem_id')
    states = records('state.lcl.txt', 'state_id')
    interfaces = records('interfaces.lcl.txt', 'interface_id')
    hosts = records('flows.lcl.txt', 'host_id')
    flows = records('flows.lcl.txt', 'flow_id')
    domains = records('failure_domains.lcl.txt', 'domain_id')
    contracts = records('remote_control.lcl.txt', 'requirement_id')
    decisions = records('decisions.lcl.txt', 'decision_id')
    reviews = records('reviews.lcl.txt', 'review_id')
    gaps = data_value('decisions.lcl.txt', 'data.arch_open_gaps')
    tasks = {f'TASK-{i:03}' for i in range(1, 76)}

    # 1. The architecture names exactly the pack on disk, and that pack is intact.
    manifest = json.loads((pack / 'MANIFEST.json').read_text(encoding='utf-8'))
    damaged = [f['path'] for f in manifest['files'] if sha256((pack / f['path']).read_bytes()) != f['sha256']]
    check('pack files match the pack manifest', not damaged, f'{len(manifest["files"])} files')
    ledger = json.loads((pack / 'CONTENT_IDENTITY.json').read_text(encoding='utf-8'))
    rehashed = [{'path': e['path'], 'sha256': sha256((pack / e['path']).read_bytes())} for e in ledger['active_lcl_sources']]
    identity = sha256(json.dumps(rehashed, sort_keys=True, separators=(',', ':')).encode('utf-8'))
    check('content identity recomputed from the pack files', identity == data_value('baseline.lcl.txt', 'binding.arch_pack_content_identity'), identity)
    check('manifest hash', sha256((pack / 'MANIFEST.json').read_bytes()) == data_value('baseline.lcl.txt', 'binding.arch_pack_manifest_sha256'))
    check('archive revision', manifest['version'] == data_value('baseline.lcl.txt', 'binding.arch_pack_archive_revision'), manifest['version'])
    check('specification version and language core',
          manifest['project_specification_version'] == data_value('baseline.lcl.txt', 'binding.arch_pack_specification_version')
          and manifest['language_core'] == data_value('baseline.lcl.txt', 'binding.arch_language_core'))

    # 2. The refined requirement documents are unchanged since the charter's baseline.
    with zipfile.ZipFile(pack / BASELINE_ZIP) as baseline:
        changed = [p for p in REFINED if sha256((pack / p).read_bytes()) != sha256(baseline.read(BASELINE_PREFIX + p))]
    check('refined documents are byte-identical to the charter baseline 0.5.2', not changed,
          f'{len(REFINED)} documents' if not changed else ', '.join(changed))

    # 3. Every product file belongs to exactly one slot, and every slot names real files.
    files = sorted(str(p.relative_to(REPO)) for root in PRODUCT_ROOTS for p in (REPO / root).rglob('*') if p.is_file())
    owners: dict[str, list[str]] = {f: [] for f in files}
    empty_slots, wrong_layer = [], []
    for sid, record in subsystems.items():
        for slot in record['slots']:
            path = slot.split(' | ', 1)[0]
            hits = [f for f in files if (f.startswith(path) if path.endswith('/') else stem(f) == path)]
            if not hits or ' | ' not in slot:
                empty_slots.append(f'{sid}: {path}')
            for f in hits:
                owners[f].append(sid)
                allowed = next((layers for root, layers in ROOT_LAYERS.items() if f.startswith(root)), None)
                if allowed and record['layer'] not in allowed:
                    wrong_layer.append(f'{f} in {sid}')
    unowned = [f'{f}: {v}' for f, v in owners.items() if len(v) != 1]
    check('every product file belongs to exactly one subsystem', not unowned, f'{len(files)} files' if not unowned else '; '.join(unowned))
    check('every slot names existing files and states a responsibility', not empty_slots, '; '.join(empty_slots))
    check('slot layers match their roots (AD-01)', not wrong_layer, '; '.join(wrong_layer))

    # 4. Every planned test file is attributed to exactly one subsystem, and none is invented.
    tests = sorted(stem(str(p.relative_to(REPO))) for p in (REPO / 'tests').rglob('*') if p.is_file())
    attributed = sorted(t for record in subsystems.values() for t in record['test_slots'])
    check('every test file is attributed to exactly one subsystem', attributed == tests, f'{len(tests)} test files')
    untested = sorted(sid for sid, record in subsystems.items() if not record['test_slots'])
    check('a subsystem without test files is a recorded gap', all(any(sid in g for g in gaps) for sid in untested), ', '.join(untested))

    # 5. Subsystem dependencies resolve and form an acyclic graph (AD-12).
    unknown = [f'{sid} -> {d}' for sid, r in subsystems.items() for d in r['depends_on'] if d not in subsystems or d == sid]
    check('dependencies name other catalogued subsystems', not unknown, '; '.join(unknown))
    loops = cycles({sid: r['depends_on'] for sid, r in subsystems.items()})
    check('the dependency graph is acyclic (AD-12)', not loops, '; '.join(loops))

    # 6. Every state category has exactly one owner, and each owner lists exactly what it owns.
    claimed = {st: [sid for sid, r in subsystems.items() if st in r['owns_state']] for st in states}
    mismatched = [f'{st}: owner {r["owner"]}, listed by {claimed[st]}' for st, r in states.items() if claimed[st] != [r['owner']]]
    phantom = [f'{sid}: {st}' for sid, r in subsystems.items() for st in r['owns_state'] if st not in states]
    check('every state category has exactly one owning subsystem', not mismatched and not phantom, '; '.join(mismatched + phantom))
    off = [f'{st}.{k}={r[k]}' for st, r in states.items() for k, allowed in STATE_VOCABULARY.items() if r[k] not in allowed]
    check('state classifications use the documented vocabulary', not off, '; '.join(off))
    check('secret state never synchronizes', all(r['sync_policy'] == 'never' for r in states.values() if r['sensitivity'] == 'secret'))

    # 7. Interfaces, hosts and flows are consistent with the catalogue.
    bad = [f'{i}: {s}' for i, r in interfaces.items() for s in r['providers'] + r['consumers'] if s not in subsystems]
    bad += [f'{i}: {c}' for i, r in interfaces.items() for c in r['consumers'] if c in subsystems
            and c not in r['providers'] and not any(p in subsystems[c]['depends_on'] for p in r['providers'])]
    check('interface consumers are subsystems that depend on a provider', not bad, '; '.join(bad))
    shared = {sid for sid, r in subsystems.items() if r['layer'] == 'shared_core'} - {'SS-HELP'}
    expected = {'desktop': shared | {'SS-LOCAL-HOST', 'SS-PLATFORM-DESKTOP'}, 'android': shared | {'SS-LOCAL-HOST', 'SS-PLATFORM-ANDROID'}}
    check('both hosts compose the same shared core and differ only in platform adapters',
          len(hosts) == 2 and all(set(h['runs']) == expected[h['device']] and len(h['runs']) == len(set(h['runs'])) for h in hosts.values()))
    steps = [f'{f}: {s}' for f, r in flows.items() for s in r['steps']
             if ' | ' not in s or any(x not in subsystems for x in s.split(' | ')[0].split(', '))]
    check('every flow step names the subsystems that perform it', not steps, '; '.join(steps))

    # 8. Cross-references resolve, task references are real, and the plan is covered.
    known = set(subsystems) | set(states) | set(interfaces) | set(hosts) | set(flows) | set(domains) | set(decisions)
    dangling = [f'{c}: {e}' for c, r in contracts.items() for e in r['elements'] if e not in known]
    dangling += [f'{rv}: {e}' for rv, r in reviews.items() for e in r['references'] if e not in known]
    check('contract elements and review references resolve', not dangling, '; '.join(dangling))
    referenced = [t for r in subsystems.values() for t in r['owner_tasks']] + [t for r in states.values() for t in r['schema_tasks']] \
        + [t for r in interfaces.values() for t in r['schema_tasks']] + [t for r in domains.values() for t in r['owner_tasks']] \
        + [t for r in decisions.values() for t in r.get('decide_by', [])] + [t for r in contracts.values() for t in r['runtime_owners']]
    check('every task reference is one of TASK-001 to TASK-075', set(referenced) <= tasks, ', '.join(sorted(set(referenced) - tasks)))
    affected = [f'{d}: {s}' for d, r in decisions.items() for s in r.get('affects', []) if s not in subsystems]
    check('open decisions affect catalogued subsystems', not affected, '; '.join(affected))
    owned = {t for r in subsystems.values() for t in r['owner_tasks']}
    missing = sorted({f'TASK-{i:03}' for i in range(5, 76)} - owned)
    check('every implementation task from TASK-005 to TASK-075 owns part of a subsystem', not missing, ', '.join(missing))
    slot_paths = [slot.split(' | ', 1)[0] for r in subsystems.values() for slot in r['slots']]
    singles = [s.split(' | ')[1] for s in data_value('subsystems.lcl.txt', 'data.arch_single_paths')]
    stray = [s for s in singles if not any(p == s or (s.endswith('/') and p.startswith(s)) or (p.endswith('/') and s.startswith(p)) for p in slot_paths)]
    check('each single path is a catalogued slot', not stray, '; '.join(stray))

    # 9. The remote contracts equal the pack's assignment, runtime owners and scenario range.
    requirement_map = json.loads((pack / 'TASK_REQUIREMENT_MAP.json').read_text(encoding='utf-8'))['tasks']
    assigned = next(t for t in requirement_map if t['task_id'] == 'TASK-002')['remote_requirements']
    check('the contracts cover exactly the requirements the pack assigns to TASK-002',
          list(contracts) == assigned == data_value('remote_control.lcl.txt', 'data.arch_assigned_remote_requirements'), ' '.join(assigned))
    runtime: dict[str, list[str]] = {}
    for task in requirement_map:
        for requirement in task['remote_requirements']:
            runtime.setdefault(requirement, []).append(task['task_id'])
    wrong = [r for r, c in contracts.items() if c['runtime_owners'] != [t for t in runtime[r] if t not in ('TASK-001', 'TASK-002')]]
    check('runtime owners equal the pack requirement map after TASK-002', not wrong, ', '.join(wrong))
    gap_ids = {g.split(' | ')[0] for g in gaps}
    plan = [f'{r}: {e}' for r, c in contracts.items() for e in c['test_plan']
            if not (re.fullmatch(r'RC-T(0[1-9]|1[0-8])', e.split(' | ')[0]) or e.split(' | ')[0] in gap_ids or e.split(' | ')[0] in tests)]
    check('test plans name RC-T01 to RC-T18, planned test files or recorded gaps', not plan, '; '.join(plan))

    print(f'\n{"ALL CHECKS PASSED" if not failures else "FAILED: " + ", ".join(failures)}')
    return 1 if failures else 0


if __name__ == '__main__':
    if len(sys.argv) != 2 or not Path(sys.argv[1]).is_dir():
        raise SystemExit(__doc__)
    raise SystemExit(main(Path(sys.argv[1])))
