# TASK-008 evidence: Implement persistent state schema and migrations foundation

| | |
|---|---|
| Task | TASK-008 — Implement persistent state schema and migrations foundation |
| Date | 2026-10-04 |
| Performed by | Coding agent (Claude Code), one primary session; no sub-agents, reviewers or background AI jobs |
| Checkout | `/mnt/F/Nexees/`, branch `main`, base `1c85864e81ed5b8bfddaf5a8c4466a74a367232c` |
| Repository | `https://github.com/aivars11232/Nexees` |
| Specification pack | `/mnt/F/Nexees_LCL_Implementation_Pack_v0.5.3/`, archive revision 0.5.3, specification version 0.5.0 |
| Pack content identity | `bd5e8fce26f542c44a891e4340da3b2a84f79ab368b5cb868beb48126c2db3e1` |
| Pack manifest SHA-256 | `8d9d0080ac13a3fb17cc70603b77eb3b11abcd3fb6d2fad515cfac136fcf07e4` |
| Procedure | Continuation profile 0.5.3 (CA-01 to CA-12), as adopted for TASK-002 and continued for this task |
| Predecessors | TASK-001 (`284f97a`), TASK-002 (`d1a673f`), TASK-003 (`2eaa199`, receipt decided in `bccc9b1`), TASK-004 (`0337be0`), TASK-005 (`e83dcd7`), TASK-006 (`fc42a93`) and TASK-007 (`9f28c37`, follow-up `1c85864`), all accepted |
| Commit of this work | Made after acceptance, as CA-06 permits; it is recorded by Git, not in this file |

## Status

The deliverables are complete and every check passed on its final run. Acceptance is recorded
in [receipt/RECEIPT_RESULT.md](receipt/RECEIPT_RESULT.md), which is written after this file
because the receipt contains a snapshot of everything else.

TASK-008 built the state store in a new crate, `nexees-state` (`core/state`): each host's one
transactional store.
- **One store, one process.** SQLite 3.53.2, bundled through rusqlite 0.40.2 as TASK-003
  selected (DEP-SQLITE). It runs in WAL mode with full sync and in exclusive locking mode, so
  only the process that opened it can use it.
- **Durable before saved.** A write is one transaction, and it returns only once the commit is
  on disk. An error, a panic or a crash before the commit keeps nothing of it.
- **Versioned records, checked on read.** The store keeps the domain's records as strict JSON
  with their schema and version. It returns a record only when it decodes strictly, at its
  type's version, under its own key; anything else is an integrity error.
- **Migrations, all or nothing.** All pending steps run in one transaction. A step may not
  change any record it does not declare, and every record must decode again before the commit.
  A store newer than the build is refused untouched.
- **Recovery metadata.** Opening records how the previous session ended. Every effect that had
  started, and every remote request the host had admitted or was running, becomes
  outcome-unknown. The journal keeps at most one effect per remote request.
- **The outbox** keeps each peer replica's content changes in order until that peer
  acknowledges them. It holds content changes only.
- **No secrets.** The domain's records hold none, and the store refuses any field named like
  one.

27 contract tests prove these rules. Among them are crash tests in which a child process of the
test binary stops dead in the middle of a write or a migration; the crate's 28th test is those
children's entry point. 20 seeded defects show the tests catch what they claim to.
For RC-07, RC-17, RC-24 and RC-T09, RC-T10 and RC-T16, TASK-008 closes the part that storage
owns; section 8 names the later owners of the rest.

The owner's VS Code again rewrote `Cargo.lock` and downloaded crates when the manifests changed
(section 11). All review here is the agent's own. Under CA-05 the routine corroboration was done
by this session's own local tools; no independent human or model review has taken place.

## 1. Authority

The owner's instruction, verbatim:

```
if task 7 is really done, proceed with task 8
```

The owner then sent a screenshot of VS Code's Problems panel, with this message:

```
But from here i would say task 7 is not done
```

