# TASK-007 evidence: Define versioned Desktop↔Android/event protocol

| | |
|---|---|
| Task | TASK-007 — Define versioned Desktop↔Android/event protocol |
| Date | 2026-10-04 |
| Performed by | Coding agent (Claude Code), one primary session; no sub-agents, reviewers or background AI jobs |
| Checkout | `/mnt/F/Nexees/`, branch `main`, base `fc42a93bf9f62f662ba7a1bb61d5e662b1812ada` |
| Repository | `https://github.com/aivars11232/Nexees` |
| Specification pack | `/mnt/F/Nexees_LCL_Implementation_Pack_v0.5.3/`, archive revision 0.5.3, specification version 0.5.0 |
| Pack content identity | `bd5e8fce26f542c44a891e4340da3b2a84f79ab368b5cb868beb48126c2db3e1` |
| Pack manifest SHA-256 | `8d9d0080ac13a3fb17cc70603b77eb3b11abcd3fb6d2fad515cfac136fcf07e4` |
| Procedure | Continuation profile 0.5.3 (CA-01 to CA-12), as adopted for TASK-002 and continued for this task |
| Predecessors | TASK-001 (`284f97a`), TASK-002 (`d1a673f`), TASK-003 (`2eaa199`, receipt decided in `bccc9b1`), TASK-004 (`0337be0`), TASK-005 (`e83dcd7`) and TASK-006 (`fc42a93`), all accepted |
| Commit of this work | Made after acceptance, as CA-06 permits; it is recorded by Git, not in this file |

## Status

The deliverables are complete, and every check passed on its final run except one: TASK-004's
frozen cross-check fails one assertion, as expected (section 11). Acceptance is recorded in
[receipt/RECEIPT_RESULT.md](receipt/RECEIPT_RESULT.md), which is written after this file because
the receipt contains a snapshot of everything else.

TASK-007 defined the Nexees protocol in a new crate, `nexees-protocol` (`core/protocol`), on top
of TASK-006's domain types:

- **Versioned channels.** Every channel opens with a hello. Both ends agree on the newest
  version they share, and only if it is at least the minimum secure version. A mismatch is
  refused, never downgraded.
- **Bounded, strict messages.** A message is measured before it is parsed. Unknown, duplicate
  and malformed fields are refused, and a refusal never quotes the input.
- **A closed operation registry.** A peer may request nine operations, and the local user four
  more. Each has typed arguments, explicit targets and expected revisions.
- **Separate families.** Window↔host, remote control and content synchronization are
  different message types. The synchronization family cannot carry a request, an approval or a
  grant.
- **No duplicate execution.** The destination's ledger answers a repeated request ID with the
  recorded outcome. After a lost acknowledgment, the sender reconciles by request ID and resends
  only what never arrived, and only before it expires. After a restart, an effect that may have
  started is outcome-unknown.
- **Reconnect without side effects.** Events are numbered and never applied twice or across a
  gap. Execution-transfer steps apply only in order, so an interrupted handoff never yields two
  executors.

Three domain modules were added for the protocol to carry: domain events, the request ledger
and pending requests, and content changes.

This is the design, contracts, types and test plan the task's remote evidence scope asks for.
Nothing is sent, stored or executed yet. No remote requirement is implemented or runtime-tested;
section 8 records what TASK-007 closed for each one and which later tasks own the rest.

All review here is the agent's own. Under CA-05 the routine corroboration was done by this
session's own local tools; no independent human or model review has taken place.

## 1. Authority

The owner's instruction, verbatim:

```
proceed with task 7
```

TASK-006 was accepted and pushed before this task began (`e83dcd7..fc42a93` on `origin/main`).
This continues the procedure the owner adopted with pack 0.5.3 (CA-01): one named task through
evidence, receipt acceptance, commit and push, then stop. TASK-008 is not covered. No owner
question was needed during the task.

## 2. Checkout preflight (binding B26)

| Item | Observed |
|---|---|
| Working directory | `/mnt/F/Nexees/`; the Git worktree root, with no parent repository, superproject or second worktree |
| `origin` fetch and push | `https://github.com/aivars11232/Nexees.git`; no URL rewrite, no separate push URL |
| Branch and HEAD | `main` at `fc42a93`, equal to `origin/main` after a fetch of that branch only, without tags |
| Hooks | None besides Git's samples; `core.hooksPath` unset |
| At task start | Clean: no staged, modified or untracked file, no stash, 508 tracked files. The owner's IDE keeps ignored output in the checkout: 198 files under `target/` and 17 under TASK-003's Gradle prototype (`.gradle/`). It is recorded as pre-existing, not task-owned state. |
| Result | Passes ([logs/preflight.txt](logs/preflight.txt)) |

TASK-007 began at 2026-10-04T12:55:58+02:00, when the owner's instruction arrived. That time is
the baseline of every "nothing written since" check.

## 3. Pack, native verification, dispatch and predecessors

