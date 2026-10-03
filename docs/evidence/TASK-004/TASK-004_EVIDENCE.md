# TASK-004 evidence: Create threat model, trust hierarchy, and security invariants

| | |
|---|---|
| Task | TASK-004 — Create threat model, trust hierarchy, and security invariants |
| Date | 2026-10-03 to 2026-10-04 |
| Performed by | Coding agent (Claude Code), one primary session; no sub-agents, reviewers or background AI jobs |
| Checkout | `/mnt/F/Nexees/`, branch `main`, base `2eaa199f28ec889a1e59a770b27f7bbf56ec9336`, then the two owner-requested TASK-003 receipt commits `a590eab` and `bccc9b1` (section 2) |
| Repository | `https://github.com/aivars11232/Nexees` |
| Specification pack | `/mnt/F/Nexees_LCL_Implementation_Pack_v0.5.3/`, archive revision 0.5.3, specification version 0.5.0 |
| Pack content identity | `bd5e8fce26f542c44a891e4340da3b2a84f79ab368b5cb868beb48126c2db3e1` |
| Pack manifest SHA-256 | `8d9d0080ac13a3fb17cc70603b77eb3b11abcd3fb6d2fad515cfac136fcf07e4` |
| Procedure | Continuation profile 0.5.3 (CA-01 to CA-12), as adopted for TASK-002 and continued for this task |
| Predecessors | TASK-001 (`284f97a`), TASK-002 (`d1a673f`) and TASK-003 (`2eaa199`, receipt corrected and decided in `a590eab` and `bccc9b1`), all accepted |
| Commit of this work | Made after acceptance, as CA-06 permits; it is recorded by Git, not in this file |

## Status

The deliverables are complete and every applicable check passed on its final run.
Acceptance is recorded in [receipt/RECEIPT_RESULT.md](receipt/RECEIPT_RESULT.md), which is
written after this file because the receipt contains a snapshot of everything else.

TASK-004 created [docs/security/](../../security/), an LCL Core 0.3.0 project holding the
Nexees threat model:

- the pack's eight-level trust hierarchy applied to Nexees, and eight separate authorities;
- twelve trust boundaries, fifteen assets covering every state category, and 34 attack
  surfaces;
- 30 immutable security invariants;
- 55 threats across the nine areas the task names, each traced to later owning tasks and
  to the architecture subsystems, interfaces and module slots that will enforce it;
- 39 planned threat tests, including all six v0.3 categories;
- the security contracts, message types and test plans of its eleven remote-control
  requirements.

**This is design.** Nothing in it is implemented or runtime-tested, and no remote
requirement or scenario is claimed as proven.

During the task the owner asked for two TASK-003 corrections and then decided on TASK-003's
deviation. Both corrections are made, verified and committed separately (section 2).

All review here is the agent's own. Under CA-05 the routine corroboration was done by this
session's own local tools; no independent human or model review has taken place.

## 1. Authority

The owner's instruction, verbatim:

```
proceed with task 4
```

This continues the procedure the owner adopted with pack 0.5.3 for TASK-002 (CA-01): one
named task through evidence, receipt acceptance, commit and push, then stop. TASK-005 is
not covered.

During the task the owner sent this message, verbatim:

```
Small correction during TASK-004:

Continue TASK-004 normally and follow the existing task-package rules.

Before TASK-004 is closed, reconcile these two TASK-003 audit findings:

1. TASK-003 records that the LCL SDK `.knownPackages` cache was rewritten, but its accepted receipt still says `no_unrelated_changes = true` with no blocker. Correct the TASK-003 evidence/receipt so it truthfully represents what actually happened, without changing or redoing the TASK-003 engineering work. Use the narrowest valid correction and rerun the affected acceptance checks.

2. `FILE_TREE.txt` lists `docs/evidence/TASK-003/logs/pc_receiver.log`, but that file is not in the repository. Correct the file tree unless the real file genuinely exists in preserved evidence. Do not invent or reconstruct evidence that was never recorded.

Do not restart TASK-001–003 and do not disturb current TASK-004 work. After these corrections are verified, continue TASK-004 normally.
```