The panel showed a rustc error at `core/protocol/serialization.rs` line 297. It came from
rust-analyzer's last `cargo check`, run at 13:14:35 on an intermediate state of that file which
TASK-007 replaced at 13:15:02 (`target/flycheck0/stdout`). rust-analyzer checks again only when
a file is saved in the editor, so the panel kept the old result. The committed code passed
`cargo check --workspace --all-targets` with rustc 1.99.0, the same build the system has, and
all nine check stages. TASK-007 was therefore done (`origin/main` at `1c85864`, receipt accepted)
and TASK-008 began. The owner was told how to refresh the panel; their VS Code was not touched.

This continues the procedure the owner adopted with pack 0.5.3 (CA-01): one named task through
evidence, receipt acceptance, commit and push, then stop. TASK-009 is not covered.

## 2. Checkout preflight (binding B26)

| Item | Observed |
|---|---|
| Working directory | `/mnt/F/Nexees/`; the Git worktree root, with no parent repository, superproject or second worktree |
| `origin` fetch and push | `https://github.com/aivars11232/Nexees.git`; no URL rewrite, no separate push URL |
| Branch and HEAD | `main` at `1c85864`, equal to `origin/main` after a fetch of that branch only, without tags |
| Hooks | None besides Git's samples; `core.hooksPath` unset |
| At task start | Clean: no staged, modified or untracked file, no stash, 532 tracked files. The owner's IDE keeps ignored output in the checkout: 262 files under `target/` and 17 under TASK-003's Gradle prototype (`.gradle/`). rust-analyzer's last check, from 13:14:35, still held the one stale error of section 1. All of it is recorded as pre-existing, not task-owned state. |
| Result | Passes ([logs/preflight.txt](logs/preflight.txt)) |

TASK-008 began at 2026-10-04T14:12:44+02:00, when the owner's instruction arrived. That time is
the baseline of every "nothing written since" check.

## 3. Pack, native verification, dispatch and predecessors

**Pack.** All 201 manifest entries match their SHA-256, with no unlisted or missing file.
The content identity recomputes to `bd5e8fce…b3e1`. `tasks/task_008.lcl.txt` (`ce314300…8258`)
equals the `after_sha256` of `PROCEDURAL_CHANGES.json`.

**Native verification.** The pack's runner passed all 51 synthetic language cases with
`lcl 0.9.1` ([logs/native_language_suite.json](logs/native_language_suite.json)). These are
language tests, not evidence for any task.

**Dispatch.** `lcl run … --input input.task_number=8 main.lcl.txt` ran `task.dispatch` and
`task.task_008` only, with `status.succeeded` and no diagnostics. It published the TASK-008
packet (SHA-256 of the packet text `35ad4f2f…450f`), with `procedure_amendment_applies: TRUE`
([logs/dispatch_task_008.record.json](logs/dispatch_task_008.record.json)).

**Predecessors.** All seven engine records are intact and `status.succeeded`, and all their
commits are on `origin/main`:

| Task | Engine record |
|---|---|
| TASK-001 | `bbda5d83…07e7` |
| TASK-002 | `c5fadc83…bd13` |
| TASK-003 | `5e4a83c1…56d0` |
| TASK-004 | `2b7406b8…c83f` |
| TASK-005 | `9554641f…7b9b` |
| TASK-006 | `b10ab5d9…9f36` |
| TASK-007 | `3e0ce66c…19e8` |

At task start no predecessor deliverable or evidence file had an uncommitted change; every
change since their acceptance is a later accepted commit.

## 4. Required reading

The task record was read in full. These pack documents were read again for what the store must
keep and guarantee ([logs/required_reading.txt](logs/required_reading.txt)):

- the persistent state model, in full: its requirements and the v0.2 and v0.3 records;
- the Android operating model: A3, A6, A7, AN-08, and A9, which assigns TASK-008 the durable
  stores, outbox, revision and conflict metadata and the migration contracts;
- RC-07, RC-17 and RC-24, and the scenarios RC-T09, RC-T10, RC-T16 and RC-T11;
- R14, R17, C11, C12, C21 and C22; the bindings B9, B20, B22 and B23; the stop conditions;
- the import pipeline and the model handoff, for the records the store keeps.

The rest of the mandatory read order is byte-identical to earlier reads and was reused.

The repository records that assign work to TASK-008 were read in full:

- **docs/architecture:**
  - SS-STATE, its slots, owners and test slot;
  - SS-SYNC, SS-UPDATES, SS-SETTINGS, SS-SECURITY and SS-REMOTE, whose entities persist through
    the store;
  - IF-STORE; ST-WORKSPACE, ST-JOURNAL, ST-REMOTE-REQUEST and ST-SYNC, whose schema tasks include
    TASK-008; FD-STORAGE; PD-PERSISTENCE and its resolution;
- **docs/security:**
  - the remote contracts of RC-07, RC-17 and RC-24;
  - `type.sec_receiver_settings`, `type.sec_grant` and `type.sec_audit_record`;
  - SI-10, SI-11, SI-13, SI-15, SI-25 and SI-26;
  - TH-22, TH-39, TH-40 and TH-46, which name the operation journal and the migration registry
    as enforcement points;
- **docs/dependencies:** DEP-SQLITE and the deny policy;
- **docs/evidence/TASK-003:** the SQLite feasibility store and its crash and migration evidence,
  as a reference;
- **docs/engineering/CONVENTIONS.md:** sections 3, 6 and 9;
- **core/domain and core/protocol:** the records the store keeps, and the restart rule of the
  request ledger.

## 5. What was decided and built

**Slots.** Two slots were added to SS-STATE in `docs/architecture/subsystems.lcl.txt`:
`core/state/Cargo` and `core/state/lib`. Three placeholders became source under their stems:
`core/state/state_store`, `core/state/migrations/migration_registry` and
`core/state/operation_journal`. `checkpoints` and `recovery` stay placeholders, for TASK-026
and TASK-038.

| Module | What it defines | Sources |
|---|---|---|
| `state_store` | Opening, closing and writing the store; the 19 domain record types it keeps, by stable key; reads checked against schema, version and key; the outbox; the integrity check; the recovery metadata; the files it owns | SS-STATE, IF-STORE, A3, R17, persistent_state_model |
| `migrations/migration_registry` | The schema version, the ordered steps, the plan for an older or newer store, and the guarded all-or-nothing upgrade | FD-STORAGE, TH-40, SI-26, SS-UPDATES |
| `operation_journal` | Journal entries with their states, the restart rules for effects and remote requests, and at most one effect per remote request | ST-JOURNAL, RC-17, SI-13, TH-46, TH-22 |

**The tables of schema version 1:**

| Table | Holds |
|---|---|
| `store_meta` | The session marker, and a summary of the latest open: how the previous session ended, any migration and what the restart rules changed |
| `migration_history` | Each version applied, and when |
| `records` | One row per record: schema, key, version and the strict JSON body |
| `outbox` | Content changes per peer replica, in order |
| `journal` | Journaled effects, with their remote request and state |

**Choices made where the sources leave room, each documented in the code:**

- **One table of versioned records.** The domain records already carry their validation and
  version, so the store keeps them as strict JSON in one table keyed by schema and key, not as
  one table per entity. An owning task that needs a query adds its index or table through a
  migration (TASK-036 for the project state).
- **The schema version covers the record set.** A test pins the 19 record schemas and versions
  of version 1. A new record type, or a changed version, needs a migration step, so an older
  build refuses the store instead of misreading it.
- **Migrations all at once.** TASK-003's prototype committed each step separately. Here every
  pending step runs in one transaction, so a failure leaves the store at a version the previous
  build can still open: the recoverable path SS-UPDATES relies on.
- **A migration may not change what it does not declare.** Each step lists the record schemas
  it rewrites. Every other record, outbox entry and journal entry must come out byte-identical,
  or the upgrade is refused (TH-40). Every record must also decode strictly afterwards, and no
  domain record's field has a default. So a field that goes missing in a migration fails the
  upgrade instead of coming back as a default, least of all as an allow (SI-26, RC-24).
- **One process.** Exclusive locking mode, beyond TASK-003's prototype, keeps a second process
  out while a host holds the store. That makes the restart rules safe to apply when it opens.
- **Restart rules on every open.** An effect that had started, or a remote request that had
  been admitted or was running, may or may not have happened. That is so after any restart,
  as the domain's ledger rule says, not only after a crash.
- **The journal records the session's generation** with each effect, so an effect started
  under an older model can be told apart (TH-22). A wall clock that moved backwards cannot
  settle an effect before it started, nor stop the store from opening.
