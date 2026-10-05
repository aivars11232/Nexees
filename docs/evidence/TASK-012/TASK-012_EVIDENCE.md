# TASK-012 evidence: Implement core Desktop panel/view persistence

**A draft at the checkpoint of 2026-10-05, not a closed record.** TASK-012 is not accepted.
What this draft says of the final checks, the seeded defects, the debugging sessions and the
receipt is not yet true. [CHECKPOINT.md](CHECKPOINT.md) says what is done and what is still
to do. This note goes when the task closes.

| | |
|---|---|
| Task | TASK-012 — Implement core Desktop panel/view persistence |
| Date | 2026-10-05 |
| Performed by | Coding agent (Claude Code, model Claude Fable 5.1), one primary session; no sub-agents, reviewers or background AI jobs |
| Checkout | `/mnt/F/Nexees/`, branch `main`, base `8bfd07ace5a62f2c6bf025356ff5d2d754123afc` |
| Repository | `https://github.com/aivars11232/Nexees` |
| Specification pack | `/mnt/F/Nexees_LCL_Implementation_Pack_v0.5.3/`, archive revision 0.5.3, specification version 0.5.0 |
| Pack content identity | `bd5e8fce26f542c44a891e4340da3b2a84f79ab368b5cb868beb48126c2db3e1` |
| Pack manifest SHA-256 | `8d9d0080ac13a3fb17cc70603b77eb3b11abcd3fb6d2fad515cfac136fcf07e4` |
| Procedure | Continuation profile 0.5.3 (CA-01 to CA-12), as adopted for TASK-002 and continued for this task |
| Predecessors | TASK-001 (`284f97a`), TASK-002 (`d1a673f`), TASK-003 (`2eaa199`, receipt decided in `bccc9b1`), TASK-004 (`0337be0`), TASK-005 (`e83dcd7`), TASK-006 (`fc42a93`), TASK-007 (`9f28c37`, follow-up `1c85864`), TASK-008 (`2bf47a2`), TASK-009 (`bba49c3`), TASK-010 (`53448f2`) and TASK-011 (`8bfd07a`), all accepted; `d00bb83` is the owner-directed correction, and `f0c713a` and `7414e6a` are TASK-010's two checkpoints, none of them a task commit |
| Commits of this work | None while the task worked. The closing commit is made after acceptance, as CA-06 permits, and is recorded by Git, not in this file |

## Status

The deliverables are complete and every check passed on its final run. Acceptance is recorded
in [receipt/RECEIPT_RESULT.md](receipt/RECEIPT_RESULT.md), which is written after this file
because the receipt contains a snapshot of everything else.

TASK-012 made the Desktop window remember its panels.
- **What is remembered:** which of the left sidebar, the right sidebar and the bottom panel
  are shown, how wide or high each is, and which view each has selected. A sidebar that is
  hidden keeps the view and the width it comes back with.
- **The host keeps it.** The window gives its panel layout to the Nexees host, which keeps it
  in the device's state store, and gets it back when it starts. The layout is the first view
  state of a client that the store holds (ST-VIEW).
- **It survives the window.** The panels come back after the window was closed, after it was
  stopped without closing, after a change made in the moment before it closed, and on a
  profile in which the foundation has stored nothing.
- **One typed record from end to end.** The layout is a record of the shared domain model,
  carried by two new messages of the window's channel. That made protocol version 2 and
  schema version 2 of the state store, the store's first real migration step.
- **Checked on the installed application.** Eight new end-to-end checks change the panels as
  a user would, ask the host what it keeps, and close, stop and reopen the window. Defects
  seeded against those checks are caught.

What TASK-012 does not do: keep a layout per workspace, or the tabs, the tree and the editors
of a workspace (TASK-013 to TASK-016). Which view stands in which panel, and the open
editors, stay in the foundation's own stored layout (section 5).

Four things the owner should know before relying on this record, each told in full in
sections 5, 11 and 12:
- every window of the user is one client for now, so several open windows share one panel
  layout: the next window gets what any of them stored last;
- where a view stands, and the open editors, are still the foundation's to keep, in the
  window's profile. After a window that stops without closing they come back as they were
  some seconds before it stopped, not to the last moment;
- the store's schema went from version 1 to 2. A host of this build upgrades an existing
  store when it opens it; the build before this task then refuses that store, as designed;
- nobody has seen this on a real screen: every window ran in a private test session.

## 1. Authority

The owner's instructions, verbatim, with the times they arrived (CEST).

On 2026-10-04, during TASK-009, giving the range and the order of each task's close
(TASK-009's to TASK-011's records quote them too):

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