The owner then answered one question. The question and the chosen answer, verbatim:

| Question | Answer |
|---|---|
| I corrected TASK-003's receipt as you asked, and the acceptance program now refuses it (validate.scope). The only unrelated change is the LCL SDK's 16-byte `.knownPackages` file: a package-list cache that SDK tools regenerate themselves, rewritten by avdmanager when it created the emulator you approved. No SDK package changed, but the old bytes are gone. TASK-004 is built and every check passes, but its receipt must state that TASK-003 is closed. How should I resolve this? | Approve cache rewrite (Recommended): "You accept the rewritten cache as your SDK's current state. I record your decision in TASK-003's receipt as the reason, set no_unrelated_changes back to true and clear the blocker. The deviation stays disclosed. I re-run TASK-003's acceptance, then accept, commit and push TASK-004 (two commits: correction, then TASK-004)." |

## 2. The TASK-003 corrections

The full record is in TASK-003's
[receipt/RECEIPT_RESULT.md](../TASK-003/receipt/RECEIPT_RESULT.md) and sections 7 and 8 of its
[corroboration.txt](../TASK-003/receipt/corroboration.txt). In short:

- **The SDK cache.** The receipt's `no_unrelated_changes: true` was untrue: `avdmanager` had
  rewritten `/mnt/F/.lcl-android/sdk/.knownPackages`, and its earlier bytes cannot be shown.
  - The narrowest correction changed two inputs: `no_unrelated_changes` to false, and the
    blocker `UNRELATED_CHANGE_LCL_SDK_CACHE` added. `expected` changed to `refuse`.
  - The acceptance program then refused the receipt on exactly `validate.scope` and
    `validate.no_blocker`. That was commit `a590eab`.
  - Asked how to resolve it, the owner approved the rewrite. The receipt records the
    question and answer in `owner_decision`, and the two values are restored on that basis.
  - The program accepts it again, with an engine record byte-identical to the first
    acceptance. That was commit `bccc9b1`.
- **`pc_receiver.log`.** The file is genuine. Its SHA-256 equals the line in TASK-003's
  accepted snapshot listing. `.gitignore`'s `*.log` rule had kept it out of commit `2eaa199`,
  and it was the only snapshot file missing from that commit. `FILE_TREE.txt` was right, so
  it was not changed.
  - Commit `a590eab` adds the file unchanged with `git add -f`.
  - From that commit, the TASK-003 snapshot `787b17e6…5153` rebuilds exactly, and
    `FILE_TREE.txt` matches the tree.
- **TASK-001 and TASK-002** were checked the same way: each snapshot listing equals its
  commit tree, outside the receipt folder.
- **No TASK-003 work was redone**, and no TASK-003 file outside the receipt folder changed,
  apart from adding the log.

The approval got its own commit, so that the refused state stays in the history. The push
therefore carries three commits rather than the two the answer's text named.

## 3. Checkout preflight (binding B26)

| Item | Observed |
|---|---|
| Working directory | `/mnt/F/Nexees/`; the Git worktree root, with no parent repository, superproject or second worktree |
| `origin` fetch and push | `https://github.com/aivars11232/Nexees.git`, an approved equivalent; no URL rewrite, no separate push URL |
| Branch and HEAD at task start | `main` at `2eaa199`, equal to the local `origin/main`; upstream `origin/main`; clean worktree right after the TASK-003 commit and push |
| Hooks | None besides Git's samples; `core.hooksPath` unset |
| Recheck at 23:55 | HEAD `bccc9b1`: `2eaa199` plus the two TASK-003 receipt commits, not yet pushed; nothing staged, no stash; only this task's files changed or added |
| Result | Passes ([logs/preflight.txt](logs/preflight.txt)) |

