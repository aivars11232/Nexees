# TASK-009 evidence: Create Desktop application shell

| | |
|---|---|
| Task | TASK-009 — Create Desktop application shell |
| Date | 2026-10-04 |
| Performed by | Coding agent (Claude Code), one primary session; no sub-agents, reviewers or background AI jobs. The owner changed the session's model during the recheck (section 1). |
| Checkout | `/mnt/F/Nexees/`, branch `main`, base `d00bb83c6f8680f526076f76b1750fe8387d6821` |
| Repository | `https://github.com/aivars11232/Nexees` |
| Specification pack | `/mnt/F/Nexees_LCL_Implementation_Pack_v0.5.3/`, archive revision 0.5.3, specification version 0.5.0 |
| Pack content identity | `bd5e8fce26f542c44a891e4340da3b2a84f79ab368b5cb868beb48126c2db3e1` |
| Pack manifest SHA-256 | `8d9d0080ac13a3fb17cc70603b77eb3b11abcd3fb6d2fad515cfac136fcf07e4` |
| Procedure | Continuation profile 0.5.3 (CA-01 to CA-12), as adopted for TASK-002 and continued for this task |
| Predecessors | TASK-001 (`284f97a`), TASK-002 (`d1a673f`), TASK-003 (`2eaa199`, receipt decided in `bccc9b1`), TASK-004 (`0337be0`), TASK-005 (`e83dcd7`), TASK-006 (`fc42a93`), TASK-007 (`9f28c37`, follow-up `1c85864`) and TASK-008 (`2bf47a2`), all accepted; then the owner-directed correction `d00bb83`, which is not a task commit |
| Commit of this work | Made after acceptance, as CA-06 permits; it is recorded by Git, not in this file |

## Status

The deliverables are complete and every check passed on its final run. Acceptance is recorded
in [receipt/RECEIPT_RESULT.md](receipt/RECEIPT_RESULT.md), which is written after this file
because the receipt contains a snapshot of everything else.

**This task was closed twice.** The first close was accepted at 17:12:46 and was never
committed. The owner had asked for a recheck before each commit, and the recheck found the
task unfinished: the status bar that shows the host was never visible, the security records'
assignments to TASK-009 had not been read, and the window contacted Google's servers at each
start. That acceptance was withdrawn, the work was completed, and the task was closed again.
Section 11 lists what the recheck found; this record describes the final state.

TASK-009 built the first Nexees Desktop shell: an installed application with its own window, on
Eclipse Theia 1.76.0 and Electron 42.11.10 as TASK-003 selected them.
- **The window runs no executor.** It starts the Nexees host when none answers and attaches to
  it. The host, `nexees-host`, is a separate Rust process that runs once per OS user, at the
  user's privilege, and holds the device's state store (RC-04).
- **A private local channel.** Window and host talk over a Unix socket in a folder only the
  user can enter. The host checks the peer's user by the kernel's account before reading
  anything, every message is bounded, and a protocol mismatch is refused (RC-23).
- **An honest lifetime.** The host stops a few seconds after its last window closes, unless
  start at login is enabled, which keeps it running; start at login is one user-level entry
  that exists only while enabled (RC-05, RC-24). Outside a login session the host refuses to
  run and reports that as unsupported.
- **What the user sees.** The status bar shows the host as attached, or as unavailable with the
  reason, and a click tries again (RC-06).
- **No idle model use.** The host has no model provider at all and waits on blocking reads
  (RC-08).
- **A hardened runtime.** The window's backend serves only the window that started it and
  refuses to start any other way; the page runs in a sandboxed renderer without Node; the
  installed Electron honours neither `NODE_OPTIONS` nor inspector arguments; and the window
  connects to nothing beyond this machine (SI-28, TH-02 to TH-04).
- **Extensions stay disabled.** The shell leaves out Theia's extension host and Open VSX
  client, as SA-08 and SI-22 require until TASK-021.

The window's TypeScript is checked strictly by the check runner, and the npm lockfile has its
own gate. The application builds offline, installs into a prefix, and an end-to-end test runs
the installed application in a private nested session: all 30 of its checks pass.

Before TASK-009 began, the owner had an audit finding corrected in the domain model: commit
`d00bb83` (section 1). All review here is the agent's own. Under CA-05 the routine
corroboration was done by this session's own local tools; no independent human or model review
has taken place.

## 1. Authority

The owner's instructions, verbatim, with the times they arrived (CEST):

At 15:19:32, after TASK-008 was committed:

```
Before u Start task 9, look into this - Small correction/audit during the current task:

Continue the current task normally and follow the existing package rules and scope.

Before freezing/persisting the current domain state, review the task/evidence boundary in `core/domain/task.rs`.

The current model says that a model claim cannot complete a task, but `Task::complete()` appears to trust a passing portable `Evidence` record mainly by its claimed execution-device ID and checkout revision. Make sure task completion can distinguish evidence genuinely derived/corroborated by the local execution host from agent-declared, copied, synced, or otherwise merely self-described evidence. Do not solve this with another forgeable boolean or enum in the portable record.

Add negative coverage proving that a fabricated/copied/self-attested passing evidence record cannot complete a task merely by claiming the correct host and revision.

Also check the distinction between ordinary `Verification` and `FinalVerification` against the existing architecture requirement for post-cleanup verification before completion. Do not implement later tasking/verification-gate work early; only make the narrow domain/state correction needed now, or preserve the later-task boundary explicitly if that is where enforcement belongs.

Do not disturb completed task evidence or unrelated work. Then continue the current task normally.
```

At 15:33:57:

```
After task 9 is done, checked if it's really done, proceed with task 10-15, but u have to commit and sync after each task as well as recheack if it's really done.
```

At 15:38:06:

```
Ou and one more thing, u also have to do report after each task, then recheack , then report again and only then commit and sync and next task.
```

At 15:39:47:

```
and don't forget to do cleanup after each task
```

During the recheck, at 17:24:04:

```
If u're not sure for something, read what Opus 5.5 did, and what instructions it got from me
```

