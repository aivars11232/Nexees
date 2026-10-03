# TASK-002 evidence: Define modular architecture and subsystem boundaries

| | |
|---|---|
| Task | TASK-002 — Define modular architecture and subsystem boundaries |
| Date | 2026-10-03 |
| Performed by | Coding agent (Claude Code), one primary session; no sub-agents, reviewers or background AI jobs |
| Checkout | `/mnt/F/Nexees/`, branch `main`, base `284f97a02d3ad9288f3701860e608b366b10979f` |
| Repository | `https://github.com/aivars11232/Nexees` |
| Specification pack | `/mnt/F/Nexees_LCL_Implementation_Pack_v0.5.3/`, archive revision 0.5.3, specification version 0.5.0 |
| Pack content identity | `bd5e8fce26f542c44a891e4340da3b2a84f79ab368b5cb868beb48126c2db3e1` |
| Pack manifest SHA-256 | `8d9d0080ac13a3fb17cc70603b77eb3b11abcd3fb6d2fad515cfac136fcf07e4` |
| Procedure | Continuation profile 0.5.3 (CA-01 to CA-12), adopted by the owner for this task |
| Predecessor | TASK-001, accepted, commit `284f97a` |
| Commit of this work | Made after acceptance, as CA-06 permits; it is recorded by Git, not in this file |

## Status

The deliverables are complete and every applicable check has passed. Acceptance is
recorded in [receipt/RECEIPT_RESULT.md](receipt/RECEIPT_RESULT.md). The receipt folder
is written after this file, because the receipt contains a snapshot of everything else.

Under CA-05 the routine corroboration of this evidence was done by the same primary
session's local tools. All review described here is the agent's own; no independent
human or model review has taken place.

## 1. Authority

The owner's instruction, verbatim:

```
Go to   /mnt/F/Nexees_LCL_Implementation_Pack_v0.5.3/     read everything and start with task2. and u'll work in /mnt/F/Nexees
```

The owner selected pack revision 0.5.3 and named TASK-002. Under CA-01 that adopts the
continuation profile `policies/task_002_075_continuation.lcl.txt` for this task. The
pack's adoption prompt `START_TASK_002.txt` (SHA-256 `7a0b838c…3409`) describes the
same bounded sequence: native verification once, then TASK-002 through verification,
receipt acceptance, commit and sync, then stop. The scope is TASK-002 only (CA-02). Not
covered: TASK-003, the Desktop renderer decision, any release, and enabling any
service, pairing or startup.

## 2. Checkout preflight (binding B26)

| Item | Observed |
|---|---|
| Working directory | `/mnt/F/Nexees/`; it is the Git worktree root, with no parent repository, superproject or second worktree |
| `origin` fetch and push | `https://github.com/aivars11232/Nexees.git`, an approved equivalent; no URL rewrite and no separate push URL |
| Branch and HEAD | `main` at `284f97a`, equal to the local `origin/main`; upstream `origin/main`; nothing was fetched |
| Hooks | None besides Git's samples; `core.hooksPath` unset |
| At task start, before the native verification began at 18:34 | 311 tracked files, nothing staged, modified or untracked, no stash |
| Recheck at 19:16 | Unchanged identity; only this task's files modified or added |
| Result | Passes |

The commands and their output are in [logs/preflight.txt](logs/preflight.txt).

## 3. Pack, native verification and predecessor

**Pack.** All 201 files match `MANIFEST.json` and the content identity recomputes to
`bd5e8fce…b3e1`. Against the 0.5.2 archive that TASK-001's charter froze, revision 0.5.3
changes none of the architecture, policy, reuse, verification, guidance, manual, template
or check documents, and adds one policy, the continuation profile. Each of its 74
changed task records differs only in the five declared
procedural fields, with its native TASK declaration unchanged; for TASK-002 that means the
objective, scope, deliverables and checks are exactly those of 0.5.2. Log:
[logs/preflight.txt](logs/preflight.txt).

**Native verification.** The pack's runner was run once, as its README_FIRST.txt asks:
`tools/verify_with_lcl.py --lcl lcl --core-01 /mnt/F/LCL/canonical/LCL_Core_0.1.0
--core-03 /mnt/F/LCL/canonical/LCL_Core_0.3.0`, with a fresh results folder in the
session's scratch directory instead of the suggested `/tmp` path. All 51 synthetic
language cases passed with `lcl 0.9.1` and the canonical Core 0.3.0 package (identity
`7c8d4693…1aff`). Its results are in
[logs/native_language_suite.json](logs/native_language_suite.json). These are synthetic
language tests, not evidence for any task.