On 2026-10-05 at 06:23:02, resuming the session after the night:

```
look where u finished yesterday and resume till task 15
```

TASK-012 is inside that range and began when TASK-011 was committed, pushed and cleaned up.
It needed no decision of the owner.

This record covers TASK-012 only.

## 2. Checkout preflight (binding B26)

| Item | Observed |
|---|---|
| Working directory | `/mnt/F/Nexees/`; the Git worktree root, with no parent repository, superproject or second worktree |
| `origin` fetch and push | `https://github.com/aivars11232/Nexees.git`; no URL rewrite, no separate push URL |
| Branch and HEAD at task start | `main` at `8bfd07a`, equal to `origin/main` after a fetch of that branch only, without tags |
| Hooks | None besides Git's samples; `core.hooksPath` unset |
| At task start | Clean: no staged, modified or untracked file, no stash, 655 tracked files. The owner's IDE keeps ignored output in the checkout: 562 files under `target/` and 17 under TASK-003's Gradle prototype (`.gradle/`), and the editor's link `apps/desktop/node_modules`; five IDE processes |
| Result | Passes ([logs/preflight.txt](logs/preflight.txt)) |

TASK-012 began at 2026-10-05T13:05:31+02:00, after TASK-011 was pushed and its scratch files
were removed. That time is the baseline of every "nothing written since" check.

## 3. Pack, native verification, dispatch and predecessors

**Pack.** All 201 manifest entries match their SHA-256, with no unlisted or missing file. The
content identity recomputes to `bd5e8fce…b3e1`. `tasks/task_012.lcl.txt` (`5b1112b6…ba47`)
equals the `after_sha256` of `PROCEDURAL_CHANGES.json`.

**Engine.** `lcl 1.0.0`, the engine that evaluated TASK-011's acceptance; the LCL repository
is at `ada0b5c`, clean, with the revision the pack binds in its history and the canonical
packages unchanged since it.

**Native verification.** The pack's runner passed all 51 synthetic language cases with that
engine ([logs/native_language_suite.json](logs/native_language_suite.json)). These are
language tests, not evidence for any task.

**Dispatch.** `lcl run … --input input.task_number=12 main.lcl.txt` ran `task.dispatch` and
`task.task_012` only, with `status.succeeded` and no diagnostics. It published the TASK-012
packet (SHA-256 of the packet text `66d48e51…982a`), with `procedure_amendment_applies: TRUE`
([logs/dispatch_task_012.record.json](logs/dispatch_task_012.record.json)).

**Predecessors.** All eleven engine records are intact and `status.succeeded`, and all their
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
| TASK-009 | `d65fa84a…7245` |
| TASK-010 | `94da16cc…a6cd` |
| TASK-011 | `b104bd8e…9659` |

At task start no predecessor deliverable or evidence file had an uncommitted change.

## 4. Required reading

The session's context had been compacted during TASK-011. So the documents the task names as
required reading were read again from the pack, in full: the task itself as the packet
delivers it, the Desktop UI contract, the continuation profile, the two code policies, the
manual contract, the master rules, the global contracts, the bindings, the stop conditions,
the reuse matrix and the usage guide. Three documents of the mandatory order that bear on
what this task keeps were read again with them: the persistent state model, the workspace
model and the acceptance criteria. The rest of the mandatory read order is byte-identical to
its last recorded read and was reused ([logs/required_reading.txt](logs/required_reading.txt)).

The repository's records were searched for every assignment to TASK-012 before the work
began; section 8 answers each. The code the layout passes through was read: the domain's
client module, the protocol's messages and version negotiation, the state store with its
migration registry, the Desktop host and the window's backend. Theia's own code was read for
how it stores and restores its layout, where, and when.

## 5. What was decided and built

### Where the window's view state is kept

The task says to persist panel widths, visibility, selected tabs and layout state. Theia
already stores its whole layout and restores it, so a first reading is that nothing is left
to build. The repository's own records say otherwise, and they bind this task:

- the state catalogue places a client's view state (ST-VIEW) in "the device's state store,
  keyed by UI client and workspace and written only on behalf of that client", and names
  TASK-012 as a schema task of it;
- the failure domain of a UI client (FD-UI-CLIENT), with TASK-012 as an owner, says that
  when the window crashes or is closed "view state was persisted", and that the client
  "reattaches to the running host and restores its view state";
- the domain's client module says that the layout details of a client, panels among them,
  "are added by TASK-012 and TASK-016";
- the persistent state model of the pack asks for an explicit schema version, migration
  support and crash-safe writes, and for no redundant state store.

Three ways were weighed.

