# TASK-006 evidence: Implement shared domain model

| | |
|---|---|
| Task | TASK-006 — Implement shared domain model |
| Date | 2026-10-04 |
| Performed by | Coding agent (Claude Code), one primary session; no sub-agents, reviewers or background AI jobs |
| Checkout | `/mnt/F/Nexees/`, branch `main`, base `e83dcd74b6ea2338b865bb8beccf8d809c476b06` |
| Repository | `https://github.com/aivars11232/Nexees` |
| Specification pack | `/mnt/F/Nexees_LCL_Implementation_Pack_v0.5.3/`, archive revision 0.5.3, specification version 0.5.0 |
| Pack content identity | `bd5e8fce26f542c44a891e4340da3b2a84f79ab368b5cb868beb48126c2db3e1` |
| Pack manifest SHA-256 | `8d9d0080ac13a3fb17cc70603b77eb3b11abcd3fb6d2fad515cfac136fcf07e4` |
| Procedure | Continuation profile 0.5.3 (CA-01 to CA-12), as adopted for TASK-002 and continued for this task |
| Predecessors | TASK-001 (`284f97a`), TASK-002 (`d1a673f`), TASK-003 (`2eaa199`, receipt decided in `bccc9b1`), TASK-004 (`0337be0`) and TASK-005 (`e83dcd7`), all accepted |
| Commit of this work | Made after acceptance, as CA-06 permits; it is recorded by Git, not in this file |

## Status

The deliverables are complete and every applicable check passed on its final run.
Acceptance is recorded in [receipt/RECEIPT_RESULT.md](receipt/RECEIPT_RESULT.md), which is
written after this file because the receipt contains a snapshot of everything else.

TASK-006 implemented the shared domain model in the crate `nexees-domain` (`core/domain`):

- **One versioned vocabulary for both hosts.** Workspaces and their device-local replicas,
  devices and their trust, agent sessions and their bindings, tasks and evidence, LCL
  revisions, imports, model handoffs, permission grants, remote grants, approvals, the remote
  request envelope, and a UI client's focus.
- **Strict validation.** Every value is checked when it is built and again when it is decoded.
  Records whose fields must agree exist only after their whole validation passed. Unknown and
  malformed fields are rejected.
- **Explicit bindings.** Sessions, tasks, imports, handoffs and requests name their device,
  workspace and revisions. A revision of another workspace is rejected, and UI focus is never
  an input to any of them.
- **Separate planes.** Every record declares whether it may leave its device. Control records
  (grants, approvals, requests) and UI focus never travel.
- **Contract tests.** 75 tests beside the code, which 21 seeded defects show catch what they
  claim to catch.

The crate holds types, their validation and their serialization only, as SS-DOMAIN-PROTOCOL
requires. Stores, protocol and behaviour attach to these schemas in their owning tasks.

The owner's VS Code now keeps build output in the checkout, and it wrote `Cargo.lock` and
downloaded crates when the manifests changed (section 11). TASK-006 left all of it in place
and fixed forward the TASK-005 rule that this output broke.

All review here is the agent's own. Under CA-05 the routine corroboration was done by this
session's own local tools; no independent human or model review has taken place.

## 1. Authority

The owner's instruction, verbatim:

```
ou, okay, then preceed with task6
```

It followed the owner's question why the toolchain folder `/mnt/F/Nexees-toolchains` exists,
which the agent answered from TASK-003's record: the owner chose that folder for the tools,
and the Nexees folder holds the project. TASK-005 was accepted and pushed before this task
began (`0337be0..e83dcd7` on `origin/main`). This continues the procedure the owner adopted
with pack 0.5.3 (CA-01): one named task through evidence, receipt acceptance, commit and push,
then stop. TASK-007 is not covered. No owner question was needed during the task.

A second pack, `Nexees_PostCore_LCL_Implementation_Pack_v1.2.0`, appeared beside the checkout
before this task. Its README states that it starts only after TASK-075 and does not replace
this pack. It was read only to establish that, and not used.

## 2. Checkout preflight (binding B26)