**Dispatch.** `lcl run … --input input.task_number=2 main.lcl.txt` ran `task.dispatch`
and `task.task_002` only and published the TASK-002 packet, with
`procedure_amendment_applies: TRUE` (packet SHA-256 `4436e221…dd67`). Record:
[logs/dispatch_task_002.record.json](logs/dispatch_task_002.record.json). A successful
dispatch only means that the instructions were produced.

**Predecessor.** TASK-001 is accepted against pack 0.5.2 at commit `284f97a`. The
0.5.2 archive kept in the 0.5.3 pack has the manifest (`e55ec613…639a`) and content
identity (`e288da0e…4672`) that the TASK-001 receipt names; that receipt's engine record
(`bbda5d83…07e7`) ends in `status.succeeded`; `docs/charter/` and
`docs/evidence/TASK-001/` are unchanged since the commit. TASK-001 was not reopened and
its receipt was not reissued.

## 4. Required reading

The continuation profile and the task file were read first, then the whole mandatory
read order, then the rest of the pack. [logs/required_reading.txt](logs/required_reading.txt)
lists each file with its SHA-256 and says whether it equals the v0.5.2 file and how it
was read: most in full, a few large data files by summary with their hashes verified.

## 5. Deliverables

| File | Change | Why it is needed |
|---|---|---|
| `docs/architecture/architecture.lcl.txt` | New | Entry of the architecture project; lists its parts |
| `docs/architecture/baseline.lcl.txt` | New | The pack revision and repository state it refines; the record types |
| `docs/architecture/subsystems.lcl.txt` | New | 24 subsystems with their layer, devices, owned module slots, owned state, dependencies, prohibitions, owner tasks and test files; the 13 capabilities that exist once; the 7 extension limits |
| `docs/architecture/state.lcl.txt` | New | 22 state categories, each with its single owner, store, copies, sync policy and sensitivity |
| `docs/architecture/interfaces.lcl.txt` | New | 14 interfaces with their invariants and schema-owning tasks |
| `docs/architecture/flows.lcl.txt` | New | The Desktop and Android hosts, and 10 data and authority flows |
| `docs/architecture/failure_domains.lcl.txt` | New | 14 failure domains: reach, what stays protected, recovery |
| `docs/architecture/remote_control.lcl.txt` | New | Design contracts and test plans for the 11 remote-control requirements TASK-002 owns |
| `docs/architecture/decisions.lcl.txt` | New | 12 decisions, 16 open portability decisions, 8 security assumptions, 4 reuse observations, 7 gaps |
| `docs/architecture/reviews.lcl.txt` | New | The two gate reviews, T002.VERIFY.01 and T002.VERIFY.06 |
| `docs/architecture/checks.lcl.txt` | New | 10 `VALIDATE` and 12 `VERIFY` structural checks |
| `docs/architecture/record.lcl.txt` | New | Returns the architecture record |
| `README.md` | Changed | Names the architecture and its rule that every product file belongs to one subsystem |
| `FILE_TREE.txt` | Changed | Inventory of the new files |
| `docs/evidence/TASK-002/` | New | This record, one checking script, its logs and the receipt |

Removed files: none. New dependencies: none. No product code was written, and no
technology, dependency or renderer was selected. Every one of the 217 product files is
owned by exactly one subsystem and each of the 59 test files is attributed to one. No
placeholder file was edited, so the 278 marker lines are unchanged; README.md and
FILE_TREE.txt had already lost theirs in TASK-001.

## 6. Checks run

| Command | Result | Log |
|---|---|---|
| `lcl check`, `lcl validate` and `lcl run` of `docs/architecture/architecture.lcl.txt` | Exit 0 each; 10 `VALIDATE` hold; `status.succeeded` with all 12 `VERIFY` TRUE | [architecture_lcl.txt](logs/architecture_lcl.txt), [architecture_run.record.json](logs/architecture_run.record.json) |
| `python3 -B docs/evidence/TASK-002/check_architecture.py <pack>` | 27 of 27 pass, exit 0 | [check_architecture.txt](logs/check_architecture.txt) |
| Negative tests: 14 seeded defects in scratch copies of the worktree | All 14 detected by the intended check | [negative_tests.txt](logs/negative_tests.txt) |
| Regression: the charter's `lcl` checks, and TASK-001's cross-check against its own pack 0.5.2 | All pass | [regression.txt](logs/regression.txt) |
| Secret and privacy scan of every added or changed file | No finding | [security_scan.txt](logs/security_scan.txt) |