| Way | Why it was or was not taken |
|---|---|
| Leave it to Theia's own stored layout | Not taken. It lives in the window's profile, is written only when the window closes, and is no record of the shared model: against ST-VIEW and FD-UI-CLIENT |
| Put Theia's whole layout document into the state store | Not taken. It is the foundation's private format, not a type both clients share (C15), it has no schema of ours to version or check, and it is far larger than a message of the window's channel may be (4096 bytes) |
| A typed record of the panels through the host, and Theia's layout for the rest | Taken |

So the three panels the Desktop UI contract places around the editor are a typed record that
the host keeps, and the rest of the layout stays where the foundation keeps it until the
workspaces have their own view state (TASK-016).

### The record

`core/domain/client.rs` gains the layout of a client's panels, beside the client's
foreground workspace:

- a **panel** is whether it is shown, its size when it was last shown, in density-independent
  pixels, and the view selected in it when it was last shown. A hidden panel keeps both, so
  that it comes back as it was;
- a **panel layout** is the left sidebar, the right sidebar and the bottom panel. It exists
  only when it is valid: each size is from 1 to 16,384, and a shown panel has one;
- a **client layout** is the layout with the client and the device, the record the store
  keeps: schema `nexees.client.layout`, version 1, never synchronized.

A view is named by an identifier of its own type, with the alphabet and length of every other
identifier. A view whose name is no identifier is not remembered. Decoding is strict: no
unknown field, at any level.

### The store: schema version 2

The state store keeps a list of the record schemas it holds, and its rule is that the list
changes only with a migration step, so that an older build refuses a store it would misread.
TASK-012 adds the first step after the initial one:

- **step 2** adds the record and changes no table, because the store's one table of records
  holds every schema. The step exists for its version;
- a host of this build upgrades a store of version 1 when it opens it. Every record the
  store held is byte-identical afterwards, which the migration machinery of TASK-008 checks
  before it commits and a new test repeats on a populated store;
- the build before this task refuses the upgraded store untouched, which the same test shows.

The tests of the migration registry had been written for a build whose only step is the
first. They now speak of "this build's version" and "the version after it", and pass with
two steps as they did with one.

### The messages: protocol version 2

The window's channel (IF-ATTACH) carried intents in and status and events out. It gains:

| Message | From | Meaning |
|---|---|---|
| `load_layout` | window | Asks for the layout the host keeps for this client |
| `store_layout` | window | The panels as they are now, to keep in place of the layout kept before |
| `layout` | host | The layout the host keeps, or none: the answer to both |

- **A window names no client.** The message holds the three panels and nothing else. The
  host keeps them for the client on that channel.
- **Version 2.** A new message is a new protocol version. Both ends still speak version 1,
  and two ends that agree on it do without the layout messages: the window does not send
  them, and the host closes the channel on one, as it does on any message outside the
  agreement. So a window and a host of different builds still work together after an update.
- **Small.** A layout is a few hundred bytes, within the channel's limit of 4096.

### The host

The host keeps the layout its windows store, under one client, `desktop-window`, and gives
it back to a window that asks.

- **One client for now.** Windows are told apart only once they show different workspaces.
  Until then every window of the user is that client, and the layout is the window's, not a
  folder's.
- **The store is shared with the window threads** behind a lock, and taken out of their
  reach before the host closes it.
- **Nothing is read into a layout.** The host checks its shape through the domain type and
  stores it. No code takes it as a target, a binding or an instruction.
- **Fail closed.** A message that is no layout closes the channel and leaves the kept layout
  as it was. When the store cannot give or take a layout, the host closes the window's
  channel instead of answering with a guess.

### The window

The backend (`apps/desktop/src/main.ts`) carries the layout to the host and back. The host
answers a window's requests in order, so the backend matches answers to requests by their
order. A request for the kept layout waits for the first attempt to attach. While no host is
attached the backend holds on to the newest layout and gives it to the next host.

The frontend module `apps/desktop/src/shell/panel_memory.ts` restores and stores:

- **At the start**, after Theia restored its own layout or made the first one and before the
  window is revealed, it asks for the kept layout and puts the panels as it says. The window
  waits at most three seconds for the answer.
- **As the user changes the panels**, it samples them once they have stayed unchanged for
  300 milliseconds, so that a drag is one change, and gives the sample to the host. While
  changes follow one another faster than that, nothing is sampled; the panels are sampled
  when the changes stop, or when the window closes.
- **A change the host has not taken yet** is kept in the window's own profile. At the next
  start it outranks the host's copy and is given to the host. That covers a window closed
  before the change settled, and a host that could not be reached.