| Item | Observed |
|---|---|
| Working directory | `/mnt/F/Nexees/`; the Git worktree root, with no parent repository, superproject or second worktree |
| `origin` fetch and push | `https://github.com/aivars11232/Nexees.git`; no URL rewrite, no separate push URL |
| Branch and HEAD | `main` at `e83dcd7`, equal to `origin/main` after a fetch of that branch only, without tags |
| Hooks | None besides Git's samples; `core.hooksPath` unset |
| At task start | No staged file and no stash. Not clean: the owner's IDE had written 27 ignored files under `target/` and 17 untracked files under TASK-003's Gradle prototype (`.gradle/`) at 11:52, after TASK-005 closed and before TASK-006 began. They are recorded as pre-existing, not task-owned state. |
| Result | Passes ([logs/preflight.txt](logs/preflight.txt)) |

TASK-006 began at 2026-10-04T11:57:55+02:00, when the owner's instruction arrived. That time is
the baseline of every "nothing written since" check.

## 3. Pack, native verification, dispatch and predecessors

**Pack.** All 201 manifest entries match their SHA-256, with no unlisted or missing file.
The content identity recomputes to `bd5e8fce…b3e1`. `tasks/task_006.lcl.txt` (`4accccce…865e`)
equals the `after_sha256` of `PROCEDURAL_CHANGES.json`, whose entry confirms that only
procedural fields changed from 0.5.2.

**Native verification.** The pack's runner passed all 51 synthetic language cases with
`lcl 0.9.1` ([logs/native_language_suite.json](logs/native_language_suite.json)). These are
language tests, not evidence for any task.

**Dispatch.** `lcl run … --input input.task_number=6 main.lcl.txt` ran `task.dispatch` and
`task.task_006` only, with `status.succeeded` and no diagnostics. It published the TASK-006
packet (SHA-256 of the packet text `25126db9…09af`), with `procedure_amendment_applies: TRUE`
([logs/dispatch_task_006.record.json](logs/dispatch_task_006.record.json)).

**Predecessors.** All five engine records are intact and `status.succeeded`, and all five
commits are on `origin/main`:

| Task | Engine record |
|---|---|
| TASK-001 | `bbda5d83…07e7` |
| TASK-002 | `c5fadc83…bd13` |
| TASK-003 | `5e4a83c1…56d0` |
| TASK-004 | `2b7406b8…c83f` |
| TASK-005 | `9554641f…7b9b` |

No predecessor deliverable or evidence file had changed at task start. The only untracked
files in a predecessor folder were the IDE's Gradle cache beside TASK-003's prototype.

## 4. Required reading

The task record, the Android operating model it names (A9 assigns TASK-006), and every
architecture document its v0.2 and v0.3 scope draws on were read in full:

- the workspace, persistent state, provider and agent, authority, LCL, import, handoff,
  settings, remote-control, manuals and system architecture documents;
- the reuse matrix and the import and handoff sources;
- the v0.3 coverage table;
- the charter policy's autonomy and specification modes.

The bindings B7, B9, B11, B17, B18, B20, B22 and B23, the contracts C2, C11, C15, C21 and C22
and the rules R11, R14 and R17 were read again. The rest of the mandatory read order is
byte-identical to earlier reads and was reused
([logs/required_reading.txt](logs/required_reading.txt)).

The repository records that assign work to TASK-006 were read in full:

- **docs/architecture:** SS-DOMAIN-PROTOCOL and its slots; the ten state records naming
  TASK-006 as a schema task (ST-CONTENT, ST-WORKSPACE, ST-SESSION, ST-TASK, ST-HISTORY,
  ST-LCL-REVISION, ST-IMPORT, ST-HANDOFF, ST-AUTHORITY, ST-DEVICE-TRUST); IF-REMOTE; RC-09;
  AD-04 to AD-08;
- **docs/security:** SI-12, TH-47 and TASK-004's contract types `type.sec_request_envelope`
  ("TASK-006 implements it as a domain type"), `type.sec_grant` and `type.sec_approval`, with
  the security bindings;
- **docs/dependencies:** DEP-SERDE, selected by TASK-003 for TASK-006 and TASK-007.

## 5. What was decided and built

**One crate, one module per slot.** Twelve slots were added to SS-DOMAIN-PROTOCOL in
`docs/architecture/subsystems.lcl.txt`, beside the six it already had. Five placeholders
became source under their stems (`errors`, `workspace`, `session`, `task`,
`model_capabilities`). `events` stays a placeholder: domain events are the payload of
TASK-007's event protocol, and no TASK-006 type needs them.