And at 17:54:06, with a picture of VS Code's Problems panel showing `Cannot find type definition
file for 'node'` for `apps/desktop/src/tsconfig.json`:

```
I don't know if u're aware of this, so i let u know
```

At 17:06:18 the owner also asked which task was in progress; it was answered in the session.

**The model changed.** The session's model was Claude Opus 5.5 until 17:23, which did the first
close and gave the first report. The owner then switched the session to Claude Fable 5.1, which
did the recheck from 17:26 on, with the session's own record of every earlier command and
instruction to read. It stayed one session and one agent at a time.

**The correction.** It was made before TASK-009 began, as its own commit, `d00bb83` ("Let
only the execution host's own final verification complete a task"), pushed at 15:37. It is
not part of TASK-009's change: CA-03 does not let a task repair an unrelated predecessor defect
under its own authority, and the owner's instruction is the authority for it.
- **What changed.** `Task::complete` now takes a `HostVerification`: the verification the
  execution host ran itself, holding the evidence record it made. A `HostVerification` has no
  serialized form, so no record that was synced, copied, imported or read back can become one.
  The task cites only exactly that record, and only a passing `FinalVerification` completes it.
  The portable `Evidence` record is unchanged: no new field, flag or kind.
- **The boundary.** When the final verification runs, after the readability review and the
  scoped cleanup, stays with the gates of `core/tasking`: TASK-054's verification gate and the
  cleanup gate, as the module now documents.
- **Its proof.** New domain tests try fabricated, self-attested, copied and synced records that
  name the correct host and revision, another device's run, a pre-cleanup verification, a
  failed and a stale one; a `compile_fail` example shows that a host verification cannot be
  decoded. Re-run during TASK-009, five seeded defects in scratch copies are each caught by the
  intended test ([logs/correction_seeded_defects.txt](logs/correction_seeded_defects.txt)).
- **What it did not touch.** No record schema, so the state store needs no migration, and no
  completed task's evidence.

The later messages extend the owner's assignment to TASK-010 through TASK-015, each with its
own receipt, report, recheck, report, commit and push, and cleanup. This record covers
TASK-009 only.

## 2. Checkout preflight (binding B26)

| Item | Observed |
|---|---|
| Working directory | `/mnt/F/Nexees/`; the Git worktree root, with no parent repository, superproject or second worktree |
| `origin` fetch and push | `https://github.com/aivars11232/Nexees.git`; no URL rewrite, no separate push URL |
| Branch and HEAD | `main` at `d00bb83`, equal to `origin/main` after a fetch of that branch only, without tags |
| Hooks | None besides Git's samples; `core.hooksPath` unset |
| At task start | Clean: no staged, modified or untracked file, no stash, 551 tracked files. The owner's IDE keeps ignored output in the checkout: 404 files under `target/` and 17 under TASK-003's Gradle prototype (`.gradle/`); six IDE processes; rust-analyzer's last check, from 14:39:45, holds no error. All of it is recorded as pre-existing, not task-owned state. |
| Result | Passes ([logs/preflight.txt](logs/preflight.txt)) |

TASK-009 began at 2026-10-04T15:39:41+02:00, after the correction was pushed. That time is the
baseline of every "nothing written since" check.

## 3. Pack, native verification, dispatch and predecessors

**Pack.** All 201 manifest entries match their SHA-256, with no unlisted or missing file. The
content identity recomputes to `bd5e8fce…b3e1`. `tasks/task_009.lcl.txt` (`3f07ec06…d104`)
equals the `after_sha256` of `PROCEDURAL_CHANGES.json`.

**Native verification.** The pack's runner passed all 51 synthetic language cases with
`lcl 0.9.1` ([logs/native_language_suite.json](logs/native_language_suite.json)). These are
language tests, not evidence for any task.

**Dispatch.** `lcl run … --input input.task_number=9 main.lcl.txt` ran `task.dispatch` and
`task.task_009` only, with `status.succeeded` and no diagnostics. It published the TASK-009
packet (SHA-256 of the packet text `5ce9a2c6…4195`), with `procedure_amendment_applies: TRUE`
([logs/dispatch_task_009.record.json](logs/dispatch_task_009.record.json)).

**Predecessors.** All eight engine records are intact and `status.succeeded`, and all their
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
| TASK-008 | `a8c8b44d…b2a2` |

At task start no predecessor deliverable or evidence file had an uncommitted change. Every
change since their acceptance is a later accepted commit, except `d00bb83`, which changed
`core/domain/task.rs`, TASK-006's deliverable, under the owner's instruction (section 1); the
preflight names it as such.

## 4. Required reading

The task record was read in full, and so was `architecture/desktop_ui.lcl.txt`, which no
earlier task had read. These pack documents were read again for what the shell must do
([logs/required_reading.txt](logs/required_reading.txt)):

- the reuse matrix, for the reuse-first requirement;
- RC-04, RC-05, RC-06, RC-08, RC-12, RC-23 and RC-24, and the scenarios RC-T02, RC-T03, RC-T04,
  RC-T14 and RC-T18;
- C20 and C21; R17 and R26; the bindings; the stop conditions; the manuals and help;
- the readability and cleanup policy and the no-unnecessary-code policy.

The rest of the mandatory read order is byte-identical to earlier reads and was reused.

The repository records that assign work to TASK-009 were read in two rounds. The first, before
the implementation, covered:

- **docs/architecture:** SS-DESKTOP-CLIENT, SS-LOCAL-HOST, SS-PLATFORM-DESKTOP and
  SS-BUILD-RELEASE, with their slots, owners and test slots; HOST-DESKTOP; AD-02, PD-LOCAL-IPC
  and GAP-01 to GAP-03; the designs of RC-04, RC-05, RC-08 and RC-23; FL-TASK-CLOSURE and
  ST-TASK;
- **docs/security:** the remote contracts of RC-23 and RC-24; SA-08, SI-22 and R-02 on
  extensions;
- **docs/dependencies:** DEP-THEIA, DEP-ELECTRON, DEP-NIX, DEP-XDG-AUTOSTART, DEP-GIO and
  DEP-OPEN-VSX; the npm overrides and the advisory dispositions; the repository tools; the
  renderer decision DESKTOP-RENDERER-01 (`electron_permitted`);
- **docs/evidence/TASK-003:** the Theia prototype and its build log and OSV records, and the
  local IPC proof, as references, not reused code;
- **docs/engineering/CONVENTIONS.md** and the check runner;
- **core/protocol:** the window-host message family, version negotiation and the local IPC
  message bound; **core/state:** the store.

That round missed records that name TASK-009 as an owner or decider. The recheck read them:

- **docs/security:** the invariants SI-15, SI-17 and SI-28; the threats TH-01 to TH-04 and
  TH-06; the threat test TT-26 and the test gap SG-05; the residual risk R-03 and the open
  decision OSD-06; and TASK-004's recorded facts about Theia and Electron;
- **docs/architecture:** the open decisions PD-IDE-PLACEMENT, PD-LOCAL-IPC and PD-STARTUP in
  full, and every gap from GAP-01 to GAP-07;
- **docs/evidence/TASK-003:** the owner's answer on testing start at login;
- Theia 1.76.0's own code, for how it starts its backend, checks its token, builds its window
  and shows its status bar.

## 5. What was decided and built

**Slots.** Nine placeholders became source under their stems: `apps/desktop/src/main`,
`apps/desktop/src/application_host`, `apps/desktop/src/shell/main_window`,
`apps/desktop/resources/application_metadata`, `platform/desktop/transport/local_ipc`,
`platform/desktop/lifecycle/application_lifecycle`, `packaging/desktop/package_definition`,
`scripts/build/build_desktop` and the test slot `tests/e2e/desktop/local_application`. Seven
slots were added to `docs/architecture/subsystems.lcl.txt` for the manifests and the platform
crate's root: `apps/desktop/package`, `apps/desktop/package-lock`, `apps/desktop/src/package` and
`apps/desktop/src/tsconfig` (SS-DESKTOP-CLIENT), `apps/desktop/Cargo` (SS-LOCAL-HOST), and
`platform/desktop/Cargo` and `platform/desktop/lib` (SS-PLATFORM-DESKTOP). One test slot was
added, `tests/tooling/test_build_desktop` (SS-BUILD-RELEASE).