- **It reads and sets the panels through Theia's own interfaces:** the layout data of a side
  panel for its size and its shown view, the shell's resize, expand and collapse, and the
  tab a collapsed side panel returns to. A hidden sidebar shows no view, so its view in the
  layout is that tab: the sidebar is kept with the view it showed last even when no sample
  was taken while it showed it (section 11). A view the stored layout names but the window
  no longer has is skipped, and Theia's own choice stays.

The check of a layout's shape is written a second time in TypeScript, with the host's rules:
the host closes the channel on a message it refuses, so the window must not send one, and a
layout that comes back is input. A tooling test holds the window's numbers to the shared
core's: the protocol versions, the largest message, the largest panel size and the length of
a view's name.

### What the foundation still keeps

Which view stands in which panel, and the open editors, are in Theia's own stored layout, in
the window's profile. Theia writes it when the window closes. `panel_memory` has it written
whenever the panels settle too, with Theia's own function for it.

The browser puts that storage on disk some seconds after it is written. So after a window
that stops without closing, the panels are as the host kept them, to the last settled
change, and the rest of the layout is as it was some seconds before the stop. This was
seen both ways: a view opened twelve seconds before the stop was there again
([logs/live_observations.txt](logs/live_observations.txt)), and in the first version of the
end-to-end check, one opened about a second before the stop was not.

Where the two disagree, the host's layout wins for the three things it holds, because it is
put in place after Theia's.

### Two questions TASK-011 left to this task

- **The area of a hidden right sidebar.** Theia forgets which view a collapsed side panel
  showed. The layout keeps it, and the sidebar comes back with it.
- **The frame of an older profile.** TASK-011 found that a profile last used with the
  system's title bar keeps it, because Theia stores the frame of a profile's last window.
  This task leaves that as it is. The frame is the user's own choice of title bar style,
  which Theia's setting changes; overriding the stored value would take that choice away.
  No profile of an earlier build exists outside the test sessions.

### What was reused, and what was not added

- **Reused:** the state store with its migration registry, the versioned records of the
  domain model, the protocol's version negotiation, the window's channel with its bounds and
  its check of the peer; Theia's layout data, resize, expand and collapse of its panels, its
  storage service for the window's profile, and its own stored layout.
- **No package** was added, and no lockfile changed.
- **Not added:** a second store; a table for view state; a setting to turn the memory off;
  a command; a message that names a client; a general "view state" framework for records
  that do not exist yet. The slot `core/workspaces/view_state` stays a placeholder: it is the
  workspaces' (TASK-016), and until then the host writes the one record directly.
- **Renamed:** the contribution that gives a new profile its first layout was called
  `PanelLayout`, which is now the name of the record everywhere else. It is `FirstLayout`.

### Security considerations

- **Presentation only.** A layout says where panels stand. The host derives nothing from it:
  no authority, no target, no binding (AD-06). The interface record and the conventions now
  say so.
- **Serialized input is validated at both ends.** The host accepts a layout only as the
  strict domain type. The window checks a layout before it applies it, and uses a stored
  view name only to find a view by that exact name.
- **Bounded.** Sizes, names and the message are bounded; the store keeps one layout per
  client, replaced by the next.
- **The same boundary as before.** The messages cross the window's channel, which admits
  only the same OS user. A program of that user can store a layout and so rearrange the
  window's panels at its next start, and nothing more; the threat TH-01 and the boundary
  TB-01 of `docs/security` now name the view state. No residual risk or open decision was
  added.
- **No secret.** A layout holds two numbers, three flags and three names of views.
- **Old and new builds.** A store of a newer schema is refused untouched, and a message of
  a newer protocol version than agreed closes the channel.

## 6. Deliverables

**New files:**

| Path | Purpose |
|---|---|
| `apps/desktop/src/shell/panel_memory.ts` | What the window remembers of its panels: restoring them at the start, storing them as they change |
| `docs/evidence/TASK-012/` | This record, `logs/` and, written last, `receipt/` |

**Removed files:** none.

**Changed files:**