| Module | What it defines | Sources |
|---|---|---|
| `ids` | One identifier type per kind of identity, compared byte for byte; registry keys for closed registries owned elsewhere | SI-12, TH-47 |
| `text`, `time` | Bounded labels and notes without control characters; timestamps | RC-23 |
| `revision` | Content hashes; workspace and specification revisions that name their workspace; generations; permission epochs | C21, B17, B20, B23 |
| `schema` | `Record` (unique schema name, version, sync policy) and `Versioned<T>`, which refuses another schema, version or unknown field | TASK-006 security requirements, A9, C22 |
| `errors` | `DomainError`; the eight RC-12 outcomes with the binding's exact names; blockers | RC-12, C11 |
| `workspace` | Workspace records; per-device replicas with their roots and content availability; workspace-relative paths; conflicts keeping both sides | B18, A3, A6 |
| `device` | Device kinds; paired-device trust without keys or authority; host capabilities, apart from model capabilities | A4, A9, ST-DEVICE-TRUST |
| `client` | A UI client's foreground selection, presentation only | C2, R11, AD-06 |
| `session` | The agent session binding; model switch advancing the generation only; the execution transfer at an acknowledged checkpoint | B17, C21, A5, A7 |
| `task` | Tasks with the minimum task state; evidence tied to revisions and environment, including the closure records | B9, B20, C11, RC-05 |
| `model_capabilities` | Provider kinds, the session's provider and model, model capabilities | provider_and_agent_model |
| `lcl` | Specification modes; revisions from draft through validation to explicit adoption | B7, C22, I4 |
| `import` | The import transaction: target, base revision, staged hashes, mapping; commit only into the reviewed destination | B22, I1 to I5 |
| `handoff` | The model handoff with generation, checkpoint and read receipts; orientation **On by default** with a session override | B23, H1 to H4 |
| `authority` | Local permission grants; directional remote grants (`type.sec_grant`); single-use approvals (`type.sec_approval`) | ST-AUTHORITY, AD-04, RC-03, RC-17 |
| `remote_request` | The request envelope with every field of `type.sec_request_envelope`, generic over the operation's typed arguments | RC-09, SI-12 |

**How records are made safe.** A record whose fields must agree is a validated wrapper
around a plain `…Fields` struct: it exists only after the whole validation passed, and
decoding runs the same validation. Records whose fields are independent are plain structs
of validated values. Every struct rejects unknown fields; every closed set is an enum. State
changes are pure functions that return a new validated record or an error: the session's
model switch, the transfer's steps, the task's completion, the import's commit, the
handoff's phases and reads, the approval's single use, the request's freshness. None of them
does I/O, storage or transport.

**Choices made where the sources leave room, each documented in the code:**

- **The envelope is generic over its operation.** TASK-007 defines the operation registry by
  implementing `Operation` once per operation, declaring its target (workspace, agent or
  device) and whether it changes state. That lets the envelope enforce SI-12's explicit
  targets now, without inventing operations.