## 4. Pack, native verification, dispatch and predecessors

**Pack.** All 201 manifest entries match their SHA-256, with no unlisted or missing file.
The content identity recomputes to `bd5e8fce…b3e1`. `tasks/task_004.lcl.txt` (`3ec98275…bd1bd`)
equals the `after_sha256` of `PROCEDURAL_CHANGES.json`, whose entry confirms that only
procedural fields changed from 0.5.2.

**Native verification.** The pack's runner, `tools/verify_with_lcl.py`, passed all 51
synthetic language cases with `lcl 0.9.1`
([logs/native_language_suite.json](logs/native_language_suite.json)). These are language
tests, not evidence for any task.

**Dispatch.** `lcl run … --input input.task_number=4 main.lcl.txt` ran `task.dispatch` and
`task.task_004` only, with `status.succeeded` and no diagnostics. It published the TASK-004
packet (SHA-256 of the packet text `0522cf19…baf2`), with the procedure amendment applying
([logs/dispatch_task_004.record.json](logs/dispatch_task_004.record.json)).

**Predecessors.**
- **TASK-001**: engine record `bbda5d83…07e7`, `status.succeeded`; commit on `origin/main`.
- **TASK-002**: engine record `c5fadc83…bd13`, `status.succeeded`; commit on `origin/main`.
- **TASK-003**: engine record `5e4a83c1…56d0`, `status.succeeded`, as decided by the owner
  (section 2); commit on `origin/main`.
- `docs/charter/`, `docs/evidence/TASK-001/`, `docs/evidence/TASK-002/`,
  `docs/architecture/` and `docs/dependencies/` are unchanged by TASK-004.

## 5. Required reading

The continuation profile and the TASK-004 record were read first. Then the security
documents TASK-004 works from were read again in full or in the parts named:
- the security baseline, the authority hierarchy, the security gate and the stop conditions;
- the global contracts and the remote-control rules with their scenarios;
- the import pipeline, the model handoff and the manuals;
- the cleanup policy, auth and settings, the provider model, the Android operating model;
- the test strategy, the v0.3 coverage, the acceptance gates and the requirement map.

The rest of the mandatory read order was read in full for TASK-002 earlier in this session
and is byte-identical now, so it was reused, as the profile allows.
[logs/required_reading.txt](logs/required_reading.txt) lists every file with its SHA-256.
Every hash equals the one TASK-002 or TASK-003 recorded, except the task file, which is new.
No external source was consulted.

## 6. What the threat model contains

[docs/security/security.lcl.txt](../../security/security.lcl.txt) is the entry. The parts:

| Part | Content |
|---|---|
| `baseline.lcl.txt` | The task binding; the nine areas and six v0.3 categories; the immutability rule; outcomes, audit events, never-remote operations and lock categories. The record types, and six message types: request envelope, grant, approval, audit record, receiver settings, lock policy. Limits: request lifetime 10 minutes, clock skew 2 minutes, 64 KiB frames, 4 KiB IPC messages. |
| `trust.lcl.txt` | The eight trust levels, named exactly as `architecture/authority_hierarchy.lcl.txt` names them, with what holds each level in Nexees. Six application rules, eight separate authorities (AU-01 to AU-08) and twelve trust boundaries (TB-01 to TB-12). Eight assumptions (SEC-A01 to SEC-A08) in addition to the architecture's SA-01 to SA-08. Eight residual risks (R-01 to R-08), recorded and not accepted, and eight open security decisions (OSD-01 to OSD-08). |
| `assets.lcl.txt` | Fifteen assets covering all 22 state categories of `docs/architecture/state.lcl.txt`, and 34 attack surfaces. |
| `invariants.lcl.txt` | SI-01 to SI-30, each with sources, enforcing subsystems and module slots, owning tasks and tests. Two coverage tables: every one of the 18 required controls of the security baseline, and every one of the 8 security stop conditions, maps to invariants. |
| `threats.lcl.txt` | TH-01 to TH-55. Each threat has actor, assets, surfaces, scenario, invariants, mitigations, the runtime-enforced boundary, owning tasks, tests and residual risk. |
| `tests.lcl.txt` | TT-01 to TT-39 threat tests, with slot, scenarios, owning tasks and platforms. SG-01 to SG-09 record the test slots the source layout lacks, each with the task that adds it. |
| `remote.lcl.txt` | The eleven remote security contracts (RC-02, 03, 07, 09, 10, 17, 18, 19, 22, 23, 24). The destination's 14 ordered execution checks (EC-01 to EC-14). Plans for the four assigned scenarios. Fresh-install defaults: receiving, startup and keep-after-close off, empty allowlist, nothing sensitive while locked. Consistent illustrations of an envelope, grant, approval and audit record. |
| `checks.lcl.txt`, `record.lcl.txt` | Two `VALIDATE` and ten `VERIFY` checks, and the record task. |