| Path | Change |
|---|---|
| `core/domain/client.rs` | The panel, the panel layout and the client layout record, with their tests |
| `core/domain/ids.rs` | The identifier of a view |
| `core/domain/lib.rs` | The crate documentation names the client layout |
| `core/domain/schema.rs` | The client layout in the test of the schema names |
| `core/state/state_store.rs` | The client layout among the stored records; a test of it; the tests that named schema version 1 |
| `core/state/migrations/migration_registry.rs` | Schema version 2 and its step; a test of the upgrade from version 1; the tests written for a single step |
| `core/protocol/messages.rs` | The three layout messages, and the first version each message of a window belongs to |
| `core/protocol/version_negotiation.rs` | Protocol version 2 |
| `apps/desktop/src/application_host.rs` | The host keeps and gives back the layout; its store shared with the window threads; three tests |
| `apps/desktop/src/main.protocol.ts` | The layout in the contract between the window's backend and frontend, and the check of its shape |
| `apps/desktop/src/main.ts` | The backend carries the layout to the host and back; protocol version 2 |
| `apps/desktop/src/shell/main_window.ts` | Binds the panel memory |
| `apps/desktop/src/shell/panel_layout.ts` | The shell tells the state of its bottom panel; `FirstLayout` |
| `tests/e2e/desktop/local_application.test.mjs` | Eight new checks; a window can be opened and left as it comes back |
| `tests/tooling/test_build_desktop.py` | The window's protocol numbers are the shared core's |
| `docs/architecture/interfaces.lcl.txt` | IF-ATTACH carries a client's view state; TASK-012 among its schema tasks |
| `docs/architecture/state.lcl.txt` | ST-VIEW notes what TASK-012 keeps and what follows |
| `docs/architecture/subsystems.lcl.txt` | The slots the layout passes through, and TASK-012 among the owner tasks of the host, the shared model and the store |
| `docs/security/threats.lcl.txt` | TH-01 names the stored layout |
| `docs/security/trust.lcl.txt` | TB-01 names the view state that crosses it |
| `docs/engineering/CONVENTIONS.md` | Section 15 lists the module; the new section 16 on what a client remembers |
| `docs/manuals/NEXEES_USER_MANUAL.md` | Section 2: what the window remembers |
| `README.md` | What TASK-012 built |
| `FILE_TREE.txt` | Regenerated |

No dependency was added and none removed.

## 7. Checks run

| Log | What ran | Result |
|---|---|---|
| [run_checks.txt](logs/run_checks.txt) | Every stage of the check runner, and the three core crates built for both Android targets | Nine stages pass; 96 domain, 32 protocol, 31 state, 12 platform and 12 host tests; 59 tooling tests |
| [desktop_build.txt](logs/desktop_build.txt) | The token and logo checks, the TypeScript check, the offline build from scratch, the installation | Eight commands exit 0; the installed application carries the panel memory in its page, the layout messages in its backend and the layout record in its host |
| [e2e_local_application.txt](logs/e2e_local_application.txt) | The end-to-end test of the installed application | 67 of 67 checks pass: the 59 of TASK-009 to TASK-011 and 8 of the panel memory |
| [seeded_defects.txt](logs/seeded_defects.txt) | Defects seeded against the new end-to-end checks, in three runs | In each run exactly the checks of the seeded defects fail; the restored installation passes again |
| [negative_tests.txt](logs/negative_tests.txt) | 22 defects seeded into scratch copies of the checkout, and 2 controls | All caught; the 10 test-stage cases by the named failing test |
| [live_observations.txt](logs/live_observations.txt) | An observation in a debugging session of what the end-to-end test does not repeat | As expected (section 5) |
| [regression.txt](logs/regression.txt) | The predecessors' LCL projects and cross-checks, every crate's tests, the tooling tests | 25 commands exit 0 |
| [security_scan.txt](logs/security_scan.txt) | The security stage and the scan of the changed files | No finding; no file made executable |
| [ide_activity.txt](logs/ide_activity.txt) | What else ran on the machine during the task | No Rust manifest changed; the home-cache writes are the owner's IDE's |

New tests of the crates:
- domain: a client layout round-trips with what a hidden panel keeps; a size that is none is
  refused; decoding refuses what construction refuses and any unknown field;
- state: a client's layout is one record that its next layout replaces; version 2 keeps a
  version 1 store byte for byte, and the first build then refuses it;
- protocol: the layout messages are small, strict and of version 2; this build and one of
  the first protocol agree on version 1;
- host: a window finds the panels it stored after the host restarted; a layout that is none
  closes the channel and leaves the kept one; a window of the first protocol is served but
  refused the layout messages.

The eight new end-to-end checks, in the order they run:
1. a sidebar hidden at once after an area was selected is kept with that area and its
   width. It is the first layout the window of a new profile gives the host, taken when the
   sidebar was already hidden. The test asks the host on its own channel, here and below;
2. the window gives the host its panels as the user leaves them: which are shown, their
   sizes and their selected views;
3. a reopened window has its panels as they were left: both sidebars hidden, the bottom
   panel in its height and with its view;
4. a sidebar that was hidden when the window closed comes back with the view and the width
   it had;