All `lcl` commands used `--spec /mnt/F/LCL/canonical/LCL_Core_0.1.0 --project-spec
/mnt/F/LCL/canonical/LCL_Core_0.3.0` with `lcl 0.9.1`. The engine checks the
architecture's structure and cross-references; it cannot check the meaning of its prose.
The script checks what the engine cannot see: the repository layout, the pack, the
requirement map and the dependency graph.

## 7. Check accounting

### Verification

| Check | How it was met |
|---|---|
| T002.VERIFY.01 state and permission boundaries against imported content, late replies and cleanup | Reviews RV-01 to RV-03 in `reviews.lcl.txt` (the agent's own review). `verify.arch_review_gates` and `verify.arch_reviews_hold` hold; the script resolves every reference. |
| T002.VERIFY.02 build or compile the affected targets | The affected target is the architecture LCL project: `lcl check` and `lcl validate` pass. See the applicability note. |
| T002.VERIFY.03 task-specific tests | `lcl run` with 12 `VERIFY` checks, the 27 script checks and the 14 negative tests. |
| T002.VERIFY.04 regressions of the changed subsystem | The charter's checks and TASK-001's cross-check still pass. See the applicability note. |
| T002.VERIFY.05 final diff inspected | Two tracked files changed (`README.md`, `FILE_TREE.txt`); every new file is under `docs/architecture/` or `docs/evidence/TASK-002/`. |
| T002.VERIFY.06 data and authority flow against A1 to A8 | Reviews RV-A1 to RV-A8. No local authoring or LCL-validation path requires a PC (RV-A2, FL-STANDALONE-START, `verify.arch_android_host_standalone`, and the script's check that both hosts compose the same core). Sixteen portability decisions are identified for TASK-003 and later before foundation selection; the renderer stays open (`verify.arch_renderer_left_open`). |

### Completion gate

| Check | How it was met |
|---|---|
| T002.CLOSE.01 dependencies closed | TASK-001 is accepted and its lineage verified (section 3). |
| T002.CLOSE.02 objective without unrelated scope | Components (24 subsystems), ownership boundaries (slots, prohibitions, owner tasks), interfaces (14), data ownership and authoritative state (22 categories) and failure domains (14) are defined. The scope items map to: import, handoff and help inside existing subsystems (AD-08 and the single paths); extension read-only limits (EXT-01 to EXT-07); mobile reuse (AD-01 and the hosts); shared versus client-specific code (the layers); data and authority flow (10 flows); phone-local and Desktop-local hosts with separate remote and sync paths; execution ownership separate from UI focus (AD-06). No code, no technology choice and no later task. |
| T002.CLOSE.03 build passes where applicable | As T002.VERIFY.02. |
| T002.CLOSE.04 required tests pass | As T002.VERIFY.03. |
| T002.CLOSE.05 security checks pass | The scan finds nothing. Security assumptions are explicit (SA-01 to SA-08); unknown implications are recorded as blockers for their owning tasks (EXT-07 and the open decisions); the trust hierarchy and fail-closed behaviour shape the design, for example `verify.arch_authority_never_synced`. No secret was handled. |
| T002.CLOSE.06 no unnecessary code or dependency | No dependency. One script, needed because the engine cannot read the repository or the pack. The single-path list rules out duplicate services. |
| T002.CLOSE.07 no binding invariant violated | Work stayed in the bound checkout on its existing branch and remote; the pack stays outside the repository; the LCL engine, canonical packages and the LCL repository are unchanged; one primary agent; no service started. |
| T002.CLOSE.08 evidence recorded | This folder. |
| T002.CLOSE.09 Git diff understood | As T002.VERIFY.05. |
| T002.CLOSE.10 assigned standalone, remote and sync requirements; later scenarios traceable | The A9 row of TASK-002, device-local runtime and storage boundaries with independent local, remote and sync paths, is HOST-DESKTOP, HOST-ANDROID, FL-LOCAL-EXECUTION, FL-STANDALONE-START, FL-REMOTE-REQUEST and FL-SYNC. AN-01 to AN-12 stay with their later owners; each RC contract names its runtime owners and test plan. |
| T002.CLOSE.11 readability and comments | Agent's own review, section 8. |
| T002.CLOSE.12 scoped cleanup | Section 8. |
| T002.CLOSE.13 post-cleanup verification on the final revision | Run after this file was final: `receipt/final_verification.txt` and `receipt/RECEIPT_RESULT.md`. |
| T002.CLOSE.14 manual impact | Section 8. |

### Remote-control requirements

TASK-002 owns the design, contract and test plan of each requirement below; each
contract carries exactly that status and `verify.arch_remote_design_only` checks it.
**None of them is implemented or runtime-tested.** The runtime owners are those of the
pack's `TASK_REQUIREMENT_MAP.json` after TASK-002; the script checks them.

| Check | Requirement | Realised by | Runtime owners |
|---|---|---|---|
| T002.RC.01 | RC-01 Two-way device actions | SS-REMOTE in both roles, FL-REMOTE-REQUEST, IF-REMOTE | 8 later tasks |
| T002.RC.03 | RC-03 Directional trust | Grants on the destination per peer, direction and capability (ST-AUTHORITY), never synced | 13 later tasks |
| T002.RC.04 | RC-04 Desktop receiver independent of window | HOST-DESKTOP separate from the window, single instance, IF-ATTACH (AD-02) | 9 later tasks |
| T002.RC.05 | RC-05 Startup and login are distinct | Four lifecycle states; user-level start-at-login only (PD-STARTUP) | 10 later tasks |
| T002.RC.08 | RC-08 No idle model usage | Deterministic receiver; `verify.arch_receiver_uses_no_model` (AD-11) | 8 later tasks |
| T002.RC.09 | RC-09 Typed request envelope | Envelope type in `core/protocol/messages`; no ambient target | 15 later tasks |
| T002.RC.13 | RC-13 Phone-hosted actions | Android host executes with the standalone tools; no PC proxy | 14 later tasks |
| T002.RC.19 | RC-19 Transport and private networking | One audited channel per platform for remote control and sync (AD-05, PD-TRANSPORT) | 10 later tasks |
| T002.RC.20 | RC-20 Cross-workspace independence | ST-VIEW is never a targeting input; explicit targets (AD-06) | 20 later tasks |
| T002.RC.23 | RC-23 Secure service integration | Same-user processes, authenticated local IPC, one permission engine | 14 later tasks |
| T002.RC.25 | RC-25 One product, reused services | The single-path list; remote admission is not a dispatcher (AD-03) | 11 later tasks |

### Applicability notes

- **Application build (T002.VERIFY.02, T002.CLOSE.03).** Not applicable to an
  application build: the repository has no build system and no implemented source, and
  the language stack is not chosen (PD-CORE-RUNTIME). What this task produces and can
  compile is its LCL project, and that was done.
- **Regressions (T002.VERIFY.04).** The repository's test files are empty placeholders,
  so there is no product suite to regress. The applicable regressions are the
  predecessor's executable checks, which pass, and the fact that no other file changed.
- **Remote-control checks.** Applicable as design, contract and test-plan records only,
  as TASK-002's remote evidence scope states.

## 8. Review, cleanup and manuals

**Readability (agent's own review).** Each part states its purpose and limits in its
`SPECIFICATION` description, and every record type explains its fields and value
vocabulary. Every slot pairs a path with its responsibility, and prohibitions name the
pack rule behind them wherever one exists; all 141 cited pack identifiers were checked to
exist in the pack. Authoring and review corrected six things before the logs were
recorded: a placeholder `SUCCESS` that referenced the goal, three dependency cycles that
contradicted the build order (resolved by AD-12), help placed in the host instead of the
clients, a development rule (U08) cited for a product boundary, an over-stated
description of the RC-08 check, and a sentence claiming that all state lives in the
state store. The script has a module
docstring, docstrings on its non-obvious helpers and one comment per group of checks.
There is no commented-out code and no filler comment.

**Cleanup.** In the checkout the task created no temporary, debug or obsolete file:
`git status --porcelain --untracked-files=all` lists only the deliverables above, and no
`__pycache__` exists because every script ran with `python3 -B`. Outside the checkout the
session scratch folder held the native results, the prototype LCL project, the
negative-test harness and its copies, and the extracted 0.5.2 archive; the copies and the
archive were removed after use, and the rest is removed after the final verification. No
pre-existing or user-owned file was deleted, reset or stashed.

**Manuals.** No impact. The Nexees manual draft already describes the behaviour this
architecture structures: local work without a PC, remote control and sync as separate
features, separate receiving, startup and keep-running settings, and extensions without
agent authority. The LCL manual draft already says that LCL runs through the one core on
both clients and that adoption is explicit. No user-facing workflow changed.

## 9. Policy review and disclosures

All 135 rules of the nine policy documents were read and reviewed for this task.

| Policy document | Rules | Relation to TASK-002 |
|---|---|---|
| `policies/master_rules.lcl.txt` | 32 | The conduct rules R1 to R7, R9, R10, R15, R17, R20, R25 to R27, R30 and R31 were followed. The product rules are given concrete boundaries, for example R11 by AD-06, R14 by ST-CONTEXT, R21 by HOST-ANDROID and R22 by ST-SESSION. |
| `policies/global_contracts.lcl.txt` | 31 | C1 to C5, C8 to C12, C15, C19, C21, C22 and C28 to C31 shape the architecture; C16, C17, C25 and C27 governed how the task was done; C20 stays open; the rest are placed in their subsystems. |
| `policies/acceptance_criteria.lcl.txt` | 11 | Product acceptance, none due now; every area has an owning subsystem. |
| `policies/code_readability_and_cleanup.lcl.txt` | 6 | Applies in full; section 8. |
| `policies/no_unnecessary_code.lcl.txt` | 7 | Applies: one script, no dependency, no removed file; the single paths exclude duplicate services. |
| `policies/reuse_policy.lcl.txt` | 5 | Reusable components were inspected read-only (RO-01 to RO-04); nothing was adopted, and classification is left to TASK-003. |
| `policies/security_baseline.lcl.txt` | 9 | The trust hierarchy and fail-closed behaviour shape the design; assumptions are explicit; no secret was handled. |
| `policies/usage_and_agents.lcl.txt` | 8 | Followed: one agent working sequentially, one heavy process at a time, Git writes only as CA-06 permits, no service started. |
| `architecture/remote_device_control.lcl.txt` | 26 | The eleven owned requirements are designed; the others have owners and boundaries; nothing was enabled. |

Disclosures:

- **Results folder.** The native suite wrote its results to the session scratch folder
  rather than the `/tmp` path in README_FIRST.txt, which allows a fresh scoped folder.
- **Session hook.** A session hook asks for a dynamic web-application security scan
  after product code changes. This task changed documentation and added one offline
  evidence script, and Nexees has no running application to scan, so no scan was run.
- **Reuse inspection.** Reading the owner's LCL repository (engine crates, Android app,
  remote service) was read-only and is recorded as observations RO-01 to RO-03.
- **Network.** No network operation was made before the final push.

## 10. Open items and limits

- **DESKTOP-RENDERER-01** is unresolved and belongs to the owner at TASK-003.
- **Sixteen open decisions** (PD-CORE-RUNTIME, PD-RENDERER, PD-IDE-PLACEMENT,
  PD-LCL-ANDROID, PD-PERSISTENCE, PD-ANDROID-EXECUTION, PD-ANDROID-BACKGROUND,
  PD-TRANSPORT, PD-SECRETS, PD-LOCAL-IPC, PD-STARTUP, PD-ARCHIVE, PD-MANUAL-RENDERER,
  PD-GIT-ANDROID, PD-CODE-INDEX-ANDROID and PD-PROVIDER-ANDROID) await TASK-003 and the
  tasks named in each.
- **Seven gaps** (GAP-01 to GAP-07) name missing slots and test files and their owners.
  GAP-07 notes that no task of the 75 is dedicated to building the updater; the plan is
  unchanged.
- **The catalogue describes layout revision 0.4.** A task that adds, moves or merges a
  slot, such as TASK-005, updates `subsystems.lcl.txt` and reruns the script.
- **Nothing is implemented or runtime-tested.** The architecture is design; its checks
  are structural.
- **BIND-B1-LOGO** and **AMEND-075-01** remain as the charter records them.
- Observed and left unchanged: `.directory`, a tracked desktop folder-settings file not
  listed in `FILE_TREE.txt`.

## 11. Next task

TASK-003. It was not started; it needs the owner's DESKTOP-RENDERER-01 answer before it
can approve a renderer-dependent foundation.