Threats per area: Desktop 12, Android 10, agent tools 8, providers 7, LCL 5, extensions 3,
auth 5, updates 4, cross-device sessions 13. A threat can count in more than one area.

The v0.3 categories, as threats and threat tests:

| Category | Threats | Threat tests |
|---|---|---|
| Malicious archives | TH-26, TH-27, TH-29 | TT-01 to TT-05 |
| Imported-rule injection | TH-28, TH-48 | TT-06 to TT-08 |
| Model-switch races | TH-22, TH-23 | TT-09 to TT-13 |
| New-provider data disclosure | TH-21 | TT-14 to TT-16 |
| Unsafe cleanup | TH-18 | TT-17 to TT-19 |
| Active help content | TH-51 | TT-20, TT-21 |

The objective's topics map as follows:

| Topic | Where it is recorded |
|---|---|
| Attack surfaces | SF-01 to SF-34 |
| Authority hierarchy | The trust levels, TR-01 to TR-06, and SI-01 |
| Least privilege | SI-02 and SI-03 |
| Secrets | SI-08, TB-12, TH-16, TH-24, TH-35, TH-52 |
| Sandboxing | SI-06, TH-15 and R-05 |
| Extension isolation | SI-22, TH-31 to TH-33, and R-02, which TASK-021 needs under EXT-07 |
| Desktop↔Android trust boundaries | TB-04, SI-09 to SI-17, TH-43 to TH-50 and the remote contracts |

Several facts were read directly from the TASK-003 prototype installation, with no network
access ([logs/foundation_security_facts.txt](logs/foundation_security_facts.txt)):

- **Theia 1.76.0 window settings:** `contextIsolation: true`, `nodeIntegration: false`, but
  `sandbox: false`.
- **Theia backend token:** the backend requires a token cookie, and allows every request
  when the token variable is absent.
- **Theia backend environment:** the backend is forked with the whole Electron environment.
- **Electron 42.11.10 fuses:** RunAsNode, NODE_OPTIONS and inspector arguments are enabled,
  and asar integrity validation is disabled.
- **The Android prototype:** backups are off, exported components are minimal and
  PendingIntents are immutable.

TH-02 to TH-04, SI-28 and SI-29 rest on these facts.

## 7. Deliverables

| Path | Change |
|---|---|
| `docs/security/` (10 files) | New: the threat-model LCL project |
| `README.md` | Introduction and layout table name the dependency inventory and the threat model; one paragraph on `docs/security/` |
| `SECURITY.md` | One paragraph pointing to the threat model and its residual risks; the start-here marker line is kept |
| `FILE_TREE.txt` | Regenerated for the new files |
| `docs/evidence/TASK-004/` | This record, `check_threat_model.py`, `logs/` and, written last, `receipt/` |

Committed separately, at the owner's request (section 2):
- `a590eab`: `docs/evidence/TASK-003/logs/pc_receiver.log` and the TASK-003 receipt folder;
- `bccc9b1`: the TASK-003 receipt folder.