- **The outbox is per destination replica.** Queuing the same change again is idempotent;
  another change under a queued ID is refused; an acknowledgment removes the entry.
- **Secrets.** The guarantee is the domain types, which have no secret field. The store adds a
  second line: it refuses a body with a field named like a secret, such as `api_key` or
  `password`.
- **No platform port.** The host passes the store's path; choosing the app-private location on
  each platform is the hosts' task (TASK-009 and TASK-057).

**Dependencies.** rusqlite 0.40.2 with its bundled SQLite is pinned exactly in the workspace.
TASK-003 selected, licensed and proved it as DEP-SQLITE, with TASK-008 as an owner, and every
crate it needs was already in the isolated cargo registry. The lockfile also lists the crates
rusqlite uses only on WebAssembly, which no Nexees target builds. cargo-deny passes for the
targets that are built. The root manifest's comment on serde_json, which still said "in tests"
after TASK-007, now names its two uses.

## 6. Deliverables

**New files:**

| Path | Purpose |
|---|---|
| `core/state/Cargo.toml`, `core/state/lib.rs` | The crate `nexees-state` and its documentation |
| `core/state/state_store.rs`, `core/state/operation_journal.rs`, `core/state/migrations/migration_registry.rs` | The modules of section 5, each with its contract tests |
| `docs/evidence/TASK-008/` | This record, `logs/` and, written last, `receipt/` |

**Removed files:** the three placeholders now implemented under the same stems:
`core/state/state_store.source`, `core/state/operation_journal.source` and
`core/state/migrations/migration_registry.source`.

**Changed files:**

| Path | Change |
|---|---|
| `Cargo.toml` | `core/state` joins the workspace members; rusqlite pinned; the serde_json comment corrected |
| `Cargo.lock` | The `nexees-state` entry and rusqlite's tree (section 11 says who wrote it) |
| `docs/architecture/subsystems.lcl.txt` | The two new slots of SS-STATE |
| `docs/dependencies/components.lcl.txt`, `docs/dependencies/LICENSE_MATRIX.md` | DEP-SQLITE: pinned in the workspace by TASK-008; its transitive crates as the lockfile has them |
| `docs/engineering/CONVENTIONS.md` | Section 3 names the three crates |
| `README.md` | The state store |
| `FILE_TREE.txt` | Regenerated with the runner |

## 7. Checks run

| Command | Result | Log |
|---|---|---|
| `python3 -B scripts/test/run_checks.py`, all nine stages, and `cargo build -p nexees-domain -p nexees-protocol -p nexees-state` for both Android targets | All pass, exit 0 | [run_checks.txt](logs/run_checks.txt) |
| `cargo test --workspace` (inside the test stage) | 91 domain, 30 protocol and 28 state tests pass | [run_checks.txt](logs/run_checks.txt) |
| Tooling tests (inside the test stage) | 26 of 26 pass | [run_checks.txt](logs/run_checks.txt) |
| Negative tests: 20 seeded defects in scratch copies of the checkout, plus 2 controls | All 20 caught by the intended stage; the controls pass | [negative_tests.txt](logs/negative_tests.txt) |
| Regression: the charter, architecture, dependency and security projects, the TASK-001 to TASK-004 cross-checks, TASK-006's and TASK-007's contract tests and the tooling tests | 23 commands, all exit 0 | [regression.txt](logs/regression.txt) |
| Native language suite | 51/51 | [native_language_suite.json](logs/native_language_suite.json) |
| What the owner's IDE did during the task | Recorded; the lockfile it wrote is byte-identical to the task's own offline resolution | [ide_activity.txt](logs/ide_activity.txt) |
| Secret and privacy scan | No finding | [security_scan.txt](logs/security_scan.txt) |
| Scoped cleanup with the TASK-005 tool | Nothing to remove; untracked files recorded as kept | [cleanup_record.json](logs/cleanup_record.json) |

The crash tests run a child process of the test binary that stops dead at its crash point, as
a killed host does: no destructor runs and the program rolls back nothing. The parent then
reopens the store. They cover:
- a write before and after its commit;
- stopping remote access halfway;
- a running remote request with its effect journaled;
- a migration step.