- **The contract's `expected_revisions` list of strings is typed:** each entry says what it is
  a revision of (workspace, specification, generation or the destination's policy).
- **Registries owned by later tasks are shape-checked keys only:** operations (TASK-007),
  grant capabilities (TASK-055), tool actions (TASK-022) and host capabilities (TASK-041).
- **A device's identity is its key fingerprint** (PD-TRANSPORT), as an opaque identifier
  whose exact derivation is TASK-060's.
- **Times are milliseconds since the Unix epoch.** The TASK-004 bindings for request lifetime
  (10 minutes) and clock skew (2 minutes) are constants of `remote_request`.

**Dependencies.** serde 1.0.229 for the domain types and serde_json 1.0.151 for their tests,
pinned exactly in the workspace (`=x.y.z`). Both were selected, licensed and proven by TASK-003
as DEP-SERDE, with TASK-006 as an owner, and were already in the isolated cargo registry: no
task command downloaded anything.

**The fix-forward of TASK-005's evidence rule** (section 11):

- `.gitignore` now ignores what Gradle and IDEs generate beside a Gradle prototype kept as
  evidence (`.gradle/`, `.kotlin/`, `build/`).
- The security stage of `run_checks.py` excuses an ignored evidence file only inside such a
  folder next to a Gradle build file of the repository; any other ignored evidence still
  fails. Two tooling tests and two negative cases prove both sides.
- CONVENTIONS.md section 7 documents it, and section 10 now defines the snapshot.

## 6. Deliverables

**New files:**

| Path | Purpose |
|---|---|
| `core/domain/{ids,text,time,revision,schema,errors,workspace,device,client,session,task,model_capabilities,lcl,import,handoff,authority,remote_request}.rs` | The 17 modules of section 5, each with its contract tests |
| `docs/evidence/TASK-006/` | This record, `logs/` and, written last, `receipt/` |

**Removed files:** the five placeholders now implemented under the same stems:
`core/domain/errors.source`, `core/domain/model_capabilities.source`,
`core/domain/session.source`, `core/domain/task.source` and `core/domain/workspace.source`.

**Changed files:**

| Path | Change |
|---|---|
| `core/domain/lib.rs` | The crate documentation and the module declarations |
| `core/domain/Cargo.toml`, `Cargo.toml` | serde for the crate, serde_json for its tests, pinned in the workspace |
| `Cargo.lock` | The 11 locked packages (section 11 says who wrote it and shows it is the task's own resolution) |
| `docs/architecture/subsystems.lcl.txt` | Twelve new slots of SS-DOMAIN-PROTOCOL; the `workspace` and `session` slots name what they now hold |
| `docs/dependencies/components.lcl.txt`, `LICENSE_MATRIX.md` | DEP-SERDE: pinned in the workspace by TASK-006; its transitive crates as the lockfile has them |
| `README.md` | The domain model, and the end of "no types yet" |
| `.gitignore`, `scripts/test/run_checks.py`, `tests/tooling/test_run_checks.py`, `docs/engineering/CONVENTIONS.md` | The fix-forward of section 5 |
| `FILE_TREE.txt` | Regenerated with the runner |

**New dependencies:** serde and serde_json, as above. Through them the lockfile holds
serde_core, serde_derive, proc-macro2, quote, syn 3, unicode-ident, itoa, memchr and zmij. Every
license is on the `deny.toml` allow list (MIT, Apache-2.0, Unlicense, Unicode-3.0), and
cargo-deny passes.

## 7. Checks run

| Command | Result | Log |
|---|---|---|
| `python3 -B scripts/test/run_checks.py`, all nine stages, and `cargo build -p nexees-domain` for both Android targets | All pass, exit 0 | [run_checks.txt](logs/run_checks.txt) |
| `cargo test --workspace` (inside the test stage) | 75 of 75 pass | [run_checks.txt](logs/run_checks.txt) |
| Tooling tests (inside the test stage) | 24 of 24 pass | [run_checks.txt](logs/run_checks.txt) |
| Negative tests: 21 seeded defects in scratch copies of the checkout, plus 2 controls | All 21 caught by the intended stage; the controls pass | [negative_tests.txt](logs/negative_tests.txt) |
| Regression: the charter, architecture, dependency and security projects, the TASK-001 to TASK-004 cross-checks and TASK-005's tooling tests | 22 commands, all exit 0 | [regression.txt](logs/regression.txt) |
| Native language suite | 51/51 | [native_language_suite.json](logs/native_language_suite.json) |
| What the owner's IDE did during the task | Recorded; the lockfile it wrote is byte-identical to the task's own offline resolution | [ide_activity.txt](logs/ide_activity.txt) |
| Secret and privacy scan | No finding | [security_scan.txt](logs/security_scan.txt) |
| Scoped cleanup with the TASK-005 tool | Nothing to remove; untracked files recorded as kept | [cleanup_record.json](logs/cleanup_record.json) |

The negative tests seed one defect per case. Fifteen break a domain rule, and the contract
tests catch each through named failing tests, never through a compile error:

- **Bindings:** a session on another workspace's checkout; a replica handing its root to
  another device; a root mapping accepting a smuggled authority field.
- **Handoff and orientation:** orientation off by default; a handoff ready before its reads;
  a model switch keeping the old generation.
- **Tasks and imports:** completion by evidence copied from another device; an import
  committed into a changed destination.
- **Authority:** an approval used twice; a request with an ambient workspace; a request run at
  its expiry; a remote grant allowed to travel.
- **Values and schemas:** an identifier accepting `/`; a path climbing out of its root; two
  records sharing one schema.

The other six cover the repository rules:
- an unpinned dependency;
- a module without a slot;
- unsafe code in the domain crate;
- an evidence log hidden in a `build/` folder;
- TASK-005's ignored-evidence rule, which still holds;
- a runner that excuses every ignored evidence file.

## 8. Check accounting

### Verification

| Check | How it was met |
|---|---|
| T006.VERIFY.01 v0.3 gate: contract-test IDs, revisions, target scope, ON default and serialization without duplicate stores | The contract tests cover each part. **IDs:** the identifier alphabet, byte-for-byte comparison, and decoding equal to construction (`ids`). **Revisions:** every record rejects a revision of another workspace, and generations and epochs never wrap (`revision`, `session`, `import`, `handoff`, `remote_request`). **Target scope:** workspace, agent and device targets with their revisions, and non-empty grant scopes (`remote_request`, `authority`). **ON default:** `OrientationSetting::default()` is On and a session override wins (`handoff`, `session`). **Serialization:** every record round-trips as `Versioned`, another schema name or version is refused, unknown fields are rejected, and a test proves that no two record types share a schema name, so no state has two stores (`schema`). |
| T006.VERIFY.02 build the affected targets | The workspace builds, lints with warnings denied, tests and documents on Rust 1.99.0. `nexees-domain` also builds for `aarch64-linux-android` and `x86_64-linux-android` ([run_checks.txt](logs/run_checks.txt)). |
| T006.VERIFY.03 task-specific tests | 75 contract tests, 24 tooling tests and 21 negative cases (section 7). |
| T006.VERIFY.04 regressions of the changed subsystem | The changed architecture and dependency records pass their LCL projects and the TASK-002 and TASK-003 cross-checks. The unchanged security project and charter pass theirs. TASK-005's changed runner passes its tooling tests ([regression.txt](logs/regression.txt)). |
| T006.VERIFY.05 final diff inspected | Section 6 lists every new, changed and removed file; the receipt's final verification repeats the inspection. |
| T006.VERIFY.06 v0.2 gate: round trips, invalid host/workspace/revision combinations, one project mapped to different device-local paths without new authority | **Round trips:** each record. **Invalid combinations rejected:** a session's checkout of another workspace; a transfer to the same device or from a foreign checkpoint; completion on another host or against a stale checkout; a handoff whose session, device, workspace, generation or specification no longer matches; an import whose base belongs to another workspace; a request whose workspace, agent or revisions do not fit its operation. **Same project, different roots:** one workspace maps to a PC root and to a phone root, and each root is refused on the other device. A replica or workspace record with a smuggled authority field is rejected, so a mapping grants nothing (`workspace`). |

### Completion gate

| Check | How it was met |
|---|---|
| T006.CLOSE.01 dependencies closed | TASK-005 accepted and its lineage verified (section 3). |
| T006.CLOSE.02 objective without unrelated scope | Every objective item has a type: workspace, agent session, task, project state, LCL state, permissions, approvals, providers and client state. So do the v0.2 identities and planes and the v0.3 import, handoff, orientation and closure records. No store, protocol, UI or behaviour was started. The fix-forward in TASK-005's tooling was needed for the gates to pass beside the owner's IDE, and section 11 records it. |
| T006.CLOSE.03 build passes | As T006.VERIFY.02. |
| T006.CLOSE.04 required tests pass | As T006.VERIFY.03. |
| T006.CLOSE.05 security checks pass | The security stage and the scan are clean, and cargo-deny passes. Every record is versioned and strictly validated; unknown and malformed authority-changing fields are rejected, as the negative cases show. Test fixtures hold no secrets or personal data. |
| T006.CLOSE.06 no unnecessary code or dependency | serde was already selected for this task (DEP-SERDE); serde_json is a test dependency. Each module fills a slot and each type cites its source. A redundant query helper found in review was removed. `events` is left to TASK-007, and no registry was invented. |
| T006.CLOSE.07 no workspace/agent/LCL binding invariant violated | Bound checkout, branch and remote. The pack, the LCL repository, the canonical packages and the LCL SDK were not written (final verification). One primary agent. The writes of the owner's IDE are disclosed (section 11). |
| T006.CLOSE.08 evidence recorded | This folder. |
| T006.CLOSE.09 Git diff understood | As T006.VERIFY.05. |
| T006.CLOSE.10 assigned v0.2 requirements pass; later scenarios traceable | A9's TASK-006 row: the device, replica, revision, host and capability identities exist, and control, sync and UI focus are separate types with declared sync policies, each proven by contract tests. AN-01 to AN-12 remain owned by the tasks of the charter's Android family and verified by TASK-074 and TASK-075. |
| T006.CLOSE.11 readability and comments | Agent's own review, section 9. |
| T006.CLOSE.12 scoped cleanup | Section 9; [logs/cleanup_record.json](logs/cleanup_record.json). |
| T006.CLOSE.13 post-cleanup verification on the final revision | `receipt/final_verification.txt`. |
| T006.CLOSE.14 manual impact | Section 9. |

## 9. Review, cleanup and manuals

**Readability (agent's own review).** Every module opens with what it is for, its boundaries
and the requirements it serves. Every public item is documented, as the lints require.
Comments state why a rule exists; tests name the behaviour they prove. Review against the
final code made these changes:
- a query helper that only repeated a field lookup was removed;
- a provider kind's wire name was corrected to `openai_compatible` before any record was
  stored;
- four module comments were made exact:
  - which records travel;
  - what a provider switch leaves unchanged;
  - whose focus a client's selection is;
  - where the provider catalogue lives;
- four over-long test and macro lines were wrapped.

**Cleanup.** TASK-006 created no temporary artifact inside the checkout:
- Cargo wrote to `/mnt/F/Nexees-toolchains/target/`;
- the negative-test copies and the lockfile regeneration used the session's scratch folder;
- every Python run used `-B`.

The cleanup tool ran on the real checkout with an empty manifest. It removed nothing and
recorded every untracked file it left in place, all of them TASK-006 deliverables and evidence
([logs/cleanup_record.json](logs/cleanup_record.json)). The owner's IDE output (`target/` and
the `.gradle/` cache) is ignored, not task-owned, and was left exactly as it is. The session
scratch folder outside the checkout is removed after the receipt. No pre-existing or
user-owned file was deleted, reset or stashed.

**Manuals.** Not applicable: TASK-006 changes no user-visible behaviour. The domain types
are internal; the behaviour that uses them, and its manual text, comes with the owning tasks.

## 10. Policy review and disclosures

All 135 rules of the nine policy documents were reviewed for this task.

| Policy document | Rules | Relation to TASK-006 |
|---|---|---|
| `policies/master_rules.lcl.txt` | 32 | R11 (focus never rebinds), R14 (state outside provider chat) and R17 (no secrets in state) are type rules here |
| `policies/global_contracts.lcl.txt` | 31 | C2, C11, C15, C21 and C22 are enforced by the bindings, the completion rule, the single shared crate and the sync policies |
| `policies/acceptance_criteria.lcl.txt` | 11 | Product acceptance, none due now; the domain types are what later acceptance runs on |
| `policies/code_readability_and_cleanup.lcl.txt` | 6 | Applied through the conventions; the closure records of RC-05 are now domain types too |
| `policies/no_unnecessary_code.lcl.txt` | 7 | One pre-selected dependency; no behaviour, store or invented registry; a redundant helper removed |
| `policies/reuse_policy.lcl.txt` | 5 | serde reused as TASK-003 selected it; no new package |
| `policies/security_baseline.lcl.txt` | 9 | Validated serialized inputs, versioned schemas, rejected unknown fields, fail-closed errors, unsafe code forbidden |
| `policies/usage_and_agents.lcl.txt` | 8 | One agent; Git writes only as CA-06 permits, with the slip in section 11 disclosed |
| `architecture/remote_device_control.lcl.txt` | 26 | No remote requirement is assigned to TASK-006; RC-09, RC-12 and RC-17 shape the envelope, outcome and approval types |

Disclosures:

- **Network.** No task command used the network apart from the freshness fetch of `main`
  from `origin`. The owner's IDE downloaded three crates into `~/.cargo` (section 11).
- **Session hook.** A hook asks for a dynamic web-application security scan after code
  changes. No Nexees web application is running and `HAWK_API_KEY` is unset, so none was run.
- **Owner-only state.** cargo-deny was executed from the owner's `~/.cargo/bin` without
  writing there. HopToDesk, linger, the autostart folder, the phone, the LCL SDK and the
  owner's VS Code settings were not touched.

## 11. Deviations and findings

- **The owner's IDE writes into the checkout and the home caches.** VS Code, opened on the
  checkout at 11:52, runs rust-analyzer and the Red Hat Java extension
  ([logs/ide_activity.txt](logs/ide_activity.txt)):
  - **`target/`:** rust-analyzer keeps its build output there. It is ignored, as for any Cargo
    project.
  - **`.gradle/`:** the Java extension imported TASK-003's Gradle prototype and wrote this
    cache beside it. TASK-005's last `.gitignore` rule re-included everything under
    `docs/evidence/`, so the cache became untracked "evidence" and the docs stage failed on
    `FILE_TREE.txt`.
  - **`Cargo.lock`:** when TASK-006 added serde to the manifests, rust-analyzer rewrote it
    within 1.5 seconds and downloaded serde_json, syn 3 and unicode-ident into `~/.cargo`.
    Regenerated independently, offline, with the task's isolated toolchain, the lock is
    byte-identical, so the committed lock is the task's own resolution whoever wrote it first.
  - **`~/.gradle`:** the Gradle daemons of the Java extension keep writing their logs there.
- **Fix-forward of TASK-005's evidence rule.** The owner's files were neither deleted nor
  hidden by settings. Generated Gradle and IDE folders beside a Gradle prototype are now
  ignored, and only they are excused (section 5).
- **The snapshot is now defined as what a commit holds** (CONVENTIONS.md section 10). This
  follows the pack's "content-based final worktree snapshot, including relevant dirty
  changes". TASK-002 to TASK-005 hashed every file on disk. Their checkouts held no ignored
  file, as their final verifications checked, so for them the two definitions agree.
- **A Git slip.** The implemented placeholders were removed with `git rm` and then unstaged
  with a path-limited `git reset -- core/domain/`. Nothing else was staged. The worktree,
  history and other paths were unaffected. CA-06 lists no reset, so this is disclosed;
  placeholders are removed with `rm` from now on.
- **TASK-004's cross-check counts an enforcement point as existing only while it is a
  `.source` placeholder.** Adding the new `core/domain/remote_request` and `core/domain/ids`
  slots to SI-12 and TH-47 made it fail, so those additions were withdrawn and `docs/security`
  is unchanged. TASK-007 will meet the same limit when `core/protocol/messages`, already an
  enforcement point, becomes source.
- **The negative tests needed three runs.**
  - The first run passed, but it counted a test-stage case by the failing exit code alone, and
    a compile error would have looked the same.
  - The second run required named failing tests. It showed that the scratch copies kept the
    files' modification times while sharing one Cargo target folder. Cargo therefore treated a
    copy as unchanged and reused a test binary built from an earlier, mutated copy. The control
    failed on such a binary, and the last case of the first run had also met one, among other
    failures.
  - The copies now get fresh modification times, so every case is built from its own sources.
    The third run is the one recorded.

  TASK-005's suite copied the same way. Its test-stage cases changed only Python files and
  expected the tooling tests to fail; a reused Rust binary could add failures there, not hide
  one.

## 12. Open items and limits

- **Who uses the types.** Nothing uses them at runtime yet:
  - TASK-007 adds the protocol messages and their wire schema, the operation registry, the
    canonical request hash and the domain events;
  - TASK-008 adds the stores and migrations;
  - the owning tasks named in each module add the behaviour.
- **Registries and formats owned elsewhere:**
  - operation keys: TASK-007;
  - grant capabilities: TASK-055;
  - tool actions: TASK-022;
  - host capabilities: TASK-041;
  - the derivation of a device's key fingerprint: TASK-060;
  - filesystem alias checks for paths (case, Unicode normalization): TASK-013 and TASK-051.
- **SI-12 and TH-47** still list only their protocol and remote enforcement points (section
  11). Fixing TASK-004's frozen cross-check is a decision for the task that first implements
  one of those points.
- **The owner's IDE.** Each Cargo manifest change will repeat rust-analyzer's downloads into
  `~/.cargo`. To prevent it, point rust-analyzer at the isolated toolchain, or have it ignore
  this folder; that is the owner's choice.
- **Also open:** the open items of `docs/dependencies` (OI-01 to OI-10), the open security
  decisions of `docs/security` (OSD-01 to OSD-08), BIND-B1-LOGO and AMEND-075-01, as their
  records state.

## 13. Next task

TASK-007 (versioned Desktop↔Android event protocol). It was not started.