No product placeholder, architecture file, dependency record, charter file or manual changed.

## 8. Checks run

| Command | Result | Log |
|---|---|---|
| `lcl check`, `validate`, `run` of `docs/security/security.lcl.txt` | Exit 0 each; `status.succeeded` with all 10 `VERIFY` TRUE | [security_lcl.txt](logs/security_lcl.txt) |
| `python3 -B docs/evidence/TASK-004/check_threat_model.py <pack>` | 38 of 38 pass, exit 0 | [security_lcl.txt](logs/security_lcl.txt) |
| Negative tests: 21 seeded defects in scratch copies, plus 3 unmutated controls | All detected by the intended check; controls pass | [negative_tests.txt](logs/negative_tests.txt) |
| Regression: charter, architecture and dependency `lcl` checks; the TASK-001, TASK-002 and TASK-003 cross-checks; predecessor folders unchanged | 16 commands, all exit 0 | [regression.txt](logs/regression.txt) |
| Native language suite | 51/51 | [native_language_suite.json](logs/native_language_suite.json) |
| Foundation facts (Theia, Electron fuses, Android prototype) | Recorded read-only | [foundation_security_facts.txt](logs/foundation_security_facts.txt) |
| Secret and privacy scan of every added or changed file, and of the correction commit | No finding | [security_scan.txt](logs/security_scan.txt) |

`check_threat_model.py` checks what the engine cannot see from inside `docs/security/`:

- **The remote contracts against the pack:** the assigned IDs, runtime owners and scenario
  plans equal `TASK_REQUIREMENT_MAP.json`.
- **The v0.3 gate:**
  - every threat's owners are tasks from TASK-005 on that own a subsystem of its boundary;
  - every boundary element exists in the architecture;
  - every enforcement point is a module slot of that boundary and exists in the source
    layout.
- **References:** every test reference is a real test slot, a pack scenario or a recorded
  gap; every invariant answers a threat; every asset and surface is threatened.
- **Coverage against the pack:** the 22 state categories, the 18 required controls and the 8
  security stop conditions are covered, and the trust levels equal the pack's.
- **Identifiers:** every one mentioned in the texts resolves, and the numbering has no gaps.

The negative tests prove that these checks, and the LCL `VERIFY` checks, catch each kind of
defect.

## 9. Check accounting

### Verification

| Check | How it was met |
|---|---|
| T004.VERIFY.01 trace each threat to an owning task and a runtime-enforced boundary | Every one of the 55 threats names owning tasks (TASK-005 or later), the subsystems and interfaces of its boundary, the module slots where enforcement lives, invariants and tests. `verify.sec_threats_traced` checks the structure; `check_threat_model.py` resolves each reference and requires the owners to own the boundary. |
| T004.VERIFY.02 build the affected targets | Documentation and specification task: no product target exists yet (TASK-005 creates the scaffold). The affected targets are LCL projects, checked by `lcl check`, `validate` and `run`: the new security project, and the architecture, dependency and charter projects it depends on (CA-04). |
| T004.VERIFY.03 task-specific tests | `check_threat_model.py` (38 checks), the 10 `VERIFY` checks and the 21 negative tests (section 8). |
| T004.VERIFY.04 regressions of the changed subsystem | The architecture and dependency projects the model refers to and their cross-checks, the charter and TASK-001's cross-check, and the native suite. No product test suite exists yet. |
| T004.VERIFY.05 final diff inspected | Section 7 lists every changed and new file; no product placeholder changed; the receipt's final verification repeats the inspection. |

### Completion gate