The negative tests seed one defect per case. Seventeen break a rule of the state store, and the
contract tests catch each through named failing tests, never through a compile error:

- **Migrations:** a step allowed to change undeclared records; an upgrade committed without
  decoding every record again; a newer store opened and written to; a record's version raised
  without a migration step. The domain pins every record's version as well, and its failing
  test stops the run before the store's tests. In the suite's first run that case was caught by
  the domain's pin alone, so the case now moves that pin along too. The suite was then run again
  in full, and the store's own pin catches the case.
- **Records:** a record read back under another key, or at another version; a field named like
  a secret stored; an old task ID reused by an insert; another change queued under a queued ID.
- **The store:** a second process allowed in; commits that survive a crash but not a power
  loss; an interrupted session recorded as closed.
- **The journal:** started effects or running remote requests left as they were after a
  restart; a second effect for the same request; a completed effect settled again; an effect
  settled before it started after the clock moved back.

The other three cover the repository rules: rusqlite not pinned exactly, a state module without
a slot, and unsafe code in the state crate.

## 8. Check accounting

### Verification

| Check | How it was met |
|---|---|
| T008.VERIFY.01 v0.3 gate: crash/migration recovery and safe defaults without treating old task IDs as new completed tasks | **Crash recovery:** a write cut off before its commit leaves the last committed state, and one after it keeps the write; the next open notes the interrupted session and checks the whole store. **Migration recovery:** a failing step, a crash during a step, a step that changes undeclared records and a step that leaves a record undecodable each leave the store at its previous version with every record unchanged. After the failing step the build that wrote the store still opens it; after the crash the next open migrates normally. **Safe defaults:** a new store holds no grant and no peer; a newer store is refused untouched; a migration that drops a field is refused, because no stored field has a default. **Old task IDs:** an insert never reuses a key. Tasks keep their status when a session ends without closing the store and through a migration; the store never sets a status (`state_store`, `migration_registry`). |
| T008.VERIFY.02 build the affected targets | The workspace builds, lints with warnings denied, tests and documents on Rust 1.99.0. `nexees-state`, with SQLite's own sources, builds for `aarch64-linux-android` and `x86_64-linux-android`, as do the other two crates ([run_checks.txt](logs/run_checks.txt)). |
| T008.VERIFY.03 task-specific tests | 28 state tests and 20 negative cases (section 7). |
| T008.VERIFY.04 regressions of the changed subsystem | The changed architecture and dependency records pass their LCL projects and the TASK-002 and TASK-003 cross-checks. The unchanged security project and charter pass theirs; TASK-004's cross-check finds the two enforcement points TASK-008 implemented. TASK-006's and TASK-007's contract tests and the tooling tests pass ([regression.txt](logs/regression.txt)). |
| T008.VERIFY.05 final diff inspected | Section 6 lists every new, changed and removed file; the receipt's final verification repeats the inspection. |
| T008.VERIFY.06 v0.2 gate: fixtures for crash-safe write and recovery and for the migration of unsynced local changes; expired control approvals never become replayable content-queue entries | **Fixtures:** the crash children of section 7, and a populated store holding queued changes, a draft LCL revision, a staged import, tasks and security records. A migration keeps every one of them byte for byte. **Approvals:** the outbox takes content changes only, by type; approvals and pending requests are records that never travel. In the RC-T10 test an expired queued request becomes expired while the outbox holds only the content change. In the RC-07 test the approvals of the old epoch fail after the stop. |

### Completion gate