**Pack.** All 201 manifest entries match their SHA-256, with no unlisted or missing file.
The content identity recomputes to `bd5e8fce…b3e1`. `tasks/task_007.lcl.txt` (`89a57b21…8058`)
equals the `after_sha256` of `PROCEDURAL_CHANGES.json`.

**Native verification.** The pack's runner passed all 51 synthetic language cases with
`lcl 0.9.1` ([logs/native_language_suite.json](logs/native_language_suite.json)). These are
language tests, not evidence for any task.

**Dispatch.** `lcl run … --input input.task_number=7 main.lcl.txt` ran `task.dispatch` and
`task.task_007` only, with `status.succeeded` and no diagnostics. It published the TASK-007
packet (SHA-256 of the packet text `13156692…9d9ce`), with `procedure_amendment_applies: TRUE`
([logs/dispatch_task_007.record.json](logs/dispatch_task_007.record.json)).

**Predecessors.** All six engine records are intact and `status.succeeded`, and all six
commits are on `origin/main`:

| Task | Engine record |
|---|---|
| TASK-001 | `bbda5d83…07e7` |
| TASK-002 | `c5fadc83…bd13` |
| TASK-003 | `5e4a83c1…56d0` |
| TASK-004 | `2b7406b8…c83f` |
| TASK-005 | `9554641f…7b9b` |
| TASK-006 | `b10ab5d9…9f36` |

At task start no predecessor deliverable or evidence file had an uncommitted change; every
change since their acceptance is a later accepted commit.

## 4. Required reading

The task record was read in full. These pack documents were read again for what the protocol
must carry ([logs/required_reading.txt](logs/required_reading.txt)):

- the remote-control requirements RC-01, RC-03, RC-09, RC-10, RC-12, RC-17, RC-20 and RC-23;
- the scenarios RC-T08 to RC-T10, and RC-T13, RC-T14 and RC-T16, which bound them;
- the Android operating model, A5 to A7: commands, acknowledgments, synchronization and handoff;
- the persistent-state, model-handoff (H1 to H4) and import (I1, I4) documents;
- STOP-08: a protocol mismatch blocks rather than downgrades;
- the bindings B11, B17, B20, B22 and B23, and the contracts C15, C21 and C22.

The rest of the mandatory read order is byte-identical to earlier reads and was reused.

The repository records that assign work to TASK-007 were read in full:

- **docs/architecture:**
  - SS-DOMAIN-PROTOCOL, SS-REMOTE and SS-SYNC with their slots;
  - IF-ATTACH, IF-REMOTE, IF-SYNC and IF-TRANSPORT, whose schema tasks include TASK-007;
  - ST-REMOTE-REQUEST, ST-SYNC and ST-REMOTE-SNAPSHOT;
  - the remote-control designs;
  - the flows of a remote request, synchronization, a model switch and a focus switch;
- **docs/security:**
  - the remote contracts and the execution checks EC-01 to EC-14;
  - the envelope and its illustration, SI-12, SI-13, TH-12, TH-42 and TH-46 to TH-48;
  - the bindings for frames, IPC messages, lifetimes and outcomes;
- **docs/dependencies:** DEP-SERDE, selected by TASK-003 for TASK-006 and TASK-007;
- **core/domain:** TASK-006's types.

## 5. What was decided and built

**Slots.** Six slots were added to SS-DOMAIN-PROTOCOL in `docs/architecture/subsystems.lcl.txt`:
`core/domain/changes`, `core/domain/requests`, `core/protocol/Cargo`, `core/protocol/lib`,
`core/protocol/operations` and `core/protocol/admission`. Four placeholders became source under
their stems: `core/domain/events`, `core/protocol/messages`, `core/protocol/serialization` and
`core/protocol/version_negotiation`.

| Module | What it defines | Sources |
|---|---|---|
| `version_negotiation` | The hello every channel opens with; agreement on the newest common version at or above the minimum secure version, refused otherwise | TH-42, STOP-08, RC-23 |
| `serialization` | Size checked before parsing; strict decoding; refusals that never quote the input; the canonical form of a request that approvals bind to; strict base64 for content bytes | RC-23, TH-12, `type.sec_approval` |
| `operations` | The closed registries: nine operations a peer may request, four more only the local user may; typed arguments, targets and whether each changes state | RC-09, RC-18, EC-02 |
| `messages` | The message families and their reconnect rules: intents, acknowledgments, reconciliation, status, sequenced events, execution transfer, content changes and chunks | IF-ATTACH, IF-REMOTE, IF-SYNC, C22, AD-05 |
| `admission` | The destination's EC-01 and EC-03 to EC-06, in order, with the outcome of each refusal; what a stale revision (EC-10) and a binding mismatch (EC-11) mean | RC-10, RC-17, SI-12, SI-13 |
| `core/domain/events` | Facts a host reports: workspace, replica, session, agent state, orientation, handoff, transfer, task, evidence, LCL revision, import, approval, request outcome and log lines; none carries authority | IF-ATTACH, ST-REMOTE-SNAPSHOT, H1, I4 |
| `core/domain/requests` | The destination's request ledger and the sender's pending requests, with their deduplication, restart, reconciliation and expiry rules | ST-REMOTE-REQUEST, RC-17, SI-13 |
| `core/domain/changes` | Content changes of selected synchronization: applied only onto their base content, idempotent, with conflicts that keep both versions and tombstones for deletions | ST-SYNC, A6, C22 |

