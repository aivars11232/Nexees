#!/usr/bin/env python3
"""Cross-check the threat model against the architecture, the repository and the pack.

Evidence tooling for TASK-004, not product code. It reads docs/security, docs/architecture,
docs/dependencies, the repository tree and the specification pack as plain text and JSON, and
checks what the LCL engine cannot see from inside docs/security: that every threat is traced to
owning tasks and to a runtime-enforced boundary that exists in the architecture, that every
reference resolves, and that the remote contracts match the pack's requirement map. The
project's own structure is checked by `lcl check`, `lcl validate` and `lcl run` of
docs/security/security.lcl.txt.

Rerun it whenever the threat model, the architecture, the test layout or the pack changes.

Usage: check_threat_model.py <pack directory>
Exit code 0 when every check passes, 1 otherwise. Nothing is written.
"""
from __future__ import annotations
import hashlib
import json
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
SEC = REPO / 'docs' / 'security'
ARCH = REPO / 'docs' / 'architecture'
DEP = REPO / 'docs' / 'dependencies'
FACTS = Path(__file__).resolve().parent / 'logs' / 'foundation_security_facts.txt'
FIRST_OWNER = 5  # every owner of later work is TASK-005 or later
PLATFORMS = {'desktop', 'android'}
LOCATIONS = {'desktop', 'android', 'service'}
NAMED_SOURCES = {'authority_hierarchy', 'auth_and_settings', 'manuals_and_help', 'provider_and_agent_model'}
# Facts of the TASK-003 foundation that TH-02, TH-03, TH-04 and TH-10 rely on, as recorded in the evidence log.
FOUNDATION_FACTS = ['contextIsolation: true', 'sandbox: false', 'nodeIntegration: false', 'if (!this.electronSecurityToken) {',
                    'timingSafeEqual', '0 RunAsNode: enabled', '2 EnableNodeOptionsEnvironmentVariable: enabled',
                    '3 EnableNodeCliInspectArguments: enabled', '4 EnableEmbeddedAsarIntegrityValidation: disabled',
                    'android:allowBackup="false"', 'PendingIntent.FLAG_IMMUTABLE']


def parse_value(text: str):
    """One LCL literal of a record field: JSON strings and lists, integers, TRUE and FALSE; anything else as written."""
    if text in ('TRUE', 'FALSE'):
        return text == 'TRUE'
    try:
        return json.loads(text)
    except json.JSONDecodeError:
        return text


def records(folder: Path, part: str, key: str) -> dict[str, dict]:
    """The typed OBJECT records of one project part that carry the identifier field `key`, in file order."""
    text = (folder / part).read_text(encoding='utf-8')
    found = {}
    pattern = r'^    ID: data\.\w+\n    TYPE: OBJECT\[REF\(type\.\w+\)\]\n    VALUE:\n((?:        .*\n)+)'
    for block in re.findall(pattern, text, flags=re.M):
        fields = {m.group(1): parse_value(m.group(2)) for m in re.finditer(r'^        (\w+): (.*)$', block, flags=re.M)}
        if key in fields:
            found[fields[key]] = fields
    return found


def data_value(folder: Path, part: str, data_id: str):
    """Literal VALUE of one DATA or DEFINE declaration."""
    text = (folder / part).read_text(encoding='utf-8')
    return parse_value(re.search(rf'^    ID: {re.escape(data_id)}\n(?:    .*\n)*?    VALUE: (.*)$', text, flags=re.M).group(1))


def pack_text(pack: Path, relative: str, block_id: str, field: str = 'DESCRIPTION') -> str:
    """The unescaped text field of one declaration of a pack document."""
    text = (pack / relative).read_text(encoding='utf-8')
    return json.loads(re.search(rf'^    ID: {re.escape(block_id)}\n(?:    .*\n)*?    {field}: (".*")$', text, flags=re.M).group(1))


def bullet_items(text: str) -> list[str]:
    """The '- item;' lines of a pack rule, without the dash and the final punctuation."""
    return [line[2:].rstrip(';.').strip() for line in text.splitlines() if line.startswith('- ')]


def task_number(task: str) -> int:
    return int(task.split('-')[1])