| Check | How it was met |
|---|---|
| T008.CLOSE.01 dependencies closed | TASK-007 accepted and its lineage verified (section 3). |
| T008.CLOSE.02 objective without unrelated scope | **Objective:** versioned persistence primitives (versioned records, schema version), transactional writes, migration hooks (the registry), integrity checks (on every read, and in full after a migration or a crash) and recovery metadata (the session marker, the open summary, the journal and the restart rules). **v0.3:** import transactions, handoffs with their orientation setting, sessions with their orientation override and task evidence are stored records from the first schema version. A field added to them later comes with a migration step that declares the records it rewrites, and a migration keeps drafts, staged imports and unsynced changes byte for byte (tested). **v0.2:** durable Android-local state (the store builds for Android); outgoing content changes (the outbox); revision and base hashes (each change keeps its base hash and origin revision); deletions and conflicts (tombstones are delete changes; conflict records are kept); interrupted action recovery (the journal and the restart rules); no secrets in ordinary state. No checkpoint, project-state query, sync exchange or UI was started. |
| T008.CLOSE.03 build passes | As T008.VERIFY.02. |
| T008.CLOSE.04 required tests pass | As T008.VERIFY.03. |
| T008.CLOSE.05 security checks pass | The security stage and the scan are clean, and cargo-deny passes. Stored bodies are versioned and decoded strictly on every read, so an unknown field is an integrity error, never a returned record; a body with a field named like a secret is never written. Errors name the record they concern and never quote a stored body. Test fixtures hold no secrets or personal data; the one credential-shaped test value is assembled at run time. |
| T008.CLOSE.06 no unnecessary code or dependency | The dependency was selected for this task by TASK-003. One table of versioned records instead of one per entity, and no platform port. Review fixed three small issues, and the tests brought out two rules that the code now states (section 9). |
| T008.CLOSE.07 no workspace/agent/LCL binding invariant violated | Bound checkout, branch and remote. The pack, the LCL repository, the canonical packages and the LCL SDK were not written (final verification). One primary agent. The writes of the owner's IDE are disclosed (section 11). |
| T008.CLOSE.08 evidence recorded | This folder. |
| T008.CLOSE.09 Git diff understood | As T008.VERIFY.05. |
| T008.CLOSE.10 assigned v0.2 requirements pass; later scenarios traceable | A9's TASK-008 row, "durable mobile stores/outbox, revision/conflict metadata and crash-safe migration contracts", is met by the store, the outbox and the migration registry, proven by contract tests. The remote requirements and scenarios below are closed for their storage part, and each names its later owners. |
| T008.CLOSE.11 readability and comments | Agent's own review, section 9. |
| T008.CLOSE.12 scoped cleanup | Section 9; [logs/cleanup_record.json](logs/cleanup_record.json). |
| T008.CLOSE.13 post-cleanup verification on the final revision | `receipt/final_verification.txt`. |
| T008.CLOSE.14 manual impact | Section 9. |

### Remote requirements and scenarios

Each row says what TASK-008 closed, with an implementation and its tests, and which later tasks
of the pack's requirement map own the rest. Nothing here is runtime-tested end to end.

| Check | Requirement | What TASK-008 closed | Later owners |
|---|---|---|---|
| T008.RC.07 | RC-07 settings and immediate stop | The stored part of the stop is one transaction: every grant's epoch moves on together, or none does, even when the host dies halfway (crash test). Approvals of the old epoch then fail their check, and local editing keeps writing. The receiver settings, the Settings view, closing the sessions and the audit record of the stop are their owners'; they persist through this store. | TASK-022, TASK-030, TASK-055, TASK-056, TASK-060, TASK-067, TASK-068, TASK-070, TASK-071, TASK-073, TASK-074, TASK-075 |
| T008.RC.17 | RC-17 replay and unknown outcomes | The destination's ledger and the sender's pending requests are stored durably. The restart rules make admitted or running requests and started effects outcome-unknown, the journal holds at most one effect per request, and the outbox holds content only. Journaling at each effect's call site, the reconciliation exchange and approval revalidation are their owners'. | TASK-026, TASK-031, TASK-033, TASK-035, TASK-038, TASK-055, TASK-066, TASK-067, TASK-069, TASK-070, TASK-071, TASK-073, TASK-074, TASK-075 |
| T008.RC.24 | RC-24 update, logout and removal | A migration carries every record over byte for byte unless it declares the change, and a field that goes missing fails it instead of taking a default. Grants, revocations, epochs and peers survived an update unchanged in the test. A newer store is refused. The store's own files are listed for removal, and user projects live outside them. The update service, sign-out, startup registration and uninstalling are their owners'. | TASK-009, TASK-026, TASK-030, TASK-035, TASK-038, TASK-056, TASK-067, TASK-069, TASK-070, TASK-071, TASK-073, TASK-074, TASK-075 |
| T008.RCT.09 | RC-T09 lost acknowledgment | A crash test: the host admits and runs a request, journals its effect and dies. Reopened, the request and the effect are outcome-unknown. A resend is answered with that outcome, a second effect is refused, and only evidence settles it. The exchange between two devices is its owners'. | TASK-026, TASK-031, TASK-035, TASK-038, TASK-066, TASK-069, TASK-070, TASK-073, TASK-074, TASK-075 |
| T008.RCT.10 | RC-T10 expired queued request | A test: a request queued while disconnected survives a restart. After its expiry it is dropped as expired, the queued content change still waits for the peer, and nothing else is in the outbox. The sync exchange and the remote runtime are their owners'. | TASK-069, TASK-070, TASK-073, TASK-074, TASK-075 |
| T008.RCT.16 | RC-T16 update, restart and uninstall | A test in a disposable profile: an update migrates the store, a restart reopens it, and uninstalling removes only the store's own files. The user's project is untouched, and grants, revocations, epochs and peers are kept exactly. An obsolete peer's stored versions still let negotiation refuse it under a raised minimum. Real packages, updates and startup entries are their owners'. | TASK-026, TASK-038, TASK-070, TASK-073, TASK-074, TASK-075 |