| Check | How it was met |
|---|---|
| T004.CLOSE.01 dependencies closed | TASK-003 accepted, with its receipt corrected and decided by the owner, and its lineage verified (sections 2 and 4). |
| T004.CLOSE.02 objective without unrelated scope | The scope's three items are done: the threat model of the nine areas, the immutable invariants, and the six v0.3 categories in threat tests. Nothing was implemented, and nothing of TASK-005 or later was started. The TASK-003 corrections were the owner's explicit request. |
| T004.CLOSE.03 build passes | As T004.VERIFY.02. |
| T004.CLOSE.04 required tests pass | As T004.VERIFY.03. |
| T004.CLOSE.05 security checks pass | The secret and privacy scan is clean, and it covered the TASK-003 log never scanned before. The model's own checks and negative tests pass. The security gate's per-task demonstrations belong to the tasks that build each capability; the threat tests assign them. |
| T004.CLOSE.06 no unnecessary code or dependency | No product code and no dependency. One evidence script, `check_threat_model.py`, because the engine cannot read the architecture, the source layout or the pack map from inside the project. |
| T004.CLOSE.07 no workspace/agent/LCL binding invariant violated | Bound checkout, branch and remote. The pack, the LCL repository, the canonical packages and the LCL SDK were not written by TASK-004 (final verification). One primary agent. The owner's receiver, startup, phone, firewall and router were untouched, and no process was started outside scratch tests. |
| T004.CLOSE.08 evidence recorded | This folder. |
| T004.CLOSE.09 Git diff understood | As T004.VERIFY.05; the two TASK-003 commits are explained in section 2. |
| T004.CLOSE.10 readability and comments | Agent's own review, section 10. |
| T004.CLOSE.11 scoped cleanup | Section 10. |
| T004.CLOSE.12 post-cleanup verification on the final revision | `receipt/final_verification.txt`. |
| T004.CLOSE.13 manual impact | Section 10: not applicable, with the reason. |

### Remote-control requirements and scenarios

Each requirement's contract is in `docs/security/remote.lcl.txt`, with its message types,
invariants, threats, test plan and runtime owners. **Each is closed as design only. None is
implemented or runtime-tested.** The runtime owners equal the pack map after TASK-004.