def sequential(ids: list[str], prefix: str, width: int = 2) -> bool:
    return ids == [f'{prefix}{n:0{width}d}' for n in range(1, len(ids) + 1)]


def slot_exists(point: str) -> bool:
    """A folder slot exists as a directory. A module slot exists as its placeholder or, once a task has
    implemented it, as source under the same stem (docs/engineering/CONVENTIONS.md section 2). TASK-007, which
    implemented the first enforcement points, repaired this check, which had counted only the placeholder."""
    path = REPO / point
    if point.endswith('/'):
        return path.is_dir()
    return any(f.is_file() and f.name.split('.')[0] == path.name for f in path.parent.glob(f'{path.name}.*'))


def main(pack: Path) -> int:
    failures: list[str] = []

    def check(name: str, ok: bool, detail: str = '') -> None:
        print(f'{"PASS" if ok else "FAIL"}  {name}' + (f'  [{detail}]' if detail else ''))
        if not ok:
            failures.append(name)

    # Architecture facts the threat model refers to.
    subsystems = records(ARCH, 'subsystems.lcl.txt', 'subsystem_id')
    interfaces = records(ARCH, 'interfaces.lcl.txt', 'interface_id')
    states = records(ARCH, 'state.lcl.txt', 'state_id')
    arch_contracts = records(ARCH, 'remote_control.lcl.txt', 'requirement_id')
    decisions = records(ARCH, 'decisions.lcl.txt', 'decision_id')
    arch_gaps = {g.split(' | ')[0] for g in data_value(ARCH, 'decisions.lcl.txt', 'data.arch_open_gaps')}
    arch_assumptions = {a.split(' ')[0] for a in data_value(ARCH, 'decisions.lcl.txt', 'data.arch_security_assumptions')}
    extension_limits = {e.split(' ')[0] for e in data_value(ARCH, 'subsystems.lcl.txt', 'data.arch_extension_limits')}
    open_items = {o.split(' | ')[0] for o in data_value(DEP, 'strategy.lcl.txt', 'data.dep_open_items')}
    strategy = {d.split(' | ')[0] for d in data_value(DEP, 'strategy.lcl.txt', 'data.dep_strategy')}
    slots = {sid: [slot.split(' | ', 1)[0] for slot in r['slots']] for sid, r in subsystems.items()}
    owners = {sid: set(r['owner_tasks']) for sid, r in subsystems.items()}
    providers = {iid: r['providers'] for iid, r in interfaces.items()}
    tests_on_disk = {str(p.relative_to(REPO))[:-len('.test.source')] for p in (REPO / 'tests').rglob('*.test.source')}

    # The threat model.
    levels = records(SEC, 'trust.lcl.txt', 'level')
    authorities = records(SEC, 'trust.lcl.txt', 'authority_id')
    boundaries = records(SEC, 'trust.lcl.txt', 'boundary_id')
    assets = records(SEC, 'assets.lcl.txt', 'asset_id')
    surfaces = records(SEC, 'assets.lcl.txt', 'surface_id')
    invariants = records(SEC, 'invariants.lcl.txt', 'invariant_id')
    threats = records(SEC, 'threats.lcl.txt', 'threat_id')
    threat_tests = records(SEC, 'tests.lcl.txt', 'test_id')
    contracts = records(SEC, 'remote.lcl.txt', 'requirement_id')
    gaps = data_value(SEC, 'tests.lcl.txt', 'data.sec_test_gaps')
    sec_gaps = {g.split(' | ')[0]: g for g in gaps}
    residuals = [r.split(' | ')[0] for r in data_value(SEC, 'trust.lcl.txt', 'data.sec_residual_risks')]
    assumptions = [a.split(' | ')[0] for a in data_value(SEC, 'trust.lcl.txt', 'data.sec_assumptions')]
    open_decisions = data_value(SEC, 'trust.lcl.txt', 'data.sec_open_decisions')
    areas = data_value(SEC, 'baseline.lcl.txt', 'binding.sec_areas')
    v03 = data_value(SEC, 'baseline.lcl.txt', 'binding.sec_v03_categories')
    categories = data_value(SEC, 'baseline.lcl.txt', 'binding.sec_categories')
    execution_checks = [e.split(' | ')[0] for e in data_value(SEC, 'remote.lcl.txt', 'data.sec_execution_checks')]
    baseline_text = (SEC / 'baseline.lcl.txt').read_text(encoding='utf-8')
    message_types = set(re.findall(r'^    ID: (type\.sec_\w+)$', baseline_text, flags=re.M))

    # Scenario and rule identifiers defined by the pack.
    scenarios = ({f'RC-T{n:02d}' for n in range(1, 19)} | {f'IM-{n:02d}' for n in range(1, 11)} |
                 {f'HO-{n:02d}' for n in range(1, 11)} | {f'MAN-{n:02d}' for n in range(1, 6)} |
                 {f'AN-{n:02d}' for n in range(1, 13)} | {f'Q5-RC-{n:02d}' for n in range(1, 6)})
    pack_scenarios_text = ''.join((pack / p).read_text(encoding='utf-8') for p in (
        'verification/remote_control_acceptance.lcl.txt', 'architecture/lcl_import_pipeline.lcl.txt',
        'architecture/model_handoff.lcl.txt', 'architecture/manuals_and_help.lcl.txt',
        'architecture/android_operating_model.lcl.txt', 'policies/code_readability_and_cleanup.lcl.txt'))
    missing_in_pack = sorted(s for s in scenarios if s.replace('Q5-', '') not in pack_scenarios_text)

    def test_ref_ok(ref: str) -> bool:
        return ref in tests_on_disk or ref in scenarios or ref in arch_gaps or ref in sec_gaps

    def boundary_owners(enforced: list[str]) -> set[str]:
        found = set()
        for element in enforced:
            for sid in [element] if element in subsystems else providers.get(element, []):
                found |= owners[sid]
        return found

    def boundary_slots(enforced: list[str]) -> set[str]:
        found = set()
        for element in enforced:
            for sid in [element] if element in subsystems else providers.get(element, []):
                found |= set(slots[sid])
        return found

    def in_slots(point: str, enforced: list[str]) -> bool:
        """A module slot of the boundary, or a module inside one of its folder slots."""
        return any(point == slot or (slot.endswith('/') and point.startswith(slot)) for slot in boundary_slots(enforced))

    def known_element(element: str) -> bool:
        return element in subsystems or element in interfaces

    # 1. The pack and the assignment.
    manifest = json.loads((pack / 'MANIFEST.json').read_text(encoding='utf-8'))
    damaged = [f['path'] for f in manifest['files'] if hashlib.sha256((pack / f['path']).read_bytes()).hexdigest() != f['sha256']]
    check('pack files match the pack manifest', not damaged, f'{len(manifest["files"])} files')
    check('every scenario family the model cites exists in the pack', not missing_in_pack, ', '.join(missing_in_pack))
    requirement_map = json.loads((pack / 'TASK_REQUIREMENT_MAP.json').read_text(encoding='utf-8'))['tasks']
    mine = next(t for t in requirement_map if t['task_id'] == 'TASK-004')
    check('the contracts cover exactly the requirements the pack assigns to TASK-004',
          list(contracts) == mine['remote_requirements'] == data_value(SEC, 'remote.lcl.txt', 'data.sec_assigned_remote_requirements'),
          ' '.join(mine['remote_requirements']))
    check('the assigned scenarios equal the pack map',
          mine['remote_scenarios'] == data_value(SEC, 'remote.lcl.txt', 'data.sec_assigned_remote_scenarios'), ' '.join(mine['remote_scenarios']))
    later = lambda key, field: [t['task_id'] for t in requirement_map if key in t[field] and task_number(t['task_id']) > 4]
    wrong = [r for r, c in contracts.items() if c['runtime_owners'] != later(r, 'remote_requirements')]
    check('runtime owners equal the pack requirement map after TASK-004', not wrong, ', '.join(wrong))
    plans = data_value(SEC, 'remote.lcl.txt', 'data.sec_scenario_plans')
    bad_plans = []
    for plan in plans:
        scenario, test, proved, _ = plan.split(' | ')
        if re.findall(r'TASK-\d{3}', proved) != later(scenario, 'remote_scenarios') or test not in threat_tests or scenario not in threat_tests[test]['scenarios']:
            bad_plans.append(scenario)
    check('each assigned scenario has a threat test and the runtime owners of the pack map',
          not bad_plans and [p.split(' | ')[0] for p in plans] == mine['remote_scenarios'], ', '.join(bad_plans))
    bad_arch = [r for r, c in contracts.items()
                if (c['architecture'] != 'none' and (r not in arch_contracts or c['architecture'] != f'data.arch_rc_{r[3:]}'))
                or (c['architecture'] == 'none' and r in arch_contracts)]
    check('a requirement TASK-002 also designed builds on its contract, and only then', not bad_arch, ', '.join(bad_arch))
    bad_types = [f'{r}: {t}' for r, c in contracts.items() for t in c['types'] if t not in message_types]
    unused_types = sorted({'type.sec_request_envelope', 'type.sec_grant', 'type.sec_approval', 'type.sec_audit_record',
                           'type.sec_receiver_settings', 'type.sec_lock_policy'} - {t for c in contracts.values() for t in c['types']})
    check('contract types exist and every message type serves a contract', not bad_types and not unused_types, '; '.join(bad_types + unused_types))
    bad_plan_refs = [f'{r}: {e}' for r, c in contracts.items() for e in c['test_plan'] if not test_ref_ok(e.split(' | ')[0])]
    check('contract test plans name scenarios, test slots or recorded gaps', not bad_plan_refs, '; '.join(bad_plan_refs))
    check('the destination checks are EC-01 to EC-14 in order', sequential(execution_checks, 'EC-'), f'{len(execution_checks)} checks')

    # 2. The v0.3 gate: each threat traced to owning tasks and a runtime-enforced boundary.
    early = [f'{t}: {o}' for t, r in threats.items() for o in r['owner_tasks'] if not FIRST_OWNER <= task_number(o) <= 75]
    check('every threat has owning tasks, all from TASK-005 to TASK-075', not early and all(r['owner_tasks'] for r in threats.values()), '; '.join(early))
    unknown = [f'{t}: {e}' for t, r in threats.items() for e in r['enforced_by'] if not known_element(e)]
    check('every threat boundary names architecture subsystems and interfaces', not unknown, '; '.join(unknown))
    stray = [f'{t}: {p}' for t, r in threats.items() for p in r['enforcement_points'] if not in_slots(p, r['enforced_by'])]
    check('every enforcement point is a module slot of the threat\'s boundary', not stray, '; '.join(stray))
    unowned = [f'{t}: {o}' for t, r in threats.items() for o in r['owner_tasks'] if o not in boundary_owners(r['enforced_by'])]
    check('every owning task owns a subsystem of the threat\'s boundary', not unowned, '; '.join(unowned))
    bad_tests = [f'{t}: {x}' for t, r in threats.items() for x in r['tests'] if not test_ref_ok(x)]
    check('every threat test reference is a test slot, a pack scenario or a recorded gap', not bad_tests, '; '.join(bad_tests))
    bad_refs = [f'{t}: {x}' for t, r in threats.items() for x in r['invariants'] + r['assets'] + r['surfaces']
                if x not in invariants and x not in assets and x not in surfaces]
    bad_refs += [f'{t}: {a}' for t, r in threats.items() for a in r['areas'] if a not in areas]
    bad_refs += [f'{t}: {r["category"]}' for t, r in threats.items() if r['category'] not in categories]
    check('threat invariants, assets, surfaces, areas and categories resolve', not bad_refs, '; '.join(bad_refs))

    # 3. Coverage of the task scope.
    per_area = {a: sum(a in r['areas'] for r in threats.values()) for a in areas}
    check('every area of the task scope has at least three threats', min(per_area.values()) >= 3, ', '.join(f'{a} {n}' for a, n in per_area.items()))
    per_category = {c: (sum(r['category'] == c for r in threats.values()), sum(r['category'] == c for r in threat_tests.values())) for c in v03}
    uncovered = [t for t, r in threats.items() if r['category'] in v03
                 and not any(t in x['threats'] and x['category'] == r['category'] for x in threat_tests.values())]
    check('every v0.3 category has threats and at least two threat tests, covering each of its threats',
          all(n >= 1 and m >= 2 for n, m in per_category.values()) and not uncovered,
          '; '.join(f'{c} {n}/{m}' for c, (n, m) in per_category.items()) + ('; uncovered ' + ', '.join(uncovered) if uncovered else ''))
    unused = sorted(set(invariants) - {i for r in threats.values() for i in r['invariants']})
    check('every invariant answers at least one threat', not unused, ', '.join(unused))
    unused = sorted(set(assets) - {a for r in threats.values() for a in r['assets']}) + \
        sorted(set(surfaces) - {s for r in threats.values() for s in r['surfaces']}) + \
        sorted(set(boundaries) - {r['boundary'] for r in surfaces.values()})
    check('every asset and surface is threatened, and every boundary has a surface', not unused, ', '.join(unused))
    covered_states = {s for r in assets.values() for s in r['states']}
    check('the assets cover every state category of the architecture',
          covered_states == set(states), ', '.join(sorted(set(states) ^ covered_states)))
    controls = bullet_items(pack_text(pack, 'policies/security_baseline.lcl.txt', 'rule.policies_security_baseline_03'))
    coverage = [c.split(' | ') for c in data_value(SEC, 'invariants.lcl.txt', 'data.sec_control_coverage')]
    bad = [name for name, refs in coverage if not refs or any(i not in invariants for i in refs.split(', '))]
    check('every required control of the security baseline maps to invariants',
          [name for name, _ in coverage] == controls and not bad, f'{len(controls)} controls' + ('; ' + ', '.join(bad) if bad else ''))
    stops = bullet_items(pack_text(pack, 'checks/stop_conditions.lcl.txt', 'failure.stop_04'))
    coverage = [c.split(' | ') for c in data_value(SEC, 'invariants.lcl.txt', 'data.sec_stop_coverage')]
    bad = [name for name, refs in coverage if not refs or any(i not in invariants for i in refs.split(', '))]
    check('every security stop condition maps to invariants',
          [name for name, _ in coverage] == stops and not bad, f'{len(stops)} conditions' + ('; ' + ', '.join(bad) if bad else ''))
    hierarchy = pack_text(pack, 'architecture/authority_hierarchy.lcl.txt', 'document.architecture_authority_hierarchy', 'VALUE')
    pack_levels = re.findall(r'^(\d)\. (.+)$', hierarchy, flags=re.M)
    check('the trust levels are the pack\'s eight levels, in order',
          [(str(level), r['name']) for level, r in sorted(levels.items())] == pack_levels and sorted(levels) == list(range(1, 9)),
          ' > '.join(r['name'] for _, r in sorted(levels.items())))

    # 4. Invariants, boundaries, surfaces and authorities.
    bad = [f'{i}: {e}' for i, r in invariants.items() for e in r['enforced_by'] if not known_element(e)]
    bad += [f'{i}: {p}' for i, r in invariants.items() for p in r['enforcement_points'] if not in_slots(p, r['enforced_by'])]
    check('every invariant names architecture elements and their module slots', not bad, '; '.join(bad))
    runtime_by_source = lambda r: {t for s in r['sources'] if re.fullmatch(r'RC-\d\d', s) for t in later(s, 'remote_requirements')}
    bad = [f'{i}: {o}' for i, r in invariants.items() for o in r['owner_tasks']
           if not FIRST_OWNER <= task_number(o) <= 75 or o not in boundary_owners(r['enforced_by']) | runtime_by_source(r)]
    check('every invariant owner is a later task that owns its boundary or its remote requirement', not bad, '; '.join(bad))
    bad = [f'{i}: {x}' for i, r in invariants.items() for x in r['tests'] if not test_ref_ok(x)]
    check('every invariant test reference resolves', not bad, '; '.join(bad))
    catalog =(pack / 'data/task_catalog.lcl.txt').read_text(encoding='utf-8')
    policy_ids = set(json.loads(re.search(r'ID: data\.expected_policy_ids\n(?:    .*\n)*?    VALUE: (\[.*\])', catalog).group(1)))

    def source_ok(s: str) -> bool:
        patterns = [(r'RC-(\d\d)', lambda m: 1 <= int(m[1]) <= 26), (r'C(\d+)', lambda m: 1 <= int(m[1]) <= 31),
                    (r'STOP-(\d\d)', lambda m: 1 <= int(m[1]) <= 12), (r'B(\d+)', lambda m: 1 <= int(m[1]) <= 26),
                    (r'R(\d+)', lambda m: 1 <= int(m[1]) <= 31), (r'A(\d+)', lambda m: 1 <= int(m[1]) <= 10),
                    (r'I(\d)', lambda m: 1 <= int(m[1]) <= 6), (r'H(\d)', lambda m: 1 <= int(m[1]) <= 5),
                    (r'Q(\d)', lambda m: 1 <= int(m[1]) <= 5)]
        if s in policy_ids or s in NAMED_SOURCES or s in scenarios or s in arch_gaps or s in arch_assumptions:
            return True
        if s in extension_limits or s in decisions or s in strategy:
            return True
        return any((m := re.fullmatch(p, s)) and ok(m) for p, ok in patterns)
    bad = sorted({s for r in invariants.values() for s in r['sources'] if not source_ok(s)})
    check('every invariant source is a pack policy, rule, contract, scenario or architecture decision', not bad, ', '.join(bad))
    bad = [f'{b}: {x}' for b, r in boundaries.items() for x in r['invariants'] if x not in invariants]
    bad += [f'{b}: {e}' for b, r in boundaries.items() for e in r['enforced_by'] if not known_element(e)]
    bad += [f'{b}: {p}' for b, r in boundaries.items() for p in r['enforcement_points'] if not in_slots(p, r['enforced_by'])]
    check('every boundary names invariants, architecture elements and their module slots', not bad, '; '.join(bad))
    bad = [f'{s}: {a}' for s, r in surfaces.items() for a in r['areas'] if a not in areas]
    bad += [f'{s}: {e}' for s, r in surfaces.items() for e in r['elements'] if not known_element(e)]
    bad += [f'{s}: {r["boundary"]}' for s, r in surfaces.items() if r['boundary'] not in boundaries]
    check('every surface has valid areas, a boundary and architecture elements', not bad, '; '.join(bad))
    bad = [f'{a}: {x}' for a, r in assets.items() for x in r['states'] + r['invariants'] if x not in states and x not in invariants]
    bad += [f'{a}: {loc}' for a, r in assets.items() for loc in r['locations'] if loc not in LOCATIONS]
    bad += [f'{a}: {x}' for a, r in authorities.items() for x in r['states'] if x not in states]
    bad += [f'{a}: {o}' for a, r in authorities.items() for o in r['owner_tasks'] if not FIRST_OWNER <= task_number(o) <= 75]
    check('assets and authorities name existing state categories, invariants and later tasks', not bad, '; '.join(bad))
    missing_slots = sorted({p for r in list(threats.values()) + list(invariants.values()) + list(boundaries.values())
                            for p in r['enforcement_points'] if not slot_exists(p)})
    check('every enforcement point exists in the source layout', not missing_slots, ', '.join(missing_slots))

    # 5. Threat tests, gaps and text references.
    bad = [f'{t}: {x}' for t, r in threat_tests.items() for x in r['threats'] if x not in threats]
    bad += [f'{t}: {r["slot"]}' for t, r in threat_tests.items() if not test_ref_ok(r['slot'])]
    bad += [f'{t}: {x}' for t, r in threat_tests.items() for x in r['scenarios'] if x not in scenarios]
    bad += [f'{t}: {o}' for t, r in threat_tests.items() for o in r['owner_tasks'] if not FIRST_OWNER <= task_number(o) <= 75]
    bad += [f'{t}: {p}' for t, r in threat_tests.items() for p in r['platforms'] if p not in PLATFORMS]
    bad += [f'{t}: {r["category"]}' for t, r in threat_tests.items() if r['category'] not in categories]
    check('every threat test names threats, a slot or gap, pack scenarios, later owners and platforms', not bad, '; '.join(bad))
    every_ref = [x for r in threats.values() for x in r['tests']] + [x for r in invariants.values() for x in r['tests']] + \
        [r['slot'] for r in threat_tests.values()] + [e.split(' | ')[0] for c in contracts.values() for e in c['test_plan']]
    orphan = sorted(set(sec_gaps) - set(every_ref))
    gap_owners = [g for g in gaps if not re.findall(r'TASK-\d{3}', g) or any(not FIRST_OWNER <= task_number(t) <= 75 for t in re.findall(r'TASK-\d{3}', g))]
    gap_threats = [g for g in gaps if any(t not in threats for t in re.findall(r'TH-\d\d', g))]
    check('every recorded gap is needed, names later tasks and real threats', not orphan and not gap_owners and not gap_threats,
          '; '.join(orphan + gap_owners + gap_threats))
    texts = [json.dumps(r) for part in (threats, invariants, threat_tests, contracts, boundaries, surfaces, assets, authorities)
             for r in part.values()] + [json.dumps(x) for x in (gaps, open_decisions)] + \
        [json.dumps(data_value(SEC, 'trust.lcl.txt', name)) for name in ('data.sec_residual_risks', 'data.sec_assumptions', 'data.sec_trust_rules')]
    blob = ' '.join(texts)
    dangling = sorted({x for x in re.findall(r'\bTH-\d\d\b', blob) if x not in threats} |
                      {x for x in re.findall(r'\bSI-\d\d\b', blob) if x not in invariants} |
                      {x for x in re.findall(r'\bTT-\d\d\b', blob) if x not in threat_tests} |
                      {x for x in re.findall(r'\bR-\d\d\b', blob) if x not in residuals} |
                      {x for x in re.findall(r'\bSEC-A\d\d\b', blob) if x not in assumptions} |
                      {x for x in re.findall(r'\bSA-\d\d\b', blob) if x not in arch_assumptions} |
                      {x for x in re.findall(r'\bOI-\d\d\b', blob) if x not in open_items} |
                      {x for x in re.findall(r'\bEXT-\d\d\b', blob) if x not in extension_limits} |
                      {x for x in re.findall(r'\bGAP-\d\d\b', blob) if x not in arch_gaps} |
                      {x for x in re.findall(r'\bSG-\d\d\b', blob) if x not in sec_gaps} |
                      {x for x in re.findall(r'\bAU-\d\d\b', blob) if x not in authorities} |
                      {x for x in re.findall(r'\bEC-\d\d\b', blob) if x not in execution_checks} |
                      {x for x in re.findall(r'\bDS-\d\d\b', blob) if x not in strategy})
    check('identifiers mentioned in the texts resolve', not dangling, ', '.join(dangling))
    numbering = {'TH-': list(threats), 'SI-': list(invariants), 'TT-': list(threat_tests), 'SF-': list(surfaces),
                 'AS-': list(assets), 'TB-': list(boundaries), 'AU-': list(authorities), 'SG-': list(sec_gaps),
                 'R-': residuals, 'SEC-A': assumptions, 'OSD-': [d.split(' | ')[0] for d in open_decisions]}
    broken = [prefix for prefix, ids in numbering.items() if not sequential(ids, prefix)]
    check('identifiers are unique and numbered without gaps', not broken,
          ', '.join(f'{p} {len(ids)}' for p, ids in numbering.items()) if not broken else ', '.join(broken))
    every_owner = [o for part in (threats, invariants, threat_tests) for r in part.values() for o in r['owner_tasks']] + \
        [o for r in contracts.values() for o in r['runtime_owners']] + [o for r in authorities.values() for o in r['owner_tasks']] + \
        re.findall(r'TASK-\d{3}', json.dumps(gaps + open_decisions))
    check('no later work is assigned to TASK-004 or an earlier task', all(FIRST_OWNER <= task_number(o) <= 75 for o in every_owner),
          f'{len(every_owner)} assignments')
    facts = FACTS.read_text(encoding='utf-8') if FACTS.is_file() else ''
    missing = [f for f in FOUNDATION_FACTS if f not in facts]
    check('the foundation facts the threats cite are in the evidence log', not missing, ', '.join(missing) or FACTS.name)

    print(f'\n{len(threats)} threats, {len(invariants)} invariants, {len(threat_tests)} threat tests, {len(contracts)} remote contracts')
    print(f'{"ALL CHECKS PASSED" if not failures else "FAILED: " + ", ".join(failures)}')
    return 1 if failures else 0


if __name__ == '__main__':
    if len(sys.argv) != 2 or not Path(sys.argv[1]).is_dir():
        raise SystemExit(__doc__)
    raise SystemExit(main(Path(sys.argv[1])))