**The message families:**

| Channel | Type | Messages |
|---|---|---|
| Window → its own host (local IPC) | `ClientMessage` | hello; an intent for any of the 13 operations; resync after the last event applied |
| Host → its windows | `HostMessage` | hello; acknowledgment; status; the next sequenced event |
| Host ↔ paired host | `PeerMessage::Remote` | request for one of the 9 remote operations; acknowledgment; reconcile query and reply; status; execution-transfer step |
| Host ↔ paired host | `PeerMessage::Sync` | content change; content chunk; apply result; portable record (a workspace or evidence) |

A peer message holds exactly one family; a second one in the same message is refused.

**Reconnect semantics:**
- **Between hosts:** after the hellos, the sender asks for the recorded outcome of its pending
  requests by request ID. Each one is then settled, resent under the same ID if it never
  arrived and has not expired, or dropped. A reply that does not mention a request never counts
  as "never received". Synchronization resumes; a change already applied is recognised by its
  content and changes nothing.
- **Between a window and its host:** the window resyncs after the last event it applied. It
  applies only the event that continues its sequence; after a gap it shows a fresh status
  first.
- **Neither** touches bindings, ownership or focus.

**Choices made where the sources leave room, each documented in the code:**

- **Wire format.** JSON through serde_json, as DEP-SERDE selected. A request is read twice: its
  operation name first, then the whole text strictly as that operation. serde_json's `raw_value`
  feature keeps the original text for the second read, so duplicate fields are still refused.
- **Bounds.** 64 KiB per frame between hosts and 4 KiB per local IPC message, as TASK-004's
  bindings set. 32 KiB per content chunk and per `file_write` text; larger content travels by
  synchronization. At most 256 request IDs per reconciliation and 64 sessions or tasks per status.
- **Versions.** The protocol version is the integer 1, which is also the minimum secure version.
  The first incompatible change raises it.
- **TASK-004's envelope illustration refined for the wire** (`data.sec_envelope_example`, marked
  "Illustration only"). The protocol version is an integer, not "1.0". Operation keys are
  snake_case, such as `app_launch`. Expected revisions are typed. An app launch expects the
  destination's policy revision instead of an empty list.
- **The canonical request form.** It is the envelope as compact JSON with its keys sorted at
  every level and without `approval_id`; a test pins its exact bytes. An approval binds to the
  SHA-256 of this form. The shared core holds no hashing crate yet: the only one
  `docs/dependencies` selects is `ring`, which comes with rustls and the transport (DEP-RUSTLS).
  Computing the hash is therefore left to the security component, TASK-055 and TASK-070.
- **Local-only operations.** Deciding an approval, committing or cancelling an import and
  adopting an LCL revision exist only as local intents (RC-18). The operations that may never be
  requested remotely (`binding.sec_never_remote`) are in neither registry.
- **Intents** follow the request envelope's rules for targets, expected revisions and expiry,
  without the remote identity fields. Window and host share one clock, so only the expiry and
  the lifetime ceiling apply. The host authorizes an intent like any request; the window is
  never authority.
- **Status** is a view: shown as live only while the host's session confirms it, otherwise as
  last observed with its time.
- **Execution transfer.** A host applies a transfer step only as the next step of the transfer
  it holds, with every other field unchanged. A repeated step changes nothing; a skipped or
  altered one is refused.

**Changes to TASK-006's crate:**
- `remote_request`: its target and revision check is now the function `check_targets`, which
  intents use too. The behaviour is unchanged.
- `ids`: a `ChangeId`.
- `session`: `ExecutionTransfer::executor()`, which names the one device that executes at each
  step, or none while a transfer is in flight.
- `schema`: its test that lists every record includes the three new ones.

TASK-006's 75 contract tests all still pass.

**TASK-005's runner** now checks that every enforcement point of the threat model names a slot
of the layout, as a placeholder or as source under the same stem (section 11). Two tooling
tests prove it, and CONVENTIONS.md section 12 lists it.

**Dependencies.** No new crate. `nexees-protocol` uses serde and serde_json from the workspace's
exact pins. serde_json's `raw_value` feature adds no crate. `Cargo.lock` gains only the
`nexees-protocol` entry; section 11 says who wrote it.

## 6. Deliverables

**New files:**