5. a window that was stopped without closing comes back with its panels as they last
   settled. Every process of the window is killed at once; the host goes on;
6. a change made in the moment before the window closes is not lost: the next window has it,
   and the host then keeps it;
7. the panels are the host's to keep: with the foundation's stored layout removed from the
   profile, the next window still has them, and no longer has a view that only that layout
   held;
8. what the user changed while the host could not be reached is given to the host once it
   can be. The test waits until the window's profile holds the change before it lets a host
   be reached, and makes no other change after that.

**Seeded defects.** Each run breaks the panel memory of the installed window:
- run 1: the host is given the panels without their selected views; and the bottom panel is
  restored forty pixels higher than the layout says;
- run 2: the host is given the right sidebar ten pixels wider than it is; and the window's
  last moment keeps nothing of a change that has not settled;
- run 3: the window does not ask the host for its panels when it starts; and its backend
  forgets a layout that it could not give to a host.

Each run fails exactly the checks of its defects. Run 2 also fails TASK-011's check of the
reopened window, whose width the wrong layout changes.

**Negative tests.** `cargo test` stops at the first crate with a failing test, and the
host's tests run first. So a rule of the domain that the host's tests also hold is caught
there, and each case names the test that does catch it: two cases are caught by the domain's
own tests, two by the host's test of a layout that is none.

## 8. Check accounting

### Verification

| Check | How it was met |
|---|---|
| T012.VERIFY.01 build the affected targets | The workspace builds, lints, tests and documents; the three core crates build for both Android targets; the window's TypeScript compiles with every strict check and builds with `theia build`; the application installs ([run_checks.txt](logs/run_checks.txt), [desktop_build.txt](logs/desktop_build.txt)). |
| T012.VERIFY.02 task-specific tests | Ten new tests of the domain, state, protocol and host crates; one new tooling test; eight new end-to-end checks; three runs of seeded end-to-end defects; 22 negative cases (section 7). |
| T012.VERIFY.03 regressions of the changed subsystem | Every crate's earlier tests pass with the changes, the store's migration tests among them; the 59 end-to-end checks of TASK-009 to TASK-011 pass on the changed window and host; the changed architecture and security records pass their LCL projects and cross-checks ([regression.txt](logs/regression.txt)). |
| T012.VERIFY.04 final diff inspected | Section 6 lists every new and changed file since the base; the receipt's final verification repeats the inspection. |

### Completion gate

| Check | How it was met |
|---|---|
| T012.CLOSE.01 dependencies closed | TASK-011 accepted and its lineage verified (section 3). |
| T012.CLOSE.02 objective without unrelated scope | **Objective and scope:** panel widths, visibility and selected tabs are persisted as the client's layout in the device's state store and restored; the layout state beyond them is persisted by the foundation's own stored layout, now written as the panels settle. No workspace semantics were built: one layout per client. |
| T012.CLOSE.03 build passes | As T012.VERIFY.01. |
| T012.CLOSE.04 required tests pass | As T012.VERIFY.02. |
| T012.CLOSE.05 security checks pass | The security stage and the scan are clean; the dependency gates pass with unchanged lockfiles; section 5 states the security considerations; the host's tests show a layout that is none refused. |
| T012.CLOSE.06 no unnecessary code or dependency | No package added. Section 5 gives the three ways weighed and lists what was not added. |
| T012.CLOSE.07 no workspace/agent/LCL binding invariant violated | Bound checkout, branch and remote. The pack, the LCL repository, the canonical packages and the LCL SDK were not written. One primary agent. A layout is never a target or a binding (AD-06); no agent or workspace binding exists yet for it to touch. |
| T012.CLOSE.08 evidence recorded | This folder. |
| T012.CLOSE.09 Git diff understood | As T012.VERIFY.04. |
| T012.CLOSE.10 readability and comments | Agent's own review, section 9. |
| T012.CLOSE.11 scoped cleanup | Section 9; [logs/cleanup_record.json](logs/cleanup_record.json). |
| T012.CLOSE.12 post-cleanup verification on the final revision | `receipt/final_verification.txt`. |
| T012.CLOSE.13 manual impact | Updated: section 9. |

TASK-012 is assigned no remote requirement and no scenario.

### Records that name TASK-012