## 9. Review, cleanup and manuals

**Readability (agent's own review).** Every module of `core/state` opens with what it is for,
its boundaries and the requirements it serves. Every public item is documented, as the lints
require. Comments state why a rule exists; tests name the behaviour they prove. Review against
the final code made these changes:
- reading the outbox now reports a negative stored time as an integrity error, instead of
  turning it into a wrong time;
- the recovery metadata names the previous session explicitly instead of through Rust's debug
  formatting;
- one doc comment was corrected.

The tests found two rules that the code and its comments now state:
- **The restart rules apply on every open.** A crash scenario had stored an admitted request,
  closed the store and expected the request to run later; on reopening it was outcome-unknown.
  That is the domain's rule after any restart, so the scenario was wrong, not the store.
- **A wall clock that moves backwards** between sessions made a journal entry settle before it
  started, which its validation refused, and that stopped the store from opening. An entry now
  settles no earlier than it started.

**Cleanup.** TASK-008 created no temporary artifact inside the checkout:
- Cargo wrote to `/mnt/F/Nexees-toolchains/target/`;
- the tests' stores lived in the system temporary folder, each removed when its test ended,
  and none was left behind;
- the negative-test copies and the lockfile regeneration used the session's scratch folder;
- every Python command that imports a module of the checkout ran with `-B`, so no bytecode was
  written there.

The cleanup tool ran on the real checkout with an empty manifest. It removed nothing and
recorded every untracked file it left in place, all of them TASK-008 deliverables and evidence
([logs/cleanup_record.json](logs/cleanup_record.json)). The owner's IDE output (`target/` and
the `.gradle/` cache) is ignored, not task-owned, and was left exactly as it is. The session
scratch folder outside the checkout is removed after the receipt. No pre-existing or
user-owned file was deleted, reset or stashed.

**Manuals.** Not applicable: TASK-008 changes no user-visible behaviour. The store is internal
to the hosts; the behaviour that uses it, and its manual text, come with the owning tasks.

## 10. Policy review and disclosures

All 135 rules of the nine policy documents were reviewed for this task.

| Policy document | Rules | Relation to TASK-008 |
|---|---|---|
| `policies/master_rules.lcl.txt` | 32 | R14 (state outside a provider chat) is the store's purpose; R17 (no secrets in state) is kept by the record types and the store's guard |
| `policies/global_contracts.lcl.txt` | 31 | C12 (recovery) through the journal and all-or-nothing migrations; C22 (no replay from sync) through an outbox that holds content only |
| `policies/acceptance_criteria.lcl.txt` | 11 | Product acceptance, none due now; AN-08 and AN-12 build on these contracts |
| `policies/code_readability_and_cleanup.lcl.txt` | 6 | Applied through the conventions; the review findings are in section 9 |
| `policies/no_unnecessary_code.lcl.txt` | 7 | One store per host, one table of records, no platform port, the selected dependency only |
| `policies/reuse_policy.lcl.txt` | 5 | SQLite through rusqlite reused as TASK-003 selected and proved it |
| `policies/security_baseline.lcl.txt` | 9 | Validated, versioned stored records; fail-closed reads, migrations and opening; no secrets; unsafe code forbidden |
| `policies/usage_and_agents.lcl.txt` | 8 | One agent; Git writes only as CA-06 permits; placeholders removed with `rm` |
| `architecture/remote_device_control.lcl.txt` | 26 | RC-07, RC-17 and RC-24 are assigned to TASK-008 and accounted for in section 8 |