| Path | Purpose |
|---|---|
| `core/protocol/Cargo.toml`, `core/protocol/lib.rs` | The crate `nexees-protocol` and its documentation |
| `core/protocol/{version_negotiation,serialization,operations,messages,admission}.rs` | The protocol modules of section 5, each with its contract tests |
| `core/domain/{events,requests,changes}.rs` | The domain modules of section 5, each with its contract tests |
| `docs/evidence/TASK-007/` | This record, `logs/` and, written last, `receipt/` |

**Removed files:** the four placeholders now implemented under the same stems:
`core/domain/events.source`, `core/protocol/messages.source`,
`core/protocol/serialization.source` and `core/protocol/version_negotiation.source`.

**Changed files:**

| Path | Change |
|---|---|
| `Cargo.toml` | `core/protocol` joins the workspace members |
| `Cargo.lock` | The `nexees-protocol` entry |
| `core/domain/lib.rs` | The crate documentation and the three new modules |
| `core/domain/remote_request.rs`, `core/domain/ids.rs`, `core/domain/session.rs`, `core/domain/schema.rs` | The changes to TASK-006's crate in section 5 |
| `docs/architecture/subsystems.lcl.txt` | The six new slots of SS-DOMAIN-PROTOCOL |
| `docs/dependencies/components.lcl.txt`, `docs/dependencies/LICENSE_MATRIX.md` | DEP-SERDE: serde_json is the protocol's wire format |
| `scripts/test/run_checks.py`, `tests/tooling/test_run_checks.py` | The enforcement-point check and its tests |
| `docs/engineering/CONVENTIONS.md` | Section 3 names both crates; section 12 lists the new check |
| `README.md` | The protocol |
| `FILE_TREE.txt` | Regenerated with the runner |

## 7. Checks run