| Module | What it does | Sources |
|---|---|---|
| `platform/desktop/transport/local_ipc` | The channel: the folder `nexees` in `$XDG_RUNTIME_DIR` at 0700 and the socket at 0600, both checked before use; the peer's user checked with `SO_PEERCRED` before anything is read; frames of a 4-byte length and JSON, at most 4096 bytes, measured before they are read | PD-LOCAL-IPC, RC-23, DEP-NIX |
| `platform/desktop/lifecycle/application_lifecycle` | One host per user (an exclusive lock on `host.lock`); closing whatever descriptors the host inherited; start at login as one user-level XDG autostart entry, removed on disable, another program's entry left alone; the refusal to run as root | RC-04, RC-05, RC-24, DEP-XDG-AUTOSTART |
| `apps/desktop/src/application_host` | The host binary `nexees-host`: `serve`, `start`, `status` and `login-start`; the state store in `$XDG_DATA_HOME/nexees/state.db`; the hellos, the resync answered with the host's status, intents answered `unsupported`; the host's lifetime | SS-LOCAL-HOST, HOST-DESKTOP, RC-04, RC-12, RC-23 |
| `apps/desktop/src/main` (`main.ts`, `main.protocol.ts`) | The window's side of the host, in Theia's backend: `nexees-host start`, the folder and socket checks, the handshake, bounded retries, the host's state for the frontend. The backend refuses to load without the window's token and keeps that token out of every process it starts | RC-04, RC-06, RC-08, RC-23, TH-02, SI-28 |
| `apps/desktop/src/main` (`main.electron.ts`) | The window's part in Electron's main process: its network stack resolves no name but this machine's, and Theia's settings go into the window's own configuration folder | C20, TH-04 |
| `apps/desktop/src/shell/main_window` | The status bar entry: attached, attaching, or unavailable with the reason, and a command to try again | RC-06 |
| `scripts/build/build_desktop` | `check`, `build` and `install`, outside the checkout, npm offline on the committed lockfile; `install` sets the fuses the package definition names in the installed Electron binary | DS-06, DS-09, SI-28, TH-03 |
| `packaging/desktop/package_definition`, `apps/desktop/resources/application_metadata` | The installed layout, the launcher, the fuses and the desktop entry | C20, SI-28 |
| `tests/e2e/desktop/local_application` | The installed application, end to end, in a private session, including its runtime hardening | C20, RC-04 to RC-08, RC-23, RC-T14, SG-05, TT-26 |

The state store gained this device's identity: recorded once in its metadata and kept, a
different one refused. The host's status names the device by it.

**Choices made where the sources leave room, each documented in the code or its record:**

- **The package set.** TASK-003's prototype named `@theia/plugin-ext-vscode` and
  `@theia/vsx-registry`. SA-08 and SI-22 keep extensions disabled until TASK-021, because
  Theia's `.vsix` unpacker uses `decompress` 4.2.1, which has unfixed critical advisories. The
  shell leaves both out. It names `@theia/preferences` and `@theia/messages` itself, which the
  prototype had only through those two: without the first, Theia's preference service never
  becomes ready and Theia keeps its status bar hidden; without the second, Theia writes its
  messages to the log instead of showing them. The lockfile keeps 807 of the reviewed packages
  and adds none; 56 package names of the prototype's tree are gone. All five packages with
  advisories are build tooling only, which the running window never loads.
- **Where the IDE's backend runs (PD-IDE-PLACEMENT).** In Theia's Node backend, a child process
  of the window's main process, as Theia does by default. Running it inside the main process
  was tried and rejected: a process the backend starts there inherits 29 descriptors of the
  window's main process, among them its sockets, Chromium's shared memory and the graphics
  device ([logs/hardening_trials.txt](logs/hardening_trials.txt)).
- **The host keeps nothing of its starter.** Even the separate backend leaves five descriptors
  of its own open in what it starts, so the host closes whatever it inherited before it opens
  anything.
- **The backend's token (TH-02).** Theia's backend compares a per-launch token on every request
  but admits every request when started without one. The Nexees backend module refuses to load
  without the token, so a backend started any other way stops before it listens. Once Theia
  holds the token, the variable that carried it is removed, so no terminal, file watcher or
  host inherits it.
- **The renderer sandbox (TH-04, OSD-06).** On. Theia leaves it off by default; its preload
  script needs only Electron's own module, and the workbench runs with the sandbox.
- **The fuses (TH-03).** The installation sets `EnableNodeOptionsEnvironmentVariable` and
  `EnableNodeCliInspectArguments` off in its copy of the Electron binary. Three others keep
  Electron's defaults, each for a recorded reason: `RunAsNode`, because Theia starts its backend
  through it; the two archive fuses, because this layout installs files, not an archive; and
  `GrantFileProtocolExtraPrivileges`, because Theia's window does not start without it. SI-28
  is therefore met in full only by a release package (section 12).
- **No connection beyond this machine.** Electron's spell checker fetches its dictionaries from
  Google's servers each time a profile starts, and no setting stops the first fetch in time.
  The window's network stack therefore resolves no name but this machine's.
- **An identity of its own.** Electron takes the application's name from the folder it starts,
  so the launcher starts the application folder and its `package.json` names the product. The
  window then keeps its data in `Nexees` in the user's configuration folder, with Theia's
  settings inside, not in the folders `Electron` and `.theia` that other applications share.
- **The host's lifetime.** A host started by a window stops 5 seconds after its last window
  detaches, so a reloading window reattaches to it. `serve --keep-running`, which start at
  login runs, keeps it: RC-04 makes keeping the host after the window closes its own opt-in. The
  Settings switch for it is TASK-030's.
- **Device identity.** A new store gets a random identity, `desktop-` and 32 hexadecimal
  digits, from the kernel's random source; the store keeps it. Device keys and pairing come
  with TASK-025 and TASK-060.
- **The TypeScript lint.** No new package: the TypeScript compiler of the reviewed tree, with
  every strict check on, is the lint, and `.editorconfig` is the formatting rule.
- **The npm gate.** Exact versions, registry sources with sha512 hashes, an `allowScripts`
  decision for every install script (`fsevents` is denied explicitly), and the locked package
  set against the OSV review recorded in `docs/dependencies/npm_review.json`.
- **An offline build.** The native modules are rebuilt against the cached Electron 42.8.1
  headers. Every Electron 42 has the native module ABI 146, the installed 42.11.10 reports it,
  and the three rebuilt modules load in it ([logs/desktop_build.txt](logs/desktop_build.txt)).
  Theia's ffmpeg step and Electron itself come from the caches; nothing is downloaded.
- **The host prints nothing.** The workspace lints reject console output in shipped code; each
  command answers with an exit code, and the window maps them to reasons.

**Dependencies.** No new third-party component. nix 0.31.3 is pinned in the workspace, as
TASK-003 selected it (DEP-NIX). Node.js 26.10.0 with npm 12.2.0, which builds the window and runs
the TypeScript check, is recorded as the repository tool TOOL-04; it is never shipped.

## 6. Deliverables

**New files:**

| Path | Purpose |
|---|---|
| `platform/desktop/Cargo.toml`, `platform/desktop/lib.rs`, `platform/desktop/transport/local_ipc.rs`, `platform/desktop/lifecycle/application_lifecycle.rs` | The crate `nexees-platform-desktop` |
| `apps/desktop/Cargo.toml`, `apps/desktop/src/application_host.rs` | The crate `nexees-desktop-host` and its binary `nexees-host` |
| `apps/desktop/package.json`, `apps/desktop/package-lock.json` | The window's Theia application and its lockfile |
| `apps/desktop/src/package.json`, `apps/desktop/src/tsconfig.json`, `apps/desktop/src/main.ts`, `apps/desktop/src/main.protocol.ts`, `apps/desktop/src/main.electron.ts`, `apps/desktop/src/shell/main_window.ts` | The Nexees extension of the window |
| `apps/desktop/resources/application_metadata.json`, `packaging/desktop/package_definition.json` | The application's metadata and its installed layout |
| `scripts/build/build_desktop.py` | Check, build and install |
| `tests/e2e/desktop/local_application.test.mjs` | The end-to-end test |
| `tests/tooling/test_build_desktop.py` | Tests of the fuse setting |
| `docs/dependencies/npm_review.json` | The npm lockfile's advisory review |
| `docs/evidence/TASK-009/` | This record, `logs/` and, written last, `receipt/` |

