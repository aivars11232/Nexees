#!/usr/bin/env python3
"""Cross-check the TASK-003 dependency inventory against the facts it cites.

Evidence tooling for TASK-003, not product code. The inventory's own structure
is checked by `lcl check`, `lcl validate` and `lcl run` of
docs/dependencies/dependencies.lcl.txt; this script checks what the engine
cannot see: the lockfiles and manifests of the prototypes, the recorded
registry and advisory data, the human-readable matrix, the architecture's
open-decision resolutions and the evidence files the records name.

Usage: check_dependencies.py
Exit code 0 when every check passes, 1 otherwise. Nothing is written and no
network is used.
"""
from __future__ import annotations
import json
import re
import sys
import tomllib
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
DEPS = REPO / 'docs' / 'dependencies'
ARCH = REPO / 'docs' / 'architecture'
EVIDENCE = REPO / 'docs' / 'evidence' / 'TASK-003'
PROTO = EVIDENCE / 'prototypes'
REGISTRY = Path('/mnt/F/Nexees-toolchains/cargo/registry/src')

FIELDS = ['component_id', 'name', 'area', 'classification', 'scope', 'upstream', 'version', 'license', 'obligations',
          'redistribution', 'trademark', 'advisories', 'security_impact', 'maintenance', 'transitive', 'platforms', 'need',
          'alternatives', 'reason', 'replacement', 'verification', 'evidence', 'owner_tasks']

# Shared-core crates and the inventory record that pins each.
CRATES = {'rusqlite': 'DEP-SQLITE', 'zip': 'DEP-ZIP', 'boa_engine': 'DEP-BOA', 'pulldown-cmark': 'DEP-PULLDOWN-CMARK',
          'rustls': 'DEP-RUSTLS', 'ring': 'DEP-RUSTLS', 'rcgen': 'DEP-RCGEN', 'serde': 'DEP-SERDE',
          'serde_json': 'DEP-SERDE', 'nix': 'DEP-NIX', 'jni': 'DEP-JNI'}

# Registry research entries whose license the inventory must repeat.
REGISTRY_LICENSES = {('crates', 'rusqlite'): 'DEP-SQLITE', ('crates', 'zip'): 'DEP-ZIP',
                     ('crates', 'pulldown-cmark'): 'DEP-PULLDOWN-CMARK', ('crates', 'rustls'): 'DEP-RUSTLS',
                     ('crates', 'rcgen'): 'DEP-RCGEN', ('crates', 'jni'): 'DEP-JNI', ('crates', 'uniffi'): 'DEP-UNIFFI',
                     ('crates', 'tree-sitter'): 'DEP-TREE-SITTER', ('crates', 'gix'): 'DEP-GIX',
                     ('crates', 'keyring'): 'DEP-KEYRING', ('crates', 'boa_engine'): 'DEP-BOA',
                     ('crates', 'rquickjs'): 'DEP-RQUICKJS', ('crates', 'mlua'): 'DEP-MLUA', ('crates', 'wasmi'): 'DEP-WASMI',
                     ('crates', 'cargo-ndk'): 'DEP-CARGO-NDK', ('crates', 'ammonia'): 'DEP-AMMONIA', ('crates', 'nix'): 'DEP-NIX',
                     ('npm', '@theia/core'): 'DEP-THEIA', ('npm', 'electron'): 'DEP-ELECTRON', ('npm', 'ovsx'): 'DEP-OPEN-VSX',
                     ('npm', '@agentclientprotocol/sdk'): 'DEP-ACP', ('npm', '@supabase/auth-js'): 'DEP-SUPABASE-AUTH'}

# npm packages inherited through Theia and the record that names each.
INHERITED = {'electron': 'DEP-ELECTRON', '@theia/monaco-editor-core': 'DEP-MONACO', 'xterm': 'DEP-XTERM',
             'node-pty': 'DEP-NODE-PTY', 'keytar': 'DEP-KEYTAR', 'vscode-languageserver-protocol': 'DEP-LSP'}


def records() -> dict[str, dict]:
    text = (DEPS / 'components.lcl.txt').read_text(encoding='utf-8')
    pattern = r'^    ID: data\.dep_\w+\n    TYPE: OBJECT\[REF\(type\.dep_component\)\]\n    VALUE:\n((?:        .*\n)+)'
    found = {}
    for block in re.findall(pattern, text, flags=re.M):
        fields = {m.group(1): json.loads(m.group(2)) for m in re.finditer(r'^        (\w+): (.*)$', block, flags=re.M)}
        found[fields['component_id']] = fields
    return found


def data_value(path: Path, data_id: str):
    text = path.read_text(encoding='utf-8')
    return json.loads(re.search(rf'^    ID: {re.escape(data_id)}\n(?:    .*\n)*?    VALUE: (.*)$', text, flags=re.M).group(1))


def record_literal(name: str) -> int:
    text = (DEPS / 'record.lcl.txt').read_text(encoding='utf-8')
    return int(re.search(rf'^        {name}: (\d+)$', text, flags=re.M).group(1))