| Record | What it assigns | What was done |
|---|---|---|
| ST-VIEW, schema tasks | The schema of a client's view state, with TASK-016 | The panel part: `nexees.client.layout`; the record's note says what follows |
| FD-UI-CLIENT, owner tasks | View state persisted when the window crashes or closes, and restored on reattaching | The panels, by checks 3 to 7 of section 7; the rest of the layout is the foundation's (section 5) |
| SS-DESKTOP-CLIENT, owner tasks and the slot `apps/desktop/src/shell/` | The window's shell, TASK-009 to TASK-012 | `panel_memory`; the slot's text names it |
| `core/domain/client.rs`, module documentation | "The layout details of a client, such as tabs, tree expansion and panels, are added by TASK-012 and TASK-016" | The panels; the documentation now says what TASK-016 adds |
| `README.md` and `panel_layout.ts`, as TASK-011 left them | "What Nexees keeps of the layout the user leaves behind, and where, is the subject of TASK-012" | Both now say what is kept and where |
| TASK-011's record, sections 11 and 12 | The area of a hidden right sidebar; the frame of an older profile | Section 5, "Two questions TASK-011 left to this task" |

TASK-012 was added as an owner task of the local host, the shared model and protocol, and
the state store, and as a schema task of IF-ATTACH, because it changed them.

## 9. Review, cleanup and manuals