**Removed files:** the nine placeholders now implemented under the same stems:
`apps/desktop/resources/application_metadata.source`, `apps/desktop/src/application_host.source`,
`apps/desktop/src/main.source`, `apps/desktop/src/shell/main_window.source`,
`packaging/desktop/package_definition.source`,
`platform/desktop/lifecycle/application_lifecycle.source`,
`platform/desktop/transport/local_ipc.source`, `scripts/build/build_desktop.source` and
`tests/e2e/desktop/local_application.test.source`.

**Changed files:**

| Path | Change |
|---|---|
| `Cargo.toml`, `Cargo.lock` | The two crates join the workspace; nix pinned; the lockfile's new entries (section 11 says who wrote it) |
| `core/state/state_store.rs` | This device's identity, recorded once and kept |
| `docs/architecture/subsystems.lcl.txt` | The seven new slots and the new test slot |
| `docs/architecture/decisions.lcl.txt` | TASK-009's answers to PD-IDE-PLACEMENT, PD-LOCAL-IPC and PD-STARTUP, and to GAP-01, GAP-02 and GAP-03 |
| `docs/security/threats.lcl.txt`, `docs/security/tests.lcl.txt`, `docs/security/trust.lcl.txt`, `docs/security/checks.lcl.txt` | TH-03 and TH-04 brought up to what TASK-009 did; SG-05 resolved; R-03 and OSD-06 answered; R-09 and OSD-09 added, with the two counts. No invariant changed |
| `docs/dependencies/components.lcl.txt`, `docs/dependencies/strategy.lcl.txt`, `docs/dependencies/checks.lcl.txt`, `docs/dependencies/LICENSE_MATRIX.md` | TASK-009's verification of DEP-THEIA, DEP-ELECTRON, DEP-NIX, DEP-XDG-AUTOSTART and DEP-OPEN-VSX; TOOL-04 and its count; the npm review in the advisory gate |
| `docs/engineering/CONVENTIONS.md`, `.editorconfig` | The platform and host crates, the TypeScript decision, the npm gate, the Desktop build folder, the fuses, the editor's link, end-to-end tests, the runner's settings |
| `.gitignore` | The editor's link to the window's packages |
| `scripts/test/run_checks.py`, `tests/tooling/test_run_checks.py` | The TypeScript and JavaScript checks in the lint stage, the npm gate in the deps stage, and six tests of the gate |
| `scripts/ci/continuous_integration.sh` | Node.js among the CI tools, and the npm cache filled before the checks |
| `docs/manuals/NEXEES_USER_MANUAL.md` | The Desktop window and the Nexees host (section 9) |
| `README.md` | The Desktop shell |
| `FILE_TREE.txt` | Regenerated with the runner |

## 7. Checks run

| Command | Result | Log |
|---|---|---|
| `python3 -B scripts/test/run_checks.py`, all nine stages, and `cargo build -p nexees-domain -p nexees-protocol -p nexees-state` for both Android targets | All pass, exit 0 | [run_checks.txt](logs/run_checks.txt) |
| `cargo test --workspace` (inside the test stage) | 9 host, 93 domain, 12 platform, 30 protocol and 29 state tests, and 2 doctests, pass | [run_checks.txt](logs/run_checks.txt) |
| Tooling tests (inside the test stage) | 35 of 35 pass | [run_checks.txt](logs/run_checks.txt) |
| `scripts/build/build_desktop.py check`, `build` from scratch, `install` | Offline: 792 packages installed, `theia build` with 0 errors, native modules rebuilt, the host built; installed in a disposable prefix (783 MB) with two fuses set | [desktop_build.txt](logs/desktop_build.txt) |
| `node tests/e2e/desktop/local_application.test.mjs` on that installation | 30 of 30 checks pass | [e2e_local_application.txt](logs/e2e_local_application.txt) |
| Four trials of configurations the installation does not use, and of what the window asks of the network | Each came out as the records state it | [hardening_trials.txt](logs/hardening_trials.txt) |
| Negative tests: 23 seeded defects in scratch copies of the checkout, plus 2 controls | All 23 caught by the intended stage; the controls pass | [negative_tests.txt](logs/negative_tests.txt) |
| The correction `d00bb83`: five seeded defects | All caught by the intended domain test | [correction_seeded_defects.txt](logs/correction_seeded_defects.txt) |
| Regression: the charter, architecture, dependency and security projects, the TASK-001 to TASK-004 cross-checks, the domain, protocol and state contract tests and the tooling tests | 24 commands, all exit 0 | [regression.txt](logs/regression.txt) |
| OSV query of the lockfile's 807 packages | 9 advisories in 5 packages, all build tooling, all with recorded dispositions | [osv_npm.json](logs/osv_npm.json) |
| Native language suite | 51/51 | [native_language_suite.json](logs/native_language_suite.json) |
| What the owner's IDE did during the task | Recorded; the lockfile it wrote is byte-identical to the task's own offline resolution | [ide_activity.txt](logs/ide_activity.txt) |
| Secret and privacy scan | No finding | [security_scan.txt](logs/security_scan.txt) |
| Scoped cleanup with the TASK-005 tool | At the first close: the task's one stray bytecode file and its folder removed. At the final close: nothing left to remove | [cleanup_first_close.json](logs/cleanup_first_close.json), [cleanup_record.json](logs/cleanup_record.json) |

**The end-to-end test** runs the installed application in a private session it makes itself:
a nested KWin compositor that draws into memory, a D-Bus session that activates no service, and
a temporary home with its XDG and runtime folders, with core dumps off. No window reaches the
owner's screen. It answers Theia's question whether to trust a new folder with no, opens the
Explorer as a user would, and checks:
- the page is a file of the installed application, not a website (C20), and the Explorer shows
  the local workspace's files;
- nothing of the application listens beyond this machine, it connects to no other machine and
  has fetched nothing, and it keeps its settings in its own folder;
- extensions stay disabled: no Extensions view and no extension host (SA-08);
- the status bar shows the host as attached, where the user sees it: an entry counts only when
  it is visible;
- exactly one host runs, separate from the window and holding nothing of it, at the user's
  privilege, with the channel private to the user (RC-04, RC-23);
- the idle host uses no processor time in 5 seconds and holds no network socket (RC-08);
- without the opt-in, the host stops after its last window closed; a kept host outlives its
  windows, and three reopened windows find that same host (RC-04, RC-T14);
- start at login is off by default; enabling it writes one user-level entry, which systemd's own
  generator turns into one unit bound to the graphical session; disabling it removes exactly
  that entry; outside a login session the host exits as unsupported (RC-05, RC-24);
- a host that cannot run leaves the window unavailable, with the reason shown on pointing at
  the entry, and a click attaches the window once the cause is gone (RC-06, RC-T03);
- the hardening (SG-05, TT-26): the fuses the package definition names are set; the backend
  refuses to start without the window's token; started with `ELECTRON_RUN_AS_NODE`,
  `NODE_OPTIONS` and an inspector argument, the application still starts, loads nothing and
  opens no inspector; the page runs in a sandboxed renderer without Node; no process the
  backend starts inherits the token.