def normalise(license_: str) -> str:
    return license_.replace('/', ' OR ').strip()


def main() -> int:
    failures: list[str] = []

    def check(name: str, ok: bool, detail: str = '') -> None:
        print(f'{"PASS" if ok else "FAIL"}  {name}' + (f'  [{detail}]' if detail else ''))
        if not ok:
            failures.append(name)

    inventory = records()
    classes = data_value(DEPS / 'baseline.lcl.txt', 'binding.dep_classifications')
    scopes = data_value(DEPS / 'baseline.lcl.txt', 'binding.dep_scopes')
    tasks = {f'TASK-{i:03}' for i in range(1, 76)}

    # 1. Every record answers every gate field.
    check('49 records with unique DEP- identifiers',
          len(inventory) == 49 and all(re.fullmatch(r'DEP-[A-Z0-9-]+', i) for i in inventory), str(len(inventory)))
    incomplete = [f'{i}: {f}' for i, r in inventory.items() for f in FIELDS if f not in r or r[f] in ('', [])]
    check('every record fills every gate and template field', not incomplete, '; '.join(incomplete))
    check('classifications and scopes come from the declared vocabularies',
          all(r['classification'] in classes and r['scope'] in scopes for r in inventory.values()))
    bad_tasks = sorted({t for r in inventory.values() for t in r['owner_tasks'] if t not in tasks})
    check('owner tasks are TASK-001 to TASK-075', not bad_tasks, ', '.join(bad_tasks))
    counts = {c: sum(r['classification'] == c for r in inventory.values()) for c in classes}
    stated = {c: record_literal(f'{c.lower()}_count') for c in classes}
    check('the record states the real classification counts', counts == stated, json.dumps(counts))
    missing = sorted({e for r in inventory.values() for e in r['evidence'] if not (REPO / e).exists()})
    check('every cited evidence file exists', not missing, ', '.join(missing))
    rejected_in_use = [i for i, r in inventory.items() if r['classification'] == 'REJECTED' and r['scope'] not in ('none', 'desktop')]
    check('no rejected component is used by the shared core, Android or a build', not rejected_in_use, ', '.join(rejected_in_use))

    # 2. Rust pins: Cargo.toml, Cargo.lock, the registry manifests and the records agree.
    manifest = tomllib.loads((PROTO / 'feasibility-core' / 'Cargo.toml').read_text())
    declared = dict(manifest['dependencies'])
    for target in manifest.get('target', {}).values():
        declared.update(target.get('dependencies', {}))
    lock = {p['name']: p for p in tomllib.loads((PROTO / 'feasibility-core' / 'Cargo.lock').read_text())['package']}
    mismatched = []
    for crate, dep_id in CRATES.items():
        pinned = declared[crate]['version'].lstrip('=')
        locked = lock[crate]['version']
        registry = next(REGISTRY.glob(f'*/{crate}-{locked}/Cargo.toml'), None)
        license_ = tomllib.loads(registry.read_text())['package'].get('license') if registry else None
        record = inventory[dep_id]
        if pinned != locked or locked not in record['version'] or not license_ or license_ not in record['license']:
            mismatched.append(f'{crate} pinned {pinned} locked {locked} license {license_} vs {dep_id}')
    check('Rust pins, lockfile, registry licenses and records agree', not mismatched, '; '.join(mismatched))
    unpinned = [name for name, spec in declared.items() if 'path' not in spec and not str(spec.get('version', '')).startswith('=')]
    check('every registry dependency of the prototype is pinned exactly', not unpinned, ', '.join(unpinned))

    # 3. npm: the evaluated Theia manifest, the overrides and the inherited versions agree with the records.
    theia = json.loads((PROTO / 'theia' / 'package.json').read_text())
    theia_versions = {v for k, v in theia['dependencies'].items() if k.startswith('@theia/')} | {theia['devDependencies']['@theia/cli']}
    check('all @theia packages are the recorded 1.76.0', theia_versions == {'1.76.0'} and '1.76.0' in inventory['DEP-THEIA']['version'])
    overrides = {line.split(' | ')[0]: line.split(' | ')[1].split('-> ')[-1].strip()
                 for line in data_value(DEPS / 'strategy.lcl.txt', 'data.dep_npm_overrides')}
    check('the npm overrides equal data.dep_npm_overrides', overrides == theia['overrides'], json.dumps(theia['overrides']))
    after = json.loads((EVIDENCE / 'logs' / 'osv_after_overrides.json').read_text())
    npm = {(p['name'], p['version']): p['license'] for p in after['packages'] if p['ecosystem'] == 'npm'}
    wrong = []
    for package, dep_id in INHERITED.items():
        versions = sorted(v for (n, v) in npm if n == package)
        if len(versions) != 1 or versions[0] not in inventory[dep_id]['version'] \
                or npm[(package, versions[0])] not in inventory[dep_id]['license']:
            wrong.append(f'{package} {versions} vs {dep_id}')
    check('inherited npm packages match their records in version and license', not wrong, '; '.join(wrong))
    check('the evaluated tree is the recorded 888 npm and 216 crate packages', after['queried'] == {'npm': 888, 'crates.io': 216},
          json.dumps(after['queried']))
    lockfile = json.loads((PROTO / 'theia' / 'package-lock.json').read_text())
    resolved = {(e.get('name') or path.rsplit('node_modules/', 1)[-1], e['version'])
                for path, e in lockfile['packages'].items() if path and 'version' in e}
    check('the kept lockfile resolves exactly the npm packages the advisory lookup covered', resolved == set(npm), str(len(resolved)))

    # 4. Advisories: every remaining finding has a disposition, and the Rust tree has none.
    dispositions = data_value(DEPS / 'strategy.lcl.txt', 'data.dep_advisory_dispositions')
    disposed = {}
    for line in dispositions:
        package = line.split(' | ')[0]
        disposed[package] = set(re.findall(r'GHSA-[a-z0-9]{4}-[a-z0-9]{4}-[a-z0-9]{4}', line.split(' | ')[1]))
    found = {}
    for advisory in after['advisories']:
        name, version = advisory['package'].split(':', 1)[1].rsplit('@', 1)
        found.setdefault(f'{name} {version}', set()).add(advisory['id'])
    check('every remaining advisory has a recorded disposition with its identifiers', found == disposed,
          json.dumps({k: sorted(v) for k, v in found.items()}))
    check('the Rust tree has no advisory', not [a for a in after['advisories'] if a['package'].startswith('crates.io:')])
    before = json.loads((EVIDENCE / 'logs' / 'osv_before_overrides.json').read_text())
    check('the published configuration had the recorded 34 advisories in 10 packages',
          len(before['advisories']) == 34 and len({a['package'] for a in before['advisories']}) == 10)

    # 5. Registry research repeats in the records.
    research = json.loads((EVIDENCE / 'logs' / 'registry_research.json').read_text())
    differ = []
    for (kind, name), dep_id in REGISTRY_LICENSES.items():
        license_ = normalise(research[kind][name]['license'])
        if license_ not in normalise(inventory[dep_id]['license']):
            differ.append(f'{name} {license_} vs {dep_id}')
    check('registry licenses are repeated in the records', not differ, '; '.join(differ))

    # 6. The human-readable matrix lists the same components with the same class and scope.
    rows = re.findall(r'^\| (DEP-[A-Z0-9-]+) \|(?:[^|]*\|){3} (\w+) \| ([\w ]+) \|', (DEPS / 'LICENSE_MATRIX.md').read_text(), flags=re.M)
    matrix = {i: (c, s.strip()) for i, c, s in rows}
    check('the matrix lists every component once', len(rows) == len(matrix) == len(inventory) and set(matrix) == set(inventory))
    differ = [i for i, (c, s) in matrix.items() if i in inventory and (c, s) != (inventory[i]['classification'], inventory[i]['scope'])]
    check('the matrix repeats each classification and scope', not differ, ', '.join(differ))
    totals = f'Totals: {len(inventory)} components; ' + ', '.join(f'{counts[c]} {c}' for c in classes) + '.'
    check('the matrix totals equal the inventory', totals in (DEPS / 'LICENSE_MATRIX.md').read_text(), totals)

    # 7. The architecture records the resolutions TASK-003 owns.
    text = (ARCH / 'decisions.lcl.txt').read_text(encoding='utf-8')
    pds = {}
    for block in re.findall(r'^    TYPE: OBJECT\[REF\(type\.arch_open_decision\)\]\n    VALUE:\n((?:        .*\n)+)', text, flags=re.M):
        fields = {m.group(1): json.loads(m.group(2)) for m in re.finditer(r'^        (\w+): (.*)$', block, flags=re.M)}
        pds[fields['decision_id']] = fields
    owned = {d: r for d, r in pds.items() if 'TASK-003' in r['decide_by']}
    unanswered = [d for d, r in owned.items() if r['status'] == 'open' or not r['resolution'].startswith('TASK-003')]
    check('every open decision TASK-003 had to answer carries a TASK-003 resolution', len(owned) == 15 and not unanswered,
          ', '.join(unanswered))
    renderer = data_value(DEPS / 'baseline.lcl.txt', 'binding.dep_renderer_decision')
    check('the renderer resolution equals the owner decision in the inventory',
          pds['PD-RENDERER']['status'] == 'resolved' and renderer in pds['PD-RENDERER']['resolution'] and renderer == 'electron_permitted')
    check('the decision TASK-003 does not own stays open', pds['PD-PROVIDER-ANDROID']['status'] == 'open')

    print(f'\n{"ALL CHECKS PASSED" if not failures else "FAILED: " + ", ".join(failures)}')
    return 1 if failures else 0


if __name__ == '__main__':
    if len(sys.argv) != 1:
        raise SystemExit(__doc__)
    raise SystemExit(main())