**Readability (agent's own review).** The layout is documented where it is defined and at
each place it passes through: what a client's layout is and that it is never a target; what
a hidden panel keeps; why a migration step that changes no table exists; which protocol
version has the messages and why a window names no client; under which client the host keeps
the layout and what it does when its store fails; how the backend matches answers to
requests. The frontend module opens with what is remembered, where each part is kept and why
an unsaved change outranks the host's copy, and says why a panel to be shown gets its size
first and one to be hidden gets it last. Review against the final code made these changes:
- the first-layout contribution got a name of its own, since its old one had become the
  record's;
- a roundabout check of a view's name became a function that says what it checks;
- a second resize after expanding a panel, which changed nothing, was removed;
- a relation between two constants that a test asserted became a compile-time assertion;
- the module's note on Theia's stored layout was rewritten to say what was observed of it.

**Cleanup.** TASK-012 created no temporary artifact inside the checkout.
`scripts/test/task_cleanup.py` ran with an empty manifest, removed and refused nothing, and
recorded every untracked file it left in place
([logs/cleanup_record.json](logs/cleanup_record.json)). The disposable installation and the
task's scratch files are removed after the commit; the Desktop build folder stays as a cache.

**Manuals.** `docs/manuals/NEXEES_USER_MANUAL.md` section 2 now says what the window
remembers: which panels are shown, their sizes and their selected views; that the next
window has them again, also after the last one was stopped unexpectedly; and that a hidden
sidebar comes back with its view and its width. The end-to-end test checks each of these.
The LCL manual is unchanged.

## 10. Policy review and disclosures

All 135 rules of the nine policy documents were reviewed for this task.

| Policy document | Rules | Relation to TASK-012 |
|---|---|---|
| `policies/master_rules.lcl.txt` | 32 | R5 to R7: the store, the channel and the foundation's panels reused, nothing speculative added; R8 and R9: the layout validated at both ends, and the host closes on what it cannot serve; R13: the panels' state preserved, per workspace with TASK-016; R25 and R26: review, cleanup, and the manual changed with the behaviour |
| `policies/global_contracts.lcl.txt` | 31 | C5: the panels survive a restart and a crash of the window; C15: the layout is one type of the shared model; C17: the one new module answers ST-VIEW; C2: a layout is never authority |
| `policies/acceptance_criteria.lcl.txt` | 11 | "Versioned state migrations are tested": the first step after the initial one, with its test; "workspaces preserve UI state" is due with the workspaces |
| `policies/code_readability_and_cleanup.lcl.txt` | 6 | Applied through the conventions; the review findings are in section 9 |
| `policies/no_unnecessary_code.lcl.txt` | 7 | No duplicate state store: the layout is in the one store, and Theia's own layout is the foundation's; the second check of a layout's shape, in the window, is held to the first by a test; section 5 lists what was not added |
| `policies/reuse_policy.lcl.txt` | 5 | The selected persistence, protocol and foundation primitives |
| `policies/security_baseline.lcl.txt` | 9 | Serialized input validated and versioned; unknown fields rejected; no authority in the layout |
| `policies/usage_and_agents.lcl.txt` | 8 | One agent at a time; heavy jobs one after the other; Git writes only as CA-06 permits |
| `architecture/remote_device_control.lcl.txt` | 26 | RC-20 holds: a layout is never a targeting input; RC-23: the channel's bounds and caller check are unchanged; none assigned to TASK-012 |

Disclosures:

- **Network.** Fetching `main` from `origin` at the start, and pushing the closing commit
  after acceptance. Nothing else: npm ran offline, no advisory service was queried because
  the lockfiles are unchanged, and the application under test connected to nothing.
- **The owner's files.** The logo file in the owner's Pictures folder was read by the logo
  check and nothing in that folder was changed. No command of the task wrote in the owner's
  toolchain and cache folders: since TASK-011's mistake every tool of the task refuses to
  run without the isolated toolchain.
- **Test sessions and the login session.** The task ran the installed application only in
  private nested sessions: the end-to-end test's, and four debugging sessions of the same
  kind up to the close, counted from the session's record. The harness of a debugging
  session ends it four minutes after it starts the application, at the latest; all four
  were ended sooner by the script that used them. The owner's login session carried its lock
  mark when the task read it with `loginctl show-session` (`LockedHint=yes`, in the header
  of the end-to-end log); the mark was there before the task began (TASK-010's record,
  section 11). The task sent that session no command and did not set or reset the mark.
- **A folder beside the toolchains.** The copy of the evidence scripts in
  `/mnt/F/Nexees-toolchains/task-tools/`, outside the repository, was kept up to date for
  the rest of this run of tasks.
- **Session hook.** A hook asks for a dynamic web-application security scan after each commit.
  The application serves no web application, and `HAWK_API_KEY` is unset, so none was run.
- **Owner-only state.** cargo-deny was executed from the owner's `~/.cargo/bin`. HopToDesk,
  linger, the phone, the LCL SDK, the owner's `lcl-remote` service and the owner's VS Code
  settings were not touched.

## 11. Deviations and findings

- **The foundation's stored layout is some seconds behind.** The first version of the check
  of a stopped window opened a view, waited until the host kept the panels, and killed the
  window about a second later. The panels came back; the view did not. Theia's layout had
  been written to the page's storage, and the browser had not yet put it on disk. The check
  now covers what the host keeps, and the view that proves the foundation's layout gone is
  used in the check that removes that layout. The limit is told in section 5 and in the
  module, and the observation with twelve seconds is in the log.
- **Killing a window takes more than its process group.** The first attempt killed the
  process group of the launcher and waited for the window's backend to go, which it did not:
  the backend and the page are in other groups. The test now kills every process whose
  program is the installed window's.
- **A wrong command name.** The test first asked the command palette for "View: Toggle
  Outline View". Theia's label is "View: Toggle Outline". The run stopped at its timeout
  and the name was corrected.
- **The negative tests named the wrong tests.** A rehearsal of the whole negative suite,
  before any close, found three cases caught by another test than the one they named: the
  host's tests run first and already hold the domain's rules. The cases now name the tests
  that catch them, and two cases were added so that the domain's own tests are shown to
  catch a defect too (section 7).
- **The store's migration tests assumed a single step.** They used the first step by its
  number and expected version 1 afterwards. They were rewritten in terms of the build's
  version; no test was removed.
- **Formatting.** The first run of the check runner on the new code failed its format and
  lint stages: lines too long in three Rust tests and one Python test, code the formatter
  writes otherwise, and an assertion of a constant. All were corrected, and the runner
  passes.
- **Several windows share one layout.** Every window is the client `desktop-window`. With
  two windows open, each stores its panels as they change, and the next window to start
  gets the last layout stored. A window does not follow another's changes while both are
  open.
- **A host that answers late.** A window waits three seconds at its start for the kept
  layout. If the host answers later, the window shows the layout it has, and the next change
  of its panels is stored in place of the kept one. No test here makes a host that slow.

## 12. Open items and limits

- **Nobody has looked at the real screen.** Every window the task saw ran in a private test
  session, in memory.
- **A layout per workspace** is TASK-016's, with the tabs, the tree and the editors of a
  workspace. That task also decides whether the foundation's own stored layout moves into
  the store, and puts the client's view state behind the workspaces' slot
  `core/workspaces/view_state`, which is still a placeholder: until then the host writes
  the one record directly.
- **Telling windows apart** comes with the workspaces. Until then the limits of section 11
  hold.
- **The rest of the layout after a crash** is as old as the browser's last write of its
  storage (section 5).
- **The frame of an older profile** stays as TASK-011 found it (section 5).
- **Also open, unchanged:** the open items of TASK-011's record that this task did not
  answer: the real-screen look, the icons, moving and resizing the window with a real
  pointer; the test session and the login session's lock mark, and the real login, logout
  and lock runs of TASK-009 (PD-STARTUP); the open items of `docs/dependencies` (OI-01 to
  OI-10), the open security decisions of `docs/security` (OSD-01 to OSD-09) and AMEND-075-01,
  as their records state.

## 13. Next task

TASK-013 (Implement workspace registry and lifecycle), within the owner's assigned range.
It was not started.