| Command | Result | Log |
|---|---|---|
| `python3 -B scripts/test/run_checks.py`, all nine stages, and `cargo build -p nexees-domain -p nexees-protocol` for both Android targets | All pass, exit 0 | [run_checks.txt](logs/run_checks.txt) |
| `cargo test --workspace` (inside the test stage) | 91 domain tests (TASK-006's 75 and 16 new) and 30 protocol tests pass | [run_checks.txt](logs/run_checks.txt) |
| Tooling tests (inside the test stage) | 26 of 26 pass | [run_checks.txt](logs/run_checks.txt) |
| Negative tests: 28 seeded defects in scratch copies of the checkout, plus 2 controls | All 28 caught by the intended stage; the controls pass | [negative_tests.txt](logs/negative_tests.txt) |
| Regression: the charter, architecture, dependency and security projects, the TASK-001 to TASK-004 cross-checks, TASK-006's contract tests and the tooling tests | 24 commands. All exit 0 except TASK-004's frozen check on the final tree, which fails only its known assertion (section 11) and passes in full with stand-ins for the three placeholders | [regression.txt](logs/regression.txt) |
| Native language suite | 51/51 | [native_language_suite.json](logs/native_language_suite.json) |
| What the owner's IDE did during the task | Recorded; the lockfile it wrote is byte-identical to the task's own offline resolution | [ide_activity.txt](logs/ide_activity.txt) |
| Secret and privacy scan | No finding | [security_scan.txt](logs/security_scan.txt) |
| Scoped cleanup with the TASK-005 tool | Nothing to remove; untracked files recorded as kept | [cleanup_record.json](logs/cleanup_record.json) |

The negative tests seed one defect per case. Twenty-two break a protocol or domain rule, and the
contract tests catch each through named failing tests, never through a compile error:

- **Admission:** a local-only operation accepted from a peer; a version below the secure
  minimum admitted; a request from a device other than the channel peer; a request for another
  user or host session.
- **Replay and outcomes:** a duplicate run again; another request under a used ID taken for a
  duplicate; an expired queued command resent; a restart that forgets a started effect; an
  unknown outcome allowed to run; a reconciliation reply read as "never received"; an intent
  whose targets go unchecked.
- **Versions and encoding:** a negotiation that settles below the secure minimum; an oversized
  message parsed; a refusal that quotes the input; a canonical form that keeps the approval;
  non-canonical base64.
- **Families, events and synchronization:** a sync message that can carry a request; an event
  that accepts an unknown field; an event applied across a gap; a change applied over content
  that changed.
- **Execution transfer:** an altered transfer record accepted; a destination that executes
  before the transfer completes.

The other six cover the repository rules:
- a portable record that must stay on its device, which fails the build on its travel
  assertion;
- unsafe code in the protocol crate;
- an unpinned dependency;
- a module without a slot;
- an enforcement point that names no slot;
- a runner that ignores the enforcement points.

## 8. Check accounting

### Verification

| Check | How it was met |
|---|---|
| T007.VERIFY.01 v0.3 gate: unknown versions, stale generations and lost acknowledgements without side-effect replay | **Unknown versions:** two hellos with no common version, or none at the secure minimum, are refused, and a hello with an unknown field is refused (`version_negotiation`). A destination refuses a request below its minimum secure version as unsupported (`admission`). **Stale generations:** after a model switch advances the generation, a request that expects the old one is stale (`admission`), and events carry the generation they belong to (`events`). **Lost acknowledgements:** RC-T09's test (`messages`): the destination runs a file write once, the acknowledgment is lost, the sender reconciles by request ID and settles, and a blind resend is answered from the ledger. The action runs exactly once. |
| T007.VERIFY.02 build the affected targets | The workspace builds, lints with warnings denied, tests and documents on Rust 1.99.0. `nexees-domain` and `nexees-protocol` also build for `aarch64-linux-android` and `x86_64-linux-android` ([run_checks.txt](logs/run_checks.txt)). |
| T007.VERIFY.03 task-specific tests | 30 protocol tests, 16 new domain tests, 2 new tooling tests and 28 negative cases (section 7). |
| T007.VERIFY.04 regressions of the changed subsystem | TASK-006's 75 contract tests pass on the changed crate. The changed architecture and dependency records pass their LCL projects and the TASK-002 and TASK-003 cross-checks. The unchanged security project and charter pass theirs. TASK-005's changed runner passes its tooling tests. TASK-004's frozen cross-check fails only its known assertion (section 11) ([regression.txt](logs/regression.txt)). |
| T007.VERIFY.05 final diff inspected | Section 6 lists every new, changed and removed file; the receipt's final verification repeats the inspection. |
| T007.VERIFY.06 v0.2 gate: duplicate messages, lost acknowledgments, mismatched versions, wrong targets, stale approvals and interrupted handoff | **Duplicates:** a repeated request gets its recorded outcome, a reused ID is refused, a repeated event, transfer step or content change changes nothing. **Lost acknowledgments:** as T007.VERIFY.01. **Mismatched versions:** as T007.VERIFY.01. **Wrong targets:** wrong device, user or host session, a sender that is not the channel peer, an agent request whose workspace or session is not the bound one, and an intent without its target are all refused (`admission`, `messages`). **Stale approvals:** an approval binds to the canonical form, which any other change alters (`serialization`). It is single-use within its epoch and time (TASK-006's `authority` test). A request waiting for the user returns to accepted only after revalidation (`requests`), and deciding an approval is local-only (`operations`). **Interrupted handoff:** while a transfer is in flight no device executes, and a skipped, altered or repeated step never moves ownership (`session`, `messages`). None of these duplicates execution or transfers ownership silently. |

### Completion gate

| Check | How it was met |
|---|---|
| T007.CLOSE.01 dependencies closed | TASK-006 accepted and its lineage verified (section 3). |
| T007.CLOSE.02 objective without unrelated scope | **Objective:** typed messages and events for workspace state (`WorkspaceChanged`, `ReplicaChanged`, status), agent state (`SessionBound`, `AgentStateChanged`, the agent operations), tasks (`TaskChanged`, `EvidenceRecorded`, task status), LCL (`LclRevisionChanged`, `lcl_validate`, `lcl_adopt`), approvals (`ApprovalRequested`, `ApprovalDecided`, `approval_decide`), logs (`Log`) and reconnect semantics (section 5). **v0.3:** import, orientation and cleanup-evidence messages use the same envelopes, with acknowledgments and expiry: `import_commit`, `import_cancel`, `agent_set_orientation`, `agent_switch_model` with its orientation, and the import, orientation and evidence events. **v0.2:** content synchronization and remote command/acknowledgment are separate families. They carry revision expectations; the authorization scope a grant must cover (operation, explicit targets and permission epoch; the grant check itself is EC-07's); expiry; idempotency; and explicit transfer and handoff states. No transport, store, UI or runtime was started. The changes to TASK-005's runner and TASK-006's crate serve this objective and are recorded in sections 5 and 11. |
| T007.CLOSE.03 build passes | As T007.VERIFY.02. |
| T007.CLOSE.04 required tests pass | As T007.VERIFY.03. |
| T007.CLOSE.05 security checks pass | The security stage and the scan are clean, and cargo-deny passes. Every serialized input is size-checked and strictly decoded; schemas and the protocol are versioned; unknown and malformed authority-changing fields are refused, as the negative cases show. Errors never quote the input. Test fixtures hold no secrets or personal data. |
| T007.CLOSE.06 no unnecessary code or dependency | No new crate. Each module fills a slot; each type cites its source. Review removed a test request that changed nothing and replaced a test that could not fail (section 9). The registry holds only operations the requirements name. |
| T007.CLOSE.07 no workspace/agent/LCL binding invariant violated | Bound checkout, branch and remote. The pack, the LCL repository, the canonical packages and the LCL SDK were not written (final verification). One primary agent. The writes of the owner's IDE are disclosed (section 11). |
| T007.CLOSE.08 evidence recorded | This folder. |
| T007.CLOSE.09 Git diff understood | As T007.VERIFY.05. |
| T007.CLOSE.10 assigned v0.2 requirements pass; later scenarios traceable | A9's TASK-007 row, "versioned sync, command/acknowledgment, replay protection and handoff contracts", is met by the families and rules of section 5, proven by contract tests. The remote requirements and scenarios below are closed for their protocol contracts, types and test plan only, as the task's remote evidence scope states, and each names its later runtime owners. |
| T007.CLOSE.11 readability and comments | Agent's own review, section 9. |
| T007.CLOSE.12 scoped cleanup | Section 9; [logs/cleanup_record.json](logs/cleanup_record.json). |
| T007.CLOSE.13 post-cleanup verification on the final revision | `receipt/final_verification.txt`. |
| T007.CLOSE.14 manual impact | Section 9. |

### Remote requirements and scenarios

Each row says what TASK-007 closed and which later tasks of the pack's requirement map own the
rest. Nothing here is implemented as a running feature or runtime-tested.

| Check | Requirement | What TASK-007 closed | Runtime owners |
|---|---|---|---|
| T007.RC.01 | RC-01 two-way device actions | One remote family and one set of admission rules serve both directions, PC to phone and phone to PC. A request is a typed operation from a closed registry; there is no free-text command, so a natural-language request is never an operation. Admission and authorization stay with the destination; a registered operation is a shape, not a permission. | TASK-066, TASK-068, TASK-070, TASK-071, TASK-073, TASK-074, TASK-075 |
| T007.RC.03 | RC-03 directional trust | No message carries a grant. The synchronization family cannot carry a request, approval or grant (test, and a compile-time travel rule). A request is admitted only from the authenticated channel peer it names (EC-03). The grant and its direction (EC-07) are the authority engine's. | TASK-022, TASK-025, TASK-030, TASK-055, TASK-060, TASK-068, TASK-070, TASK-071, TASK-073, TASK-074, TASK-075 |
| T007.RC.09 | RC-09 typed request envelope | The operation registry with typed arguments, the target each operation needs and its expected revisions. An app launch names an allowlist key and its device, never a workspace. No ambient workspace, for requests and intents alike. Strict decoding by operation; the canonical form. | TASK-015, TASK-031, TASK-033, TASK-044, TASK-045, TASK-066, TASK-068, TASK-069, TASK-070, TASK-071, TASK-073, TASK-074, TASK-075 |
| T007.RC.10 | RC-10 execution-host checks | EC-01 and EC-03 to EC-06 in order, the first failure deciding the outcome; freshness by the destination's clock; what a stale revision (EC-10) and a binding mismatch (EC-11) mean. EC-07 to EC-09 and EC-12 to EC-14 run in the destination's runtime. | TASK-015, TASK-022, TASK-023, TASK-031, TASK-033, TASK-055, TASK-060, TASK-066, TASK-070, TASK-071, TASK-073, TASK-074, TASK-075 |
| T007.RC.12 | RC-12 session and launch truth | The eight outcomes travel in acknowledgments, reconciliation and events. A sender counts a request as done only on a completed answer, never on delivery. The graphical session and the proof that a window appeared belong to the runtime. | TASK-009, TASK-024, TASK-033, TASK-035, TASK-056, TASK-068, TASK-070, TASK-071, TASK-073, TASK-074, TASK-075 |
| T007.RC.17 | RC-17 replay and unknown outcomes | The ledger: deduplication by request ID, hash and peer; a refused reused ID; unknown outcomes after a restart; allowed state changes. Pending requests and reconciliation by request ID; resend only before expiry, under the same ID. A request waiting for the user is accepted again only after revalidation. Expired commands are not synchronization data. | TASK-008, TASK-026, TASK-031, TASK-033, TASK-035, TASK-038, TASK-055, TASK-066, TASK-067, TASK-069, TASK-070, TASK-071, TASK-073, TASK-074, TASK-075 |
| T007.RC.20 | RC-20 cross-workspace independence | Every workspace and agent operation names its targets, and an agent request must match the bound session. Reconnecting touches no binding. Transfer steps apply only in order, with at most one executor. An app launch targets a device and cannot move a task. | TASK-015, TASK-017, TASK-023, TASK-031, TASK-044, TASK-045, TASK-056, TASK-061, TASK-063, TASK-065, TASK-066, TASK-068, TASK-069, TASK-070, TASK-071, TASK-072, TASK-073, TASK-074, TASK-075 |
| T007.RC.23 | RC-23 secure service integration | Bounded messages, measured before parsing; strict decoding; version negotiation with a minimum secure version and no downgrade; local intents in the same request shape, authorized by the host; refusals that never quote the input. OS peer identity for local IPC, privileges and queues belong to the hosts. | TASK-009, TASK-022, TASK-024, TASK-025, TASK-056, TASK-057, TASK-070, TASK-071, TASK-073, TASK-074, TASK-075 |
| T007.RCT.08 | RC-T08 rejected requests | **Covered by protocol tests:** malformed requests, unknown operations, duplicate and extra fields; a wrong device, user or host session; a sender that is not the channel peer; an older version; expired and far-future requests; stale revisions and generations; agent requests outside the bound session. **Covered by TASK-006's tests:** replayed and stale approvals. **Left to the runtime:** missing scopes and revoked permission epochs (EC-07), and a workspace absent on the destination (EC-09). | TASK-015, TASK-022, TASK-023, TASK-025, TASK-031, TASK-033, TASK-055, TASK-060, TASK-066, TASK-067, TASK-069, TASK-070, TASK-073, TASK-074, TASK-075 |
| T007.RCT.09 | RC-T09 lost acknowledgment | A protocol test: a file write runs once, its acknowledgment is lost, and the sender reconciles by request ID; a blind resend is answered from the ledger. An outcome-unknown entry stays unknown and is not resent. The ledger does not depend on the operation, so an app launch takes the same path. Real disconnects, the journal and the store belong to the runtime. | TASK-008, TASK-026, TASK-031, TASK-035, TASK-038, TASK-066, TASK-069, TASK-070, TASK-073, TASK-074, TASK-075 |
| T007.RCT.10 | RC-T10 expired queued request | A protocol test: an app launch queued while disconnected expires; on reconnect it is dropped, and if sent anyway the destination refuses it as expired. Content changes still synchronize, and the synchronization family cannot carry the command or its grant. | TASK-008, TASK-069, TASK-070, TASK-073, TASK-074, TASK-075 |

## 9. Review, cleanup and manuals

**Readability (agent's own review).** Every module of `core/protocol`, and each new module of
`core/domain`, opens with what it is for, its boundaries and the requirements it serves. Every
public item is documented, as the lints require. Comments state why a rule exists; tests name
the behaviour they prove. Review against the final code made these changes:
- **Decoding refusals.** A refusal claimed never to repeat the input, but serde_json's own
  messages can quote it, such as a string of the wrong type. A refusal now states only the kind
  of fault and where decoding stopped, and a test proves it.
- **The travel rule.** A comment said the synchronization family would not compile with a
  record that must stay on its device, but the check listed records by hand. A macro now
  generates the portable records from one list and checks each of them; a negative case shows
  the build failing.
- **The canonical form.** A test re-parsed the canonical form into sorted maps, so its
  key-order check could never fail. It now pins the exact bytes.
- **Applying a deletion.** It no longer looks at the argument that only renames use.
- **Tests.** A test request named for an older protocol version was built with the current one,
  so it changed nothing. It was removed, and the test now states the version it refuses. The
  test imports were tidied.
- **Comments.** Two crate comments named their owner tasks incompletely and now follow the
  architecture's schema tasks.

**Cleanup.** TASK-007 created no temporary artifact inside the checkout:
- Cargo wrote to `/mnt/F/Nexees-toolchains/target/`;
- the negative-test copies, the regression's copy and the lockfile regeneration used the
  session's scratch folder;
- every Python run used `-B`.

The cleanup tool ran on the real checkout with an empty manifest. It removed nothing and
recorded every untracked file it left in place, all of them TASK-007 deliverables and evidence
([logs/cleanup_record.json](logs/cleanup_record.json)). The owner's IDE output (`target/` and
the `.gradle/` cache) is ignored, not task-owned, and was left exactly as it is. The session
scratch folder outside the checkout is removed after the receipt. No pre-existing or
user-owned file was deleted, reset or stashed.

**Manuals.** Not applicable: TASK-007 changes no user-visible behaviour. The protocol defines
contracts and pure rules that no running host uses yet. The behaviour, and its manual text,
come with the transport, store and runtime tasks.

## 10. Policy review and disclosures

All 135 rules of the nine policy documents were reviewed for this task.

| Policy document | Rules | Relation to TASK-007 |
|---|---|---|
| `policies/master_rules.lcl.txt` | 32 | R11 (focus never rebinds) holds for every message: targets are explicit and reconnects touch no binding |
| `policies/global_contracts.lcl.txt` | 31 | C15 (one shared core), C21 (bindings) and C22 (separate planes) are the crate's structure and its family separation |
| `policies/acceptance_criteria.lcl.txt` | 11 | Product acceptance, none due now; the protocol is what later acceptance runs on |
| `policies/code_readability_and_cleanup.lcl.txt` | 6 | Applied through the conventions; the review findings are in section 9 |
| `policies/no_unnecessary_code.lcl.txt` | 7 | No new crate; only the operations the requirements name; a test request that changed nothing removed |
| `policies/reuse_policy.lcl.txt` | 5 | serde and serde_json reused as TASK-003 selected them; no new package |
| `policies/security_baseline.lcl.txt` | 9 | Validated and bounded serialized inputs, versioned schemas and protocol, refused unknown fields, fail-closed admission, unsafe code forbidden |
| `policies/usage_and_agents.lcl.txt` | 8 | One agent; Git writes only as CA-06 permits; placeholders removed with `rm` |
| `architecture/remote_device_control.lcl.txt` | 26 | RC-01, RC-03, RC-09, RC-10, RC-12, RC-17, RC-20 and RC-23 are assigned to TASK-007 and accounted for in section 8 |

Disclosures:

- **Network.** No task command used the network apart from the freshness fetch of `main` from
  `origin`. Nothing was downloaded.
- **Session hook.** A hook asks for a dynamic web-application security scan after code changes.
  No Nexees web application is running and `HAWK_API_KEY` is unset, so none was run.
- **Owner-only state.** cargo-deny was executed from the owner's `~/.cargo/bin` without writing
  there. HopToDesk, linger, the autostart folder, the phone, the LCL SDK, the owner's
  `lcl-remote` service and the owner's VS Code settings were not touched.
- **Scripts.** The evidence scripts of TASK-006 lived in the session's scratch folder, which
  was removed when TASK-006 closed. They were rebuilt from this session's own transcript and
  adapted for TASK-007; they are tools, not deliverables.

## 11. Deviations and findings

- **TASK-004's frozen cross-check now fails one assertion, as expected.**
  `check_threat_model.py` counts an enforcement point as existing only while it is a `.source`
  placeholder. Three of the threat model's points became source in this task:
  `core/protocol/messages`, `serialization` and `version_negotiation`.
  - On the final tree, that one assertion fails and the other 37 pass.
  - On a copy of the final tree with empty stand-ins for those three placeholders, the check
    passes in full ([regression.txt](logs/regression.txt)). So nothing else in the threat model
    changed, and `docs/security` is unchanged.
  - The check and its evidence are frozen with TASK-004 and were not edited. TASK-006 left the
    decision to the first task that implements such a point, which is this one. TASK-005's
    living runner now checks the same rule by stem, so a placeholder and the source under its
    stem both count. A negative case and two tooling tests prove it.
- **The owner's IDE wrote into the checkout and the home caches**
  ([logs/ide_activity.txt](logs/ide_activity.txt)).
  - **`Cargo.lock`:** one command of the task wrote `core/protocol/Cargo.toml` and added the
    crate to the members. It ended at 13:09:25.5, and the task's next command, at 13:09:46,
    wrote a source file. In between, at 13:09:26.5, rust-analyzer rewrote `Cargo.lock` with the
    new entry. Regenerated independently, offline, with the task's isolated toolchain, the lock
    is byte-identical. No crate was downloaded.
  - **`target/`:** rust-analyzer's build output grew from 198 to over 260 files as it checked
    the new crate. It is ignored.
  - **`.gradle/` beside TASK-003's prototype:** still 17 files, five of which the Gradle
    extension's builds below rewrote. It is ignored.
  - **`~/.cargo`:** only `.global-cache`, cargo's registry-use record, changed.
  - **`~/.gradle`:** the Gradle daemons kept writing their logs. At 13:13:23–25 VS Code's Gradle
    extension asked one daemon for two builds of its own, which touched the cache locks,
    journals and daemon registry, and the prototype's `.gradle/` above. TASK-007 ran no Gradle.
- **The negative tests ran twice.** The first run, before two crate comments were corrected
  (section 9), passed every case. The recorded run is on the final code.

## 12. Open items and limits

- **Who runs the protocol.** Nothing sends, stores or executes these messages yet. The other
  schema tasks of the protocol's interfaces own that:
  - the local attachment of a window to its host: TASK-009 and TASK-057;
  - the stores of the ledger, pending requests and the outbox: TASK-008;
  - the transport: TASK-060 and TASK-070;
  - synchronization: TASK-069;
  - the remote runtime that admits and executes requests: TASK-070.
- **The remaining execution checks.** EC-07 to EC-09 and EC-12 to EC-14 (grants and epochs,
  lock state, capabilities, policy and approvals, the ledger claim and the single dispatch) run
  in the destination's runtime. Section 8 names the owners.
- **The request hash.** The canonical form is fixed; the SHA-256 over it is computed by the
  security component that brings a hashing library, TASK-055 and TASK-070.
- **SI-12 and TH-47** still do not list `core/domain/remote_request` and `core/domain/ids` as
  enforcement points. TASK-006 withdrew those additions because of TASK-004's frozen check. The
  runner's new check would accept them; adding them changes `docs/security` and is left to its
  next owner.
- **The owner's IDE.** Each Cargo manifest change makes rust-analyzer rewrite `Cargo.lock` with
  the system toolchain. To prevent it, point rust-analyzer at the isolated toolchain, or have it
  ignore this folder; that is the owner's choice.
- **Also open:** the open items of `docs/dependencies` (OI-01 to OI-10), the open security
  decisions of `docs/security` (OSD-01 to OSD-08), BIND-B1-LOGO and AMEND-075-01, as their
  records state.

## 13. Next task

TASK-008 (persistent state schema and migrations foundation). It was not started.