| Check | How it was met | Limits and owners |
|---|---|---|
| T004.RC.02 explicit opt-in | `type.sec_receiver_settings`, all off on a fresh install (`data.sec_receiver_defaults`, `verify.sec_safe_defaults`). Changed only by a local action through the permission engine, with step-up for receiving and startup. Pairing, login, sync, update and remote requests have no path (SI-15). | Runtime: TASK-022, 030, 060, 070, 071, 073 to 075 |
| T004.RC.03 directional trust | `type.sec_grant`: held only by the destination, keyed by peer, user, direction and capability, with epoch; never synced or reversed (SI-10). Builds on TASK-002's `data.arch_rc_03`. | Runtime: TASK-007 to TASK-075, as mapped |
| T004.RC.07 settings and immediate stop | Settings content, and "stop remote access": receiving off, epoch bump that voids pending approvals and queued requests, sessions closed, audit record; local editing continues (SI-15, SI-13). | Runtime: TASK-008, 022, 030, 055, 056, 060, 067, 068, 070, 071, 073 to 075 |
| T004.RC.09 typed request envelope | `type.sec_request_envelope`: all RC-09 fields; sender identity taken from the channel; explicit destination triple; workspace and agent only where they apply; expiry clamped by the destination (SI-12). Builds on `data.arch_rc_09`. | Runtime: TASK-007 and the others mapped |
| T004.RC.10 execution-host checks | EC-01 to EC-14, in order, immediately before the effect, failing closed (SI-11). It extends the order TASK-003's prototype dispatcher used. | Runtime: TASK-007, 015, 022, 023, 031, 033, 055, 060, 066, 070, 071, 073 to 075 |
| T004.RC.17 replay and unknown outcomes | Request ledger; journal before the effect; outcome-unknown blocks retry of non-idempotent operations; single-use approvals bound to the request hash and revalidated; nothing in sync queues (SI-13). | Runtime: TASK-007, 008, 026 and the others mapped |
| T004.RC.18 lock and sensitive actions | `type.sec_lock_policy`, default nothing while locked; `binding.sec_never_remote` refused from any remote request and never on a model's request alone (SI-14). | Runtime: TASK-022, 024, 025 and the others mapped |
| T004.RC.19 transport and private networking | TLS 1.3 mutual authentication with pinned keys; no unauthenticated fallback; frame, queue, rate and connection limits; listener only while receiving; no UPnP or relay (SI-16). Builds on `data.arch_rc_19`. | Off-LAN route: owner decision (OI-03, OSD-02); runtime TASK-025, 060, 069, 070, 071, 073 to 075 |
| T004.RC.22 audit and privacy | `type.sec_audit_record` with the normalized target, decision, reason, policy revision, epoch and outcome, never content or secrets; live versus last-known reachability (SI-25). | Runtime: TASK-025, 026, 030, 056, 068, 070, 071, 073 to 075 |
| T004.RC.23 secure service integration | Per-user 0700 socket, peer uid check, 4 KiB messages; no elevation or privileged helper; the IDE backend only with its token; protocol mismatch refused (SI-17, SI-28). Builds on `data.arch_rc_23`. | Runtime: TASK-007, 009, 022, 024, 025, 056, 057, 070, 071, 073 to 075 |
| T004.RC.24 update, logout and removal | Settings, grants, revocations and epochs carried over unchanged; never reset to allow; sign-out bumps epochs; uninstall removes only owned registrations; no false rollback claims (SI-26, SI-30). | No updater task (GAP-07, OSD-04); runtime TASK-008, 009 and the others mapped |
| T004.RCT.01 | Planned as TT-22, with slot `tests/e2e/cross-device/remote_agent_control` | Not run; TASK-022, 030, 055, 060, 068, 070, 073 to 075 |
| T004.RCT.08 | Planned as TT-23: the full rejection matrix | Not run; TASK-007, 015, 022, 023, 025, 031, 033, 055, 060, 066, 067, 069, 070, 073 to 075 |
| T004.RCT.12 | Planned as TT-24: labels, arguments, binary paths, deep links, replaced entries, Android extras; slot gap SG-09 | Not run; TASK-022, 023, 024, 025, 031, 033, 070, 073 to 075 |
| T004.RCT.15 | Planned as TT-25: TLS downgrade, unpinned and revoked certificates, limits, wrong-uid IPC, off-LAN route; slot gap GAP-04 | Not run; TASK-025, 060, 069, 070, 073 to 075 |

The receipt's remote-observation inputs stay `unverified`: the pack requires them only for
tasks 70 and 73 to 75, and they describe the product, which does not exist yet.

## 10. Review, cleanup and manuals