At its end it stops every process of the session, the D-Bus daemon included, which
`dbus-run-session` then reports as terminated by signal 9.

**The trials** use a second disposable installation, changed by a scratch script:
- with Electron's default fuses, the end-to-end test fails on exactly the three checks that
  depend on them, so those checks are real;
- in Theia's single-process mode, a process the backend starts inherits 29 descriptors of the
  window's main process instead of 5;
- with `GrantFileProtocolExtraPrivileges` off, Theia's page cannot read its local storage and
  the workbench never appears;
- the only address beyond this machine that the window asks for is Electron's spell-checking
  dictionary, and that request fails to resolve.

**The negative tests** seed one defect per case. Fourteen break a rule of the host, its
channel, its lifecycle, the store or the fuse setting, and the tests catch each through the
named failing test, never through a compile error:
- **The channel:** a peer of another user admitted; a frame over the limit read anyway; a
  runtime folder open to others accepted.
- **The lifetime:** a second host taking the lock; another program's autostart entry removed; a
  shell-like command written into the entry; a kept host stopping; a host never stopping.
- **Serving windows:** a window of another protocol version kept; more windows than the bound
  attached; an intent reported as carried out.
- **The store:** the device identity replaced.
- **The fuses:** fuses set in a binary whose fuse wire is not the known one; a fuse that
  Electron removed overwritten.

The other nine cover the window and the repository rules: a TypeScript strictness error, a
JavaScript syntax error, unsafe code in the platform crate, an npm version range, a lockfile
changed without a new review, an install script without a decision, nix not pinned, a window
source without a slot, and a tab-indented TypeScript line.

## 8. Check accounting

### Verification

| Check | How it was met |
|---|---|
| T009.VERIFY.01 v0.2 Desktop gate: record the approved renderer choice and prove installed-shell startup and local workspace access without a hosted UI website | **Renderer:** `electron_permitted`, the owner's answer to DESKTOP-RENDERER-01 that TASK-003 recorded (`binding.dep_renderer_decision`); the shell renders with Electron 42.11.10. **Installed startup:** the application, built offline, installed into a prefix by `build_desktop.py install`, started from the installed launcher. **Local workspace:** its Explorer showed the files of a local folder. **No website:** the page is the installed application's own `lib/frontend/index.html`; its backend and nothing else of it listen, on the loopback interface; and it connects to no other machine (logs/e2e_local_application.txt). |
| T009.VERIFY.02 build the affected targets | The workspace builds, lints with warnings denied, tests and documents on Rust 1.99.0, and the core crates build for both Android targets. The window builds with `theia build` and its TypeScript compiles with every strict check ([run_checks.txt](logs/run_checks.txt), [desktop_build.txt](logs/desktop_build.txt)). |
| T009.VERIFY.03 task-specific tests | 12 platform, 9 host and 1 new state test; 9 new tooling tests; the end-to-end test; 23 negative cases; four trials (section 7). |
| T009.VERIFY.04 regressions of the changed subsystem | The changed architecture, dependency and security records pass their LCL projects and the TASK-002, TASK-003 and TASK-004 cross-checks; the unchanged charter passes its own; the domain, protocol and state tests and the tooling tests pass ([regression.txt](logs/regression.txt)). |
| T009.VERIFY.05 final diff inspected | Section 6 lists every new, changed and removed file; the receipt's final verification repeats the inspection. |

### Completion gate

| Check | How it was met |
|---|---|
| T009.CLOSE.01 dependencies closed | TASK-008 accepted and its lineage verified (section 3). |
| T009.CLOSE.02 objective without unrelated scope | **Objective:** the chosen foundation, Theia on Electron, is bootstrapped as the Nexees window, with its own workbench, editor, explorer and terminal reused rather than replaced, and the minimal Nexees shell launches. **v0.2 Desktop requirement:** an installed local application with its own window and local workspace and terminal, and the local host that agents will run in; the approved renderer only. The branding, layout, panels and persistence of TASK-010 to TASK-012, the receiver, and extensions were not started. |
| T009.CLOSE.03 build passes | As T009.VERIFY.02. |
| T009.CLOSE.04 required tests pass | As T009.VERIFY.03. |
| T009.CLOSE.05 security checks pass | The security stage and the scan are clean, cargo-deny passes and the npm gate passes. The channel is private to the user and checks the peer's user; messages are bounded and decoded strictly; a protocol mismatch is refused; the host refuses root and has no privileged helper; the window's status shows only what the host confirmed. The security records' assignments to TASK-009 are answered in the table below; what they leave open is recorded as R-09 and OSD-09. Extensions stay disabled (SA-08). Test fixtures hold no secrets or personal data. |
| T009.CLOSE.06 no unnecessary code or dependency | No new third-party component; 56 package names fewer than TASK-003's evaluated tree; Theia's own workbench reused. Review removed an unused helper, an unused development dependency and, in the recheck, lifecycle states that nothing used (section 9). |
| T009.CLOSE.07 no workspace/agent/LCL binding invariant violated | Bound checkout, branch and remote. The pack, the LCL repository, the canonical packages and the LCL SDK were not written (final verification). One primary agent at a time. The writes of the owner's IDE are disclosed (section 11). |
| T009.CLOSE.08 evidence recorded | This folder. |
| T009.CLOSE.09 Git diff understood | As T009.VERIFY.05. |
| T009.CLOSE.10 readability and comments | Agent's own review, section 9. |
| T009.CLOSE.11 scoped cleanup | Section 9; [logs/cleanup_record.json](logs/cleanup_record.json). |
| T009.CLOSE.12 post-cleanup verification on the final revision | `receipt/final_verification.txt`. |
| T009.CLOSE.13 manual impact | Updated: section 9. |

### Remote requirements and scenarios

Each row says what TASK-009 closed, with an implementation and its tests, and which later tasks
of the pack's requirement map own the rest. Nothing here is runtime-tested end to end with a
second device.