Disclosures:

- **Network.** No task command used the network apart from fetching `main` from `origin` and,
  after acceptance, pushing to it. The owner's IDE downloaded eight crates into `~/.cargo`
  (section 11).
- **Session hook.** A hook asks for a dynamic web-application security scan after code changes.
  No Nexees web application is running and `HAWK_API_KEY` is unset, so none was run.
- **Owner-only state.** cargo-deny was executed from the owner's `~/.cargo/bin` without writing
  there. HopToDesk, linger, the autostart folder, the phone, the LCL SDK, the owner's
  `lcl-remote` service and the owner's VS Code and its settings were not touched.
- **Scripts.** TASK-007's evidence scripts, removed with its scratch folder, were rebuilt from
  this session's transcript and adapted for TASK-008; they are tools, not deliverables. Two
  replayed steps had also edited TASK-007's evidence record; only their script halves were
  replayed.

## 11. Deviations and findings

- **The owner's IDE wrote into the checkout and the home caches**
  ([logs/ide_activity.txt](logs/ide_activity.txt)).
  - **`Cargo.lock`:** one command of the task wrote `core/state/Cargo.toml` and the workspace
    manifest; it ended at 14:39:29.9. The task's next command, its first cargo build, began at
    14:39:37.0. In between, at 14:39:32.5, rust-analyzer rewrote `Cargo.lock` with the new crate
    and rusqlite's tree. Regenerated independently, offline, with the task's isolated toolchain,
    the lock is byte-identical.
  - **`~/.cargo`:** at 14:39:30–32 rust-analyzer refreshed its copy of the crates.io index and
    downloaded rusqlite, libsqlite3-sys, bitflags, cc, find-msvc-tools, hashlink, pkg-config and
    smallvec. The task's own builds used the isolated cargo home, where every one of these crates
    already was.
  - **`target/` and `~/.gradle`:** rust-analyzer's build output and the logs of VS Code's Gradle
    daemons, as in earlier tasks. VS Code's idle Gradle 8.9 daemon also stopped itself, at
    14:54:40, after 180 minutes without a build; as it stopped, it released its cache locks and
    left the daemon registry. VS Code's import of TASK-003's Gradle prototype keeps failing because
    its Gradle 9.2.0 is older than the prototype's Android plugin needs. That is the owner's IDE,
    and nothing of the task builds that prototype.
- **The stale error the owner reported** is explained in section 1. Nothing needed to change.
- **The two rules the tests found** are in section 9.

## 12. Open items and limits

- **Who builds on the store.** Nothing runs on it yet:
  - checkpoints and the audit trail: TASK-026;
  - the project-state entities, with the queries and indexes they need: TASK-036;
  - structured history: TASK-037;
  - crash-safe resume from the journal: TASK-038;
  - the receiver settings and the other settings: TASK-030;
  - the replicas' base revisions and content hashes, the transfer journal and the sync exchange:
    TASK-069;
  - the remote runtime: TASK-070;
  - the store's location on each platform: TASK-009 and TASK-057.
- **The remote requirements** keep the parts section 8 assigns to later owners.
- **The test slot `tests/unit/state_recovery`** stays a placeholder for TASK-038. This crate's
  unit tests sit beside its code, as CONVENTIONS.md section 9 says.
- **The owner's IDE.** Each Cargo manifest change makes rust-analyzer rewrite `Cargo.lock` and
  download into `~/.cargo`. Pointing it at the isolated toolchain, or excluding this folder,
  would stop that; it is the owner's choice.
- **Also open:** the open items of `docs/dependencies` (OI-01 to OI-10), the open security
  decisions of `docs/security` (OSD-01 to OSD-08), BIND-B1-LOGO and AMEND-075-01, as their
  records state.

## 13. Next task

TASK-009 (Create Desktop application shell). It was not started.