**Readability (agent's own review).** Each part's `SPECIFICATION` description states its
purpose, and every record type, every list and every field of the six message types carries
a description. The
records use plain sentences rather than shorthand. `check_threat_model.py` documents what it
checks and why the engine cannot.

The threat model was authored as structured data by a scratch generator, which is not part
of the repository; the LCL files are the source of truth from now on. Review and testing
corrected the following before the final logs:
- three traceability errors that the cross-check found: an asset no threat named, two owners
  outside a threat's boundary, and a boundary missing its protocol subsystem. A fourth
  failure was the checker's own rule, which rejected modules inside a folder slot;
- a missing invariant for checkpoints and rollback (SI-30);
- trust-level names not taken verbatim from the pack;
- a negative-test case that did not mutate what it claimed;
- a wrong citation in the foundation-facts log.

**Cleanup.** The generator, the trial projects, the scratch copies of the negative tests and
the extracted 0.5.2 baseline lived in the session's scratch folder, outside the checkout.
That folder is removed after the receipt. Every Python script ran with `-B`. No task process
was started that outlives its command. No pre-existing or user-owned file was deleted,
reset or stashed. The receipt's final verification records the result.

**Manuals.** Not applicable: TASK-004 changes no user-visible behaviour. The manuals'
existing security statements were checked against the invariants and agree:
- separate authorities;
- fresh devices receive nothing until enabled and paired;
- independent grants per direction;
- unacknowledged means unknown;
- sync carries no secrets, grants or approvals;
- extensions gain no agent authority;
- logs redact credentials;
- orientation off still enforces permissions.

## 11. Policy review and disclosures

All 135 rules of the nine policy documents were reviewed for this task.

| Policy document | Rules | Relation to TASK-004 |
|---|---|---|
| `policies/master_rules.lcl.txt` | 32 | Conduct rules followed; R9 fail closed and the evidence rules shape SI-02 and the design-only status |
| `policies/global_contracts.lcl.txt` | 31 | C3, C6 to C14, C21 to C24 and C28 to C31 are sources of invariants; nothing is implemented |
| `policies/acceptance_criteria.lcl.txt` | 11 | Product acceptance, none due now; the threat tests feed the owning tasks |
| `policies/code_readability_and_cleanup.lcl.txt` | 6 | Applied to the project and the script; Q3 and Q5 are the basis of SI-24 and the unsafe-cleanup threat tests |
| `policies/no_unnecessary_code.lcl.txt` | 7 | No product code or dependency; one evidence script |
| `policies/reuse_policy.lcl.txt` | 5 | No new reuse decision: the model builds on TASK-003's inventory and adds no dependency |
| `policies/security_baseline.lcl.txt` | 9 | The trust hierarchy, all 18 required controls (mapped), the data-boundary, credential, developer-settings and v0.2/v0.3 rules are invariants; assumptions explicit (SEC-A01 to SEC-A08) |
| `policies/usage_and_agents.lcl.txt` | 8 | One agent; no heavy process; Git writes only as CA-06 permits |
| `architecture/remote_device_control.lcl.txt` | 26 | The eleven assigned requirements closed as design with runtime owners; the four scenarios planned |

Disclosures:

- **Network.** None used. The Theia and Electron facts come from the local TASK-003
  installation.
- **Session hook.** A hook asks for a dynamic web-application security scan after code
  changes and commits. There is no running Nexees web application to scan and
  `HAWK_API_KEY` is unset, so none was run.
- **Owner-only state.** HopToDesk, linger, the owner's autostart folder, the phone and the
  LCL SDK were not touched by TASK-004.

## 12. Deviations and findings

- **TASK-003 findings reconciled at the owner's request** (section 2). The SDK cache
  deviation is now approved by the owner and recorded in the TASK-003 receipt. The missing
  log is committed.
- **A lesson for the evidence convention.** A final verification that counts untracked files
  misses ignored ones. This task's final verification therefore also requires that no
  ignored file exists in the checkout and that the files about to be committed equal the
  snapshot listing.
- **A Theia behaviour that later tasks must handle.** Theia's backend allows every request
  when its token variable is missing, so it is safe only when the application starts it
  (TH-02, SI-28).
- **The Electron defaults.** Electron's default fuses allow the packaged binary to run as
  Node (TH-03, SI-28). TASK-009 and TASK-073 own both.

## 13. Open items and limits

- **Design only.** No requirement, scenario or threat test was implemented or run; each
  belongs to its owning task.
- **Gaps.** Nine test slots are missing (SG-01 to SG-09), beyond the architecture's GAP-01
  to GAP-07; each names the task that adds it.
- **Open decisions.** Eight security decisions are still open (OSD-01 to OSD-08). Among
  them: extension confinement, the off-LAN route, the auth backend, the update mechanism,
  the Desktop confinement backend and Theia's renderer sandbox.
- **Residual risks.** Eight are recorded, not accepted (R-01 to R-08); each names who reduces
  it or obtains the owner's acceptance.
- **Also open:** OI-01 to OI-10 of `docs/dependencies/`, BIND-B1-LOGO and AMEND-075-01, as
  their records state.

## 14. Next task

TASK-005 (repository scaffold and engineering conventions). It was not started.