| Check | Requirement | What TASK-009 closed | Later owners |
|---|---|---|---|
| T009.RC.04 | RC-04 Desktop receiver independent of window | The host is a process separate from the window, one per user by its lock, so a reopened window reattaches instead of starting a second executor; it keeps no descriptor of the window that started it; keeping it after the window closes is its own opt-in. Proven by host tests and end to end. The receiver inside the host, background tasks and the Settings switch are their owners'. | TASK-017, TASK-038, TASK-056, TASK-070, TASK-071, TASK-073, TASK-074, TASK-075 |
| T009.RC.05 | RC-05 Startup and login are distinct | Start at login is one user-level autostart entry, off by default and present only while enabled; systemd's generator turns it into a unit bound to the graphical session; no system service, linger, firewall rule or automatic login. Outside a login session the host exits as unsupported, and nothing starts it before login or keeps it after logout. Proven in tests and in an isolated profile; the owner's real login and logout were not exercised (section 11). | TASK-024, TASK-030, TASK-056, TASK-070, TASK-071, TASK-073, TASK-074, TASK-075 |
| T009.RC.06 | RC-06 No receiver means unavailable | When the window cannot reach a host it says unavailable, with the reason, and starts nothing else; it retries a bounded number of times, and a click tries again. Proven end to end. The phone's view of an unreachable PC is its owners'. | TASK-035, TASK-070, TASK-071, TASK-073, TASK-074, TASK-075 |
| T009.RC.08 | RC-08 No idle model usage | The host has no model provider; it waits on blocking reads and a condition variable, with bounded windows and the window's bounded retries. Measured end to end: no processor time in 5 idle seconds, no network socket. The receiver's budgets are its owners'. | TASK-030, TASK-056, TASK-070, TASK-071, TASK-073, TASK-074, TASK-075 |
| T009.RC.12 | RC-12 Desktop session and launch truth | The window is launched in the user's graphical session and counts as attached only once the host answers its hello and status, never because a process was spawned; an intent gets an explicit `unsupported` outcome. Launching applications and their states are their owners'. | TASK-024, TASK-033, TASK-035, TASK-056, TASK-068, TASK-070, TASK-071, TASK-073, TASK-074, TASK-075 |
| T009.RC.23 | RC-23 Secure service integration | The channel of PD-LOCAL-IPC: a 0700 folder and a 0600 socket, the peer's user checked by the kernel, 4096-byte frames measured before reading, strict decoding, version mismatch refused, no root and no privileged helper; the IDE backend runs only from the application with its token. The permission engine and transport hardening are their owners'. | TASK-022, TASK-024, TASK-025, TASK-056, TASK-057, TASK-070, TASK-071, TASK-073, TASK-074, TASK-075 |
| T009.RC.24 | RC-24 Update, logout and removal | Disabling start at login removes only the entry Nexees wrote, never another program's; the store's files stay listed for removal (TASK-008). Updates, sign-out and uninstalling are their owners'. | TASK-026, TASK-030, TASK-035, TASK-038, TASK-056, TASK-067, TASK-069, TASK-070, TASK-071, TASK-073, TASK-074, TASK-075 |
| T009.RCT.02 | RC-T02 Android-to-PC app launch | The PC side's foundation: closing only the window leaves a kept host running and reattachable. Pairing, grants, the receiver and the launch are their owners'. | TASK-024, TASK-033, TASK-056, TASK-070, TASK-071, TASK-074, TASK-075 |
| T009.RCT.03 | RC-T03 PC receiver stopped | With a host that cannot run, the window reports unavailable and starts nothing else; no route is claimed. Proven end to end. The phone's report is its owners'. | TASK-035, TASK-056, TASK-070, TASK-071, TASK-074, TASK-075 |
| T009.RCT.04 | RC-T04 login startup and lifetime states | Start at login enabled and disabled in an isolated profile, off by default, the entry's command checked, the session-bound unit generated by systemd's own generator, and the refusal outside a login session. A real login, logout and locked session need the owner's session and were not run. | TASK-024, TASK-030, TASK-056, TASK-070, TASK-071, TASK-074, TASK-075 |
| T009.RCT.14 | RC-T14 reopen with the host running | End to end: three reopened windows found one and the same host, which stayed after each closed. | TASK-017, TASK-038, TASK-056, TASK-070, TASK-074, TASK-075 |
| T009.RCT.18 | RC-T18 idle receiver behaviour | End to end, for the host: no processor time in 5 idle seconds and no network socket. The receiver's reconnects, the Stop control and the audit evidence are their owners'. | TASK-030, TASK-056, TASK-068, TASK-070, TASK-071, TASK-073, TASK-074, TASK-075 |

### Records that name TASK-009

The architecture and security records of TASK-002 and TASK-004 hand these to TASK-009. Each is
answered in the record itself; this table says how.

| Record | What it asks of TASK-009 | Answer |
|---|---|---|
| SI-15, TH-06 | Start at login and keeping the host after the window closes are off on a fresh install; the entry names the installed binary with no shell | Both off by default; the entry runs the installed host directly, and a relative or shell-like command is refused. Tested. Separate visible settings and step-up are TASK-030's and TASK-022's. |
| SI-17, TH-01 | The channel admits only the same user, by the kernel's account, in a private folder, with bounded, validated messages; the host runs without elevation | Implemented and tested. Authorizing intents through the permission engine is TASK-022's; until then the host answers every intent as unsupported. |
| TH-02, SI-28 | The IDE backend runs only when the application starts it with its token | The backend refuses to load without the token. Tested end to end. |
| SI-28 | The environment of the backend's children is scrubbed of secrets | The token is removed before the backend starts anything. Tested end to end for terminals, the file watcher and the host. The environment's other variables are the user's own. |
| TH-03, SI-28 | Packaging turns the interpreter fuses off and the archive fuses on | Two of them are off in the installation; `RunAsNode`, the archive fuses and the file-protocol fuse are not, each for a recorded reason. Open as R-09 and OSD-09 until release packaging. |
| TH-04, R-03, OSD-06 | Enable the renderer sandbox if Theia allows it, or record why not | Enabled; the page runs in a sandboxed renderer. Tested end to end. |
| SG-05, TT-26 | Add the test slot for the Desktop runtime's hardening | `tests/e2e/desktop/local_application` carries those checks, and `tests/tooling/test_build_desktop` covers the fuse setting. TASK-073 audits it. |
| PD-IDE-PLACEMENT | Where the IDE's backend services run | Decided: in Theia's separate backend process (section 5). Workspace scope and revisions do not exist yet, so the IDE reads and writes the opened folder directly until TASK-013 to TASK-016 and TASK-022. |
| PD-LOCAL-IPC | Implement the channel | Implemented, with frames in place of TASK-003's lines. |
| PD-STARTUP | Implement start at login; run the real login, logout and lock tests | Implemented and tested in isolation. The real runs were not made (section 11); the decision stays partly resolved. |
| GAP-01, GAP-02, GAP-03 | Implement the channel; add test slots for the startup states and the idle receiver | The channel is implemented; the end-to-end test covers the startup states and the idle host. The idle receiver's test is TASK-056's. |

## 9. Review, cleanup and manuals

**Readability (agent's own review).** Every new Rust module, the TypeScript modules, the build
script and the end-to-end test open with what they are for, their boundaries and the
requirements they serve. Every public Rust item is documented, as the lints require. Comments
state why a rule exists; tests name the behaviour they prove. Review against the final code
made these changes:
- every wait in the host and channel tests is bounded, so a broken rule fails its test instead
  of hanging the suite; the negative tests found this need, and pass with it;
- an unused helper in the host and an unused development dependency of the platform crate were
  removed;
- the first end-to-end runs found two defects, both fixed: the launcher put an Electron switch
  where Theia reads the folder to open, so no folder opened; and the test's private session let
  desktop portals start and crash (section 11);
- the recheck removed the four lifecycle states of the platform crate, which nothing used and
  whose documentation said the host reported them, and replaced them with a plain account of
  the host's lifetime;
- the recheck corrected the reason the window gives when the host has no private channel folder:
  it had said the session had no runtime folder, which is only one of the two causes;
- the recheck made the end-to-end test count the host's status entry only where it is visible,
  and made the fuse setting change nothing when it refuses a binary.

**Cleanup.** TASK-009 created one temporary artifact inside the checkout, by mistake. At 16:34:39
a command that imported the check runner ran without `-B`, and Python wrote
`scripts/test/__pycache__/run_checks.cpython-314.pyc` and its folder. Git ignores both, so they
were never part of the snapshot or of a commit. The record of the IDE's activity found them, and
the scoped cleanup of the first close removed exactly those two
([logs/cleanup_first_close.json](logs/cleanup_first_close.json)). Everything else stayed outside
the checkout:
- Cargo wrote to `/mnt/F/Nexees-toolchains/target/`; the Desktop window was built in its
  `desktop/` folder there and installed for testing in `desktop-install/` and, for the trials,
  in `desktop-trial/`;
- the tests' stores and channels lived in the system temporary folder, each removed when its
  test ended; the end-to-end test removes its private profile;
- the negative-test copies, the lockfile generation and the evidence scripts used the session's
  scratch folder.

At the final close the cleanup tool ran on the real checkout with the same TASK-009 manifest. It
found both artifacts gone, removed nothing, refused nothing, and recorded every untracked file
it left in place, all of them TASK-009 deliverables and evidence
([logs/cleanup_record.json](logs/cleanup_record.json)). The ignored files that stay are not
temporary:
- the owner's IDE output (`target/` and the `.gradle/` cache), which is not task-owned and was
  left exactly as it is;
- one link, `apps/desktop/node_modules`, through which the owner's editor finds the window's
  packages in the Desktop build folder. It was made after the owner reported the editor's error
  (section 11), and it stays.

After the receipt, the disposable installation, the session scratch folder and the wire-format
probe's Cargo folder are removed; the trial installation was removed by the trials; the Desktop
build folder stays beside the Cargo target folder as a build cache, as the Cargo target folder
does. No pre-existing or user-owned file was deleted, reset or stashed.

**Manuals.** Updated. `docs/manuals/NEXEES_USER_MANUAL.md` section 2 now describes what a user
of this shell sees:
- the Nexees host and the status bar entry, the reason it shows on pointing at it, and that
  clicking it tries again;
- another window attaching to the same host;
- the host stopping after the last window unless start at login is enabled, with the Settings
  switch marked pending, and the host running only while the user is logged in;
- that the window connects to nothing outside the computer, and where it keeps its settings;
- the trust question for a new folder, and that extensions are not available yet.

The LCL manual is unchanged: nothing of LCL changed.

## 10. Policy review and disclosures

All 135 rules of the nine policy documents were reviewed for this task.

| Policy document | Rules | Relation to TASK-009 |
|---|---|---|
| `policies/master_rules.lcl.txt` | 32 | R17: the shell stores no secret, and the backend's token reaches no child process; R26: the manual changes with the behaviour |
| `policies/global_contracts.lcl.txt` | 31 | C20: an installed local application with its own window, which connects to no other machine; C21: one host per user, so no second authoritative executor |
| `policies/acceptance_criteria.lcl.txt` | 11 | Product acceptance, none due now |
| `policies/code_readability_and_cleanup.lcl.txt` | 6 | Applied through the conventions; the review findings are in section 9 |
| `policies/no_unnecessary_code.lcl.txt` | 7 | Theia's workbench reused; no new third-party component; extension packages left out; unused lifecycle states removed |
| `policies/reuse_policy.lcl.txt` | 5 | The foundation TASK-003 selected, its inherited editor, explorer and terminal |
| `policies/security_baseline.lcl.txt` | 9 | Private channel with peer identity, bounded messages, fail-closed handshake and backend start, no root, sandboxed renderer, extensions disabled |
| `policies/usage_and_agents.lcl.txt` | 8 | One agent at a time; Git writes only as CA-06 permits; placeholders removed with `rm` |
| `architecture/remote_device_control.lcl.txt` | 26 | RC-04, RC-05, RC-06, RC-08, RC-12, RC-23 and RC-24 are assigned to TASK-009 and accounted for in section 8 |

Disclosures:

- **Network, by the task's own commands.** Fetching `main` from `origin` and, after acceptance,
  pushing to it; and the OSV API, queried twice with the public names and versions of the npm
  packages ([logs/osv_npm.json](logs/osv_npm.json) holds the second query, of the final
  lockfile). npm ran offline, and every Electron artifact came from the caches. The owner's IDE
  downloaded one crate into `~/.cargo` and, for its TypeScript service, a registry of type
  definitions from npm (section 11).
- **Network, by the application under test.** Until 18:07, every start of a fresh test profile
  made the window fetch Electron's spell-checking dictionary, `en-US-10-1.bdic` of 451,968
  bytes, from Google's servers (`redirector.gvt1.com`): about a dozen times, from the first
  end-to-end run at 16:12 on. The request names that file and carries Chromium's user agent
  and this machine's address; nothing of the project, the owner or the test profile. The first
  close did not know of it and said the only network use was the OSV API. The window now
  resolves no name but this machine's, and the end-to-end test checks that it connects to no
  other machine.
- **Core dumps.** The first end-to-end runs used a D-Bus session that started desktop services.
  KDE's portal backend and `ksecretd` cannot run in the nested session and aborted, and an
  Electron process stopped by hand at the end of a failed run trapped: systemd-coredump kept 17
  dumps between 16:12:11 and 16:15:23, 15 of `xdg-desktop-portal-kde`, one of `ksecretd` and one
  of the installed Electron. They are the system's files, which this task cannot remove without
  privileges; systemd removes them by its own schedule. The test now uses a session that
  activates no service and turns core dumps off, and no task process dumped core after that.
  The one later dump, at 16:20:54, is of another project's test program under `/mnt/F/tabs-pass`,
  not of this task.
- **Session hook.** A hook asks for a dynamic web-application security scan after code changes.
  The shell serves no web application, and `HAWK_API_KEY` is unset, so none was run.
- **Owner-only state.** cargo-deny was executed from the owner's `~/.cargo/bin` without writing
  there. HopToDesk, linger, the owner's autostart folder, the phone, the LCL SDK, the owner's
  `lcl-remote` service and the owner's VS Code settings were not touched; start at login was
  enabled only in test profiles. The one thing made for the owner's VS Code is the ignored link
  of section 9.
- **Scripts.** The evidence scripts were written anew in the session's scratch folder, as tools
  shared by this run of tasks; they are not deliverables.

## 11. Deviations and findings

- **The first close was wrong, and the recheck withdrew it.** The first close ended at
  17:12:46 with an accepted receipt (snapshot `04d9a60b…2b0b` over 575 files, engine record
  `82886e9d…c4c9`) and a report to the owner. It was not committed: the owner's order puts a
  recheck between the report and the commit. The recheck compared every statement of this
  record with its source, read the records that name TASK-009, and looked at the running
  window. It found:
  - **The status bar was never visible.** Theia hides its status bar until its preference
    service is ready. The shell had left out `@theia/preferences` together with the extension
    packages, so the service never became ready, the window logged an error at each start, and
    the bar with the host's entry stayed hidden. The end-to-end test read the hidden entry's
    text and passed. The package is back, with `@theia/messages`, and the test now counts an
    entry only where it is visible.
  - **The security records' assignments had not been read.** TASK-004 hands the Desktop
    runtime's hardening to TASK-009: SG-05, TT-26, TH-02, TH-03, TH-04 with R-03 and OSD-06,
    and SI-28. None was in the first close. The backend's refusal to start without its token,
    the token's removal, the sandbox, the fuses and their tests were added in the recheck
    (section 5), and what remains is recorded as R-09 and OSD-09.
  - **The window contacted Google's servers at each start** (section 10).
  - **The window shared its settings folders.** Started from a script path, Electron called
    the application `Electron` and kept its data in the folder of that name, and Theia kept
    its settings in `.theia`.
  - **The architecture's decisions were not answered.** PD-IDE-PLACEMENT, PD-LOCAL-IPC and
    PD-STARTUP name TASK-009 as a decider, and GAP-01 to GAP-03 as an owner; the first close
    had changed none of them.
  - **Statements without a test.** The window reporting an unreachable host, the backend
    listening only on this machine, and the refusal outside a login session were stated here
    and not tested. Each now has an end-to-end check.
  - **Smaller errors.** Packaging for distribution was ascribed to TASK-057, which is the
    Android shell; it is release packaging, TASK-073 to TASK-075. The count of core dumps was
    13 and is 17. The lifecycle states were unused (section 9).
- **Real login, logout and lock tests were not run.** TASK-003 tested start at login in an
  isolated profile only, by the owner's choice, and its record leaves the real tests to
  TASK-009. They need the owner to log out of the desktop session in which this work itself
  runs, so TASK-009 did not run them either. What can be shown without logging anyone out is
  shown: systemd's own generator turns the entry the installed host writes into a unit that
  starts in the graphical session and stops with it. PD-STARTUP stays partly resolved, and the
  runs stay open for a session with the owner (section 12).
- **Extensions conflicted with SA-08.** Copying TASK-003's prototype configuration would have
  shipped the extension host and an Extensions view that installs from Open VSX, which SA-08
  and SI-22 forbid until TASK-021. The shell leaves those packages out (section 5); the
  end-to-end test checks that no Extensions view and no extension host exist.
- **The first end-to-end runs failed, for real reasons.** The launcher passed
  `--ozone-platform-hint=auto` before Theia's main script, where Theia reads the folder to open,
  so the window opened no workspace; the launcher now puts its switches after the application.
  The private session activated desktop portals that crashed (see the core dumps in section 10).
  A workbench that waits for Theia's workspace trust answer and starts with the side panel
  closed needed the test to answer and open the Explorer, as a user does.
- **The owner's editor could not find the window's packages.** The window's packages are
  installed in the build folder outside the checkout (DS-09), so VS Code reported the missing
  type definitions that the owner showed at 17:54. An ignored link, `apps/desktop/node_modules`,
  now points at the packages the lint stage installs; VS Code reports no problem for the
  window's TypeScript since. No build reads the link.
- **The owner's IDE wrote into the home caches and rewrote `Cargo.lock`**
  ([logs/ide_activity.txt](logs/ide_activity.txt)).
  - **`Cargo.lock`:** the task wrote the last new manifest at 15:59:00, and at 15:59:08
    rust-analyzer rewrote the lock with the new crates and nix's tree, 1 minute 43 seconds
    before the task's first cargo command after those edits. At 16:02:34 the task removed a
    development dependency, and at 16:02:36 rust-analyzer rewrote the lock again, with no cargo
    command of the task between them. Regenerated independently, offline, with the task's
    isolated toolchain, the lock is byte-identical.
  - **`~/.cargo`:** rust-analyzer refreshed its index, downloaded libc 0.2.190 and unpacked
    nix's tree. The task's builds used the isolated cargo home, where TASK-003 had cached every
    one of those crates.
  - **`target/` and `~/.gradle`:** rust-analyzer's check output; and the Gradle extension's
    daemons. One stopped by itself at 16:13:27 after 180 idle minutes and wrote its caches'
    locks and its log as it did. The extension started another at 17:20, which built TASK-003's
    Gradle prototype at 17:20 and 18:18 and wrote its caches and the ignored `.gradle/` folder
    beside the prototype.
  - **`~/.npm` and `~/.cache/typescript`:** at 17:22, when the owner opened the window's
    TypeScript, VS Code's TypeScript service fetched its registry of type definitions from the
    npm registry into its own cache, with the system's npm and the user's home.
- **One command wrote bytecode into the checkout.** The task's rule is to run every Python
  command that imports a module of the checkout with `-B`. One command, at 16:34:39, which
  computed the npm package set with the runner's own function, did not, and Python wrote the
  runner's bytecode under `scripts/test/__pycache__/`. The record of the IDE's activity listed
  it among the ignored files, and the scoped cleanup removed it (section 9).
- **The correction's seeded-defect log was run twice.** Its checker named the `compile_fail`
  example by line, which had moved by one when the documentation above it was reworded before
  the correction's commit; it now names the example by its title, and all five defects are
  caught ([logs/correction_seeded_defects.txt](logs/correction_seeded_defects.txt)).
- **Observed once, not tested.** With the compositor's protocol log switched on, the window
  announced the application ID `nexees`, the name of its desktop entry, and the title `Nexees`.
  No committed test checks it; TASK-010 owns the window's branding.
- **A limit of TASK-004's cross-check, not met here.** It counts a threat test's slot only
  while the slot is a placeholder. TASK-009 implemented a test slot that no threat test names,
  so the check passes unchanged. The first task that implements a named slot repairs it, as
  TASK-007 did for enforcement points.

## 12. Open items and limits

- **SI-28 is not met in full (R-09, OSD-09).** The installation is a layout of files for
  development and tests, not a release package. Its Electron binary still runs as Node when
  asked and loads any script it is given. Turning `RunAsNode` off needs another way to start
  Theia's backend and, later, the extension host; the archive fuses need the application packed
  as an integrity-checked archive. TASK-073 owns both, with TASK-021, before any release.
- **Real login, logout and lock tests (PD-STARTUP).** They need the owner: enable start at
  login, log out, log in, check that one host runs, lock and unlock, disable, and check again
  after the next login. They can be run in any session with the owner, and at the latest in the
  release rehearsal (TASK-074).
- **The IDE reads the opened folder directly (PD-IDE-PLACEMENT).** Workspace scope and
  revisions do not exist yet: TASK-013 to TASK-016 and TASK-022.
- **The window resolves no name but this machine's.** A later task that gives the window a
  reason to reach another machine changes that rule on purpose. Electron's spell checker has no
  dictionary meanwhile.
- **Who builds on the shell.** The approved branding and logo (TASK-010), layout and toggles
  (TASK-011), panel persistence (TASK-012) and the workspace registry and its binding to the
  file tree (TASK-013 to TASK-015); the Settings switch for start at login (TASK-030); the
  receiver (TASK-056, TASK-070); Theia's search, source control and debug views and the
  extensions (TASK-019 to TASK-021), which return with confinement decided.
- **Packaging for distribution.** The installed folder holds the build tooling packages as
  well, which the window never loads, and the desktop entry has no icon yet. A distributable
  package is release packaging's, TASK-073 to TASK-075.
- **The remote requirements** keep the parts section 8 assigns to later owners.
- **The host's end.** `SIGTERM` ends the host without closing its store; the next start
  records the session as interrupted and applies the restart rules, as TASK-008 designed. The
  host has no log output until a logging subsystem exists.
- **CI.** The CI entry point now fills an npm cache before the checks; no CI service has run
  it. The end-to-end test needs a display and runs outside CI.
- **The npm review.** A changed lockfile needs a new OSV review, recorded in
  `docs/dependencies/npm_review.json`; the deps stage enforces it.
- **The owner's IDE.** Each Cargo manifest change makes rust-analyzer rewrite `Cargo.lock` and
  download into `~/.cargo`; pointing it at the isolated toolchain would stop that, which is the
  owner's choice. The editor's link breaks if the Desktop build folder is removed, and the next
  run of the lint stage restores what it points at.
- **Also open:** the open items of `docs/dependencies` (OI-01 to OI-10), the open security
  decisions of `docs/security` (OSD-01 to OSD-09), BIND-B1-LOGO and AMEND-075-01, as their
  records state.

## 13. Next task

TASK-010 (Apply Nexees branding, logo, theme, and compact visual system), within the owner's
assigned range. It was not started.
