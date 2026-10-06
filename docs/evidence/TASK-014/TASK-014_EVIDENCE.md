# TASK-014 evidence: Bind foreground workspace to its file tree

| | |
|---|---|
| Task | TASK-014 — Bind foreground workspace to its file tree |
| Date | 2026-10-06 |
| Performed by | Coding agent (Claude Code), one primary session; no sub-agents, reviewers or background AI jobs. The model was Claude Opus 5.5 throughout |
| Checkout | `/mnt/F/Nexees/`, branch `main`, base `984e1cec9ebfbbbe56396c1cfe7fcce5da9d54f1` |
| Repository | `https://github.com/aivars11232/Nexees` |
| Specification pack | `/mnt/F/Nexees_LCL_Implementation_Pack_v0.5.3/`, archive revision 0.5.3, specification version 0.5.0 |
| Pack content identity | `bd5e8fce26f542c44a891e4340da3b2a84f79ab368b5cb868beb48126c2db3e1` |
| Pack manifest SHA-256 | `8d9d0080ac13a3fb17cc70603b77eb3b11abcd3fb6d2fad515cfac136fcf07e4` |
| Procedure | Continuation profile 0.5.3 (CA-01 to CA-12), as adopted for TASK-002 and continued for this task |
| Predecessors | TASK-001 (`284f97a`), TASK-002 (`d1a673f`), TASK-003 (`2eaa199`, receipt decided in `bccc9b1`), TASK-004 (`0337be0`), TASK-005 (`e83dcd7`), TASK-006 (`fc42a93`), TASK-007 (`9f28c37`, follow-up `1c85864`), TASK-008 (`2bf47a2`), TASK-009 (`bba49c3`), TASK-010 (`53448f2`), TASK-011 (`8bfd07a`), TASK-012 (`d40b75b`) and TASK-013 (`984e1ce`), all accepted; `d00bb83` is the owner-directed correction, and `f0c713a`, `7414e6a`, `34ac37c` and `fd28f27` are checkpoints of TASK-010 and TASK-012, none of them a task commit |
| Commits of this work | None while the task worked. The closing commit is made after acceptance, as CA-06 permits, and is recorded by Git, not in this file |

## Status

The deliverables are complete and every check passed on its final run. Acceptance is recorded
in [receipt/RECEIPT_RESULT.md](receipt/RECEIPT_RESULT.md), which is written after this file
because the receipt contains a snapshot of everything else.

TASK-014 made the foreground workspace decide what a window's Explorer shows.
- **Each window is a client of its own.** Theia runs one backend for all the windows of the
  application, so until now every window reached the host over one channel and was one client
  with one foreground workspace: the one opened last in any window. Now the backend attaches
  each window over a channel of its own, and the host keeps each window's foreground. A window
  that goes, or starts again, leaves its client showing nothing.
- **The Explorer shows the window's foreground workspace and nothing else.** A folder that
  cannot be a workspace, such as one inside another workspace's folder, is no longer shown
  with a warning: the window closes it, says why after it starts again, and offers the
  workspace the folder overlaps. Theia's commands that add a second folder to a window are
  removed, and a window given several folders is closed the same way, so a CODE tree and an
  LCL tree are never one Explorer.
- **The title row names the workspace.** The workspace the host confirmed stands in the
  title row, where the approved picture has it, with the icon of its kind from the shared icon
  mapping; clicking it lists the workspaces.
- **Nothing else follows.** A window's switches bind no agent and change no other window: a
  host test switches two windows back and forth while an agent session stays bound.
- **Checked on the installed application.** Three new end-to-end checks open two windows on
  two workspaces at once, close one, and look for a way to add a second folder; the checks of
  TASK-013 now read each window's workspace from the window itself. Defects seeded against
  those checks are caught.

What TASK-014 does not do: keep view state per workspace (TASK-016), bind agents (TASK-015),
or build a file tree of Nexees's own; the Explorer is Theia's.

Seven things the owner should know before relying on this record, each told in full in
sections 4, 5, 11 and 12:
- TASK-013's record says that every rule of the nine policy documents was read for that task;
  21 of the 135 were then read only in their first 700 or 330 characters. All 135 were read in
  full for this task; what the cuts left out is told in section 11;
- opening a folder inside another workspace's folder now closes it, where TASK-013's window
  showed it with a warning;
- Theia's "Add Folder to Workspace" is gone from the window;
- the panel layout is still one for all of a user's windows; per workspace with TASK-016;
- the session's context was compacted once during the task, after this record's first draft;
  the record was then checked claim by claim against the session's own record (sections 4
  and 11);
- in 2 of the end-to-end runs the second window's Explorer did not show the folder's files
  within a minute, and why is not known; in every run since, and in every repetition made to
  catch it, the files showed (sections 11 and 12);
- nobody has seen this on a real screen: every window ran in a private test session.

## 1. Authority

The owner's instructions, verbatim, with the times they arrived (CEST).

On 2026-10-04, during TASK-009, giving the range and the order of each task's close
(TASK-009's to TASK-013's records quote them too):

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

On 2026-10-06 at 17:23:03, after the second pause of TASK-012 (TASK-012's record, section 1):

```
Resume
```

TASK-014 is inside that range and began when TASK-013 was committed, pushed and cleaned up.
It needed no decision of the owner, and the owner sent no message while it worked; the
owner's last message, at 19:16:27, came while TASK-013 was being implemented, and TASK-013's
record quotes it. This record covers TASK-014 only.

## 2. Checkout preflight (binding B26)

| Item | Observed |
|---|---|
| Working directory | `/mnt/F/Nexees/`; the Git worktree root, with no parent repository, superproject or second worktree |
| `origin` fetch and push | `https://github.com/aivars11232/Nexees.git`; no URL rewrite, no separate push URL |
| Branch and HEAD at task start | `main` at `984e1ce`, equal to `origin/main` after a fetch of that branch only, without tags |
| Hooks | None besides Git's samples; `core.hooksPath` unset |
| At task start | Clean: no staged, modified or untracked file, no stash, 701 tracked files. The owner's IDE keeps ignored output in the checkout: 598 files under `target/` and 17 under TASK-003's Gradle prototype (`.gradle/`), and the editor's link `apps/desktop/node_modules`; five IDE processes |
| Result | Passes ([logs/preflight.txt](logs/preflight.txt)) |

TASK-014 began at 2026-10-06T20:45:56+02:00, after TASK-013 was pushed and its scratch files
were removed. That time is the baseline of every "nothing written since" check. The session
was not paused during the task.

## 3. Pack, native verification, dispatch and predecessors

**Pack.** All 201 manifest entries match their SHA-256, with no unlisted or missing file. The
content identity recomputes to `bd5e8fce…b3e1`. `tasks/task_014.lcl.txt` (`f1f61170…066f`)
equals the `after_sha256` of `PROCEDURAL_CHANGES.json`.

**Engine.** `lcl 1.0.0`, the engine that evaluated TASK-013's acceptance, by its SHA-256
(`15a3e939…`); the LCL repository is at `ada0b5c`, clean, with the revision the pack binds in
its history and the canonical packages unchanged since it.

**Native verification.** The pack's runner passed all 51 synthetic language cases with that
engine ([logs/native_language_suite.json](logs/native_language_suite.json)). These are
language tests, not evidence for any task.

**Dispatch.** `lcl run … --input input.task_number=14 main.lcl.txt` ran `task.dispatch` and
`task.task_014` only, with `status.succeeded` and no diagnostics. It published the TASK-014
packet (SHA-256 of the packet text `d2ef2ad4…1eca`), with `procedure_amendment_applies: TRUE`
([logs/dispatch_task_014.record.json](logs/dispatch_task_014.record.json)).

**Predecessors.** All thirteen engine records are intact and `status.succeeded`, and all their
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
| TASK-012 | `00a34d0b…95c2` |
| TASK-013 | `d121040d…b4d7` |

At task start no predecessor deliverable or evidence file had an uncommitted change.

## 4. Required reading

The task's declaration, the workspace model and the Desktop UI contract were read in full for
this task. The continuation profile had been read again in full when the session resumed
earlier the same day, for TASK-012. For the policy review of section 10, every rule of the
nine policy documents was read in full for this task, the global contracts and the master
rules among them; bindings B2, B5 and B6, those of the visible tree, were read again. Every
other document of the required reading is byte-identical to its reading for TASK-013 and was
reused ([logs/required_reading.txt](logs/required_reading.txt)).

While the rules were read for this task, TASK-013's reading turned out to have been less than
its record says (section 11).

The session's context was compacted once during the task, at 21:31 CEST, just after the first
draft of this record was written and while a rehearsal of the negative tests ran. The work
after it went on from a summary of what was done before. The record was then read again, claim
by claim, against the code, the logs and the session's own record, and the statements of this
record about what was read, run and observed before the compaction come from those (section
11).

The repository's records were searched for TASK-014 by name before the work began, and for the
ranges of tasks that include it when this record was checked; section 8 answers each. The code
the task changes was read: the Desktop host, the window's backend, the switcher, the title row
and the icon mapping. So were the approved Desktop picture and TASK-011's record of what it
left of the title row. Theia's own code was read, as the Desktop build folder holds it, for:
the generated Electron main with its single-instance lock; how a forwarded launch opens a
window; how long a frontend's connection outlives the frontend; how the workspace service adds
a folder to a window; its workspace commands; the title bar's styles; and the workspace trust
service.

## 5. What was decided and built

### Why every window was one client

The application is single-instance: Theia's generated Electron main takes the single-instance
lock, so launching Nexees again opens another window in the running application. All of its
windows share one backend process, and the window's backend held one attachment to the host,
with one channel. The host serves one client per channel, so every window was the same
client, `desktop-window`, with one foreground workspace: the one claimed or opened last in any
window, which a window starting on no folder then closed for all of them. TASK-013 disclosed
this and left it to this task. The workspace model's own rule is that "Foreground selection is
local to each UI client", and the domain's client is what one window shows: its foreground
workspace and the panels of its window (`core/domain/client.rs`).

### Each window a client of its own

- **The backend attaches each window.** Each frontend connection of a window gets its own
  attachment, and with it its own channel to the host; when the connection closes, because
  the window goes or starts again, the attachment ends and its channel closes. Theia closes a
  frontend's connection at once when the frontend goes: its reconnection timeout is 0.
- **The host numbers its windows.** The host gives each attached window the lowest number free,
  from 1 to the eight windows it admits. The window numbered 1 is the client `desktop-window`,
  as before, and the window numbered `n` the client `desktop-window-n`. No message names a
  client: the channel decides it, as TASK-013's protocol requires, so the protocol is unchanged
  at version 3.
- **A window that goes shows nothing.** When a window detaches, the host closes the workspace
  its client showed before it frees the number, so no window that attaches meanwhile can take
  the number with the old foreground.
- **The panel layout stays one.** The host keeps the panel layout under `desktop-window` for
  every window, as TASK-012 kept it; what each workspace keeps of its view is TASK-016's.

A probe that checks whether a host answers, such as `nexees-host start` before a window
attaches, takes a number for a moment too. A window that attaches meanwhile gets the next
number; numbers are not a window's identity, and every window claims its folder when it
attaches, so that changes nothing a user sees. The host's tests do not assume which number a
window gets.

### The foreground decides the Explorer

The switcher claims the window's folder, as TASK-013 built it, when the window starts, when it
attaches to a host, and now also whenever Theia's folders of the window change. What the
Explorer shows then follows the window's foreground workspace (B2, C4, R12):
- **A folder that cannot be a workspace is closed.** When the host refuses the folder, such as
  one inside another workspace's root, the window closes the workspace its client showed,
  keeps a notice in its session storage, and starts again on the workspace it showed before,
  or on no folder. After the start it says why and offers to open the workspace the folder
  overlaps. TASK-013's window showed such a folder with a warning; TASK-013's record left this
  to this task.
- **One folder per window.** Theia can add a second folder to a window, which turns it into a
  workspace of several folders, one Explorer over both trees. Theia's two commands for it,
  "Add Folder to Workspace" and the Explorer's own, are removed once every command is
  registered. A window that is given several folders anyway, or a saved workspace file of
  Theia's, is closed the same way, so a CODE tree and an LCL tree are never one Explorer (B5,
  B6). TASK-013's record left such a window showing the folders, with no workspace.
- **Until a host answers.** A window that cannot reach a host keeps showing the folder it
  started on, and the title row names no workspace; it claims the folder once a host
  answers.

### The title row names the workspace

The approved picture shows the workspace's name in the title row, with a box beside it,
before the sidebar toggles; TASK-011's record lists it among what the picture shows for later
tasks. The row now holds the name of the workspace the host confirmed, with the icon of its
kind: two concepts added to the shared icon mapping, `workspace_code` (`package`, a box like
the picture's) and `workspace_lcl` (`law`, the LCL area's icon). It names no workspace the
window only assumes, and says "No workspace" when the window shows none. Clicking it runs
"Nexees: Open Workspace…". The row is now logo and name, menu, title, workspace, toggles and
window controls. The picture has the workspace's name in that place, after the menu and
before the toggles; its search field, where the window keeps its title, and the status dot,
the cloud and "Update" between the name and the toggles, are still later tasks', as TASK-011's
record left them.

### The visible tree is the foreground workspace's

T013.VERIFY.06's exception is gone. The end-to-end test checks it each way (T014.VERIFY.06):
each window names in its title row the workspace its Explorer shows; a new LCL workspace and
the CODE workspace opened from the list each show their own tree; two windows show two
workspaces at once, each window's own list marking its own as shown; closing one changes
nothing in the other; a window opened on a folder inside another workspace's root shows no
folder; and a window offers no command to add a second folder.

### Agents stay bound

Opening, closing and creating a workspace still write only the window's foreground. The
host's test of a bound agent now has two windows switch in turn, each between the two
workspaces, the second always to the one the first does not show: after each switch of the
second window, the first window's foreground is still its own; at the end the agent session
is unchanged, and both windows' clients, once the windows have gone, show nothing
(T014.VERIFY.05). A negative test seeds a host that carries out every window's open request
for the first window's client, and the host's test of windows told apart catches it
(section 7).

### What was reused, and what was not added

- **Reused:** the channel, its strict protocol and the host's one-client-per-channel rule;
  the state store and the registry, unchanged; Theia's single-instance windows, its workspace
  service, its file tree, its command palette, its notifications and its title bar; the
  shared icon mapping and its generator.
- **No package** was added, and no manifest or lockfile changed.
- **Not added:** a protocol version or message, a record or schema change, a file tree of
  Nexees's own, view state per workspace (TASK-016), agent bindings (TASK-015). The slot
  `core/workspaces/file_tree_model` stays a placeholder: the visible tree is Theia's, bound to
  the foreground by the switcher.

### Security considerations

The task's security requirement is to prove that no authority leaks across workspaces.
- **A window acts for its own client only.** The host decides a request's client by the
  channel; a window cannot name another window's client, and a request of one window changes
  no other window's foreground: the host's tests open, list and close across two windows, and
  a window's close of another window's workspace is refused as not open.
- **No authority in the foreground**, as in TASK-013: the agent test above, with two windows.
- **No stale foreground.** A window that goes leaves its client showing nothing, before its
  number is free again; only a host that is itself stopped first, such as by being killed,
  leaves a foreground behind, until the next window with that number claims its own folder
  (section 12). What a client shows is presentation only (ST-VIEW).
- **Fail closed.** A folder the host refuses is not shown; a window that cannot reach a host
  names no workspace.
- **The same boundary as before.** The messages and their bounds are unchanged; each window's
  channel is checked as every channel is (TB-01, RC-23). No residual risk or open decision was
  added, and no record of docs/security changed: TH-01's and TB-01's words cover each channel.
- **No secret.** The notice a window keeps across its start holds the reason and the
  workspace's ID and name.

## 6. Deliverables

**New files:** `docs/evidence/TASK-014/`: this record, `logs/` and, written last, `receipt/`.

**Removed files:** none.

**Changed files:**

| Path | Change |
|---|---|
| `apps/desktop/src/application_host.rs` | Window numbers, a client per window, the foreground closed when a window detaches; two tests added, two changed and a comment of a third |
| `apps/desktop/src/main.ts` | An attachment, and a channel, per window; what the backend serves, in its header and its error at start |
| `apps/desktop/src/workspaces/workspace_switcher.ts` | The workspace's name in the title row; claims on every change of the window's folders; a folder that cannot be a workspace closed, with a notice across the start; Theia's add-folder commands removed |
| `apps/desktop/src/shell/title_bar.ts` | The workspace's name in the row, before the toggles, with its style |
| `apps/desktop/src/shell/main_window.ts` | Binds the workspace's name |
| `apps/desktop/src/shell/design_tokens.ts` | The window's copy of the icon mapping, written again by the generator |
| `assets/theme/icon_mapping.json` | The icons of a CODE and an LCL workspace |
| `tests/e2e/desktop/local_application.test.mjs` | Three new checks; TASK-013's checks read each window's workspace from the window, and open the list of workspaces with a click on the workspace's name; TASK-011's title-row checks with the workspace's name; TASK-009's check of the backend's refusal with the error's new words; helpers for windows, the palette and the title row; a wait for a window's files that ends says what the window showed |
| `tests/tooling/test_design_tokens.py` | Looks for the uses of every token and icon in all of the window's TypeScript |
| `docs/architecture/interfaces.lcl.txt` | IF-ATTACH: each window a client of its own |
| `docs/architecture/state.lcl.txt` | ST-VIEW: the window clients and the layout kept for all windows |
| `docs/architecture/subsystems.lcl.txt` | The slots of the host, the window's backend, its workspaces and its shell; TASK-014 among the owner tasks of the local host |
| `docs/engineering/CONVENTIONS.md` | Section 17: each window a client, the foreground deciding the Explorer |
| `docs/manuals/NEXEES_USER_MANUAL.md` | Section 3: the title row, each window's own workspace, a folder that cannot be a workspace, one folder per window |
| `README.md` | What TASK-014 built; TASK-013's paragraph and TASK-009's sentence on the backend made current |
| `FILE_TREE.txt` | Regenerated |

No dependency was added and none removed.

## 7. Checks run

| Log | What ran | Result |
|---|---|---|
| [run_checks.txt](logs/run_checks.txt) | Every stage of the check runner, and the four core crates built for both Android targets | Nine stages pass; 96 domain, 33 protocol, 31 state, 12 platform, 19 host and 8 workspaces tests; 59 tooling tests |
| [desktop_build.txt](logs/desktop_build.txt) | The token and logo checks, the TypeScript check, the offline build from scratch, the installation | Eight commands exit 0; the installed window's page carries the workspace's name in the title row, the notice of a folder it closes and the add-folder command it removes |
| [e2e_local_application.txt](logs/e2e_local_application.txt) | The end-to-end test of the installed application | 78 of 78 checks pass |
| [seeded_defects.txt](logs/seeded_defects.txt) | Defects seeded against the end-to-end checks, in three runs | In each run exactly the checks of the seeded defects fail; the restored installation passes again (the second attempt; section 11) |
| [negative_tests.txt](logs/negative_tests.txt) | 17 defects seeded into scratch copies of the checkout, and 2 controls | All caught; the 6 test-stage cases by the named failing tests |
| [regression.txt](logs/regression.txt) | The predecessors' LCL projects and cross-checks, every crate's tests, the tooling tests | 26 commands exit 0 |
| [security_scan.txt](logs/security_scan.txt) | The security stage and the scan of the changed files | No finding; no file made executable |
| [ide_activity.txt](logs/ide_activity.txt) | What else ran on the machine during the task | No Rust manifest changed; the lockfile equals an offline resolution; every home-cache write is attributed, none to the task (section 10) |

New and changed tests of the host:
- a window takes the lowest number free, and each number is one client;
- windows are told apart, and a window that leaves shows nothing: two windows open two
  workspaces, each lists its own foreground, one cannot close the other's, and once both
  have gone neither client shows a workspace;
- a window's foreground switches leave a bound agent session unchanged, now with two windows
  switching in turn;
- a window creates, opens, lists and closes workspaces that outlive the host: another window
  is now another client, showing none of them.

The end-to-end test has 78 checks: the 75 of TASK-009 to TASK-013, with eleven of them
changed, and three new ones.
- New, in the order they run: two windows show two workspaces at once, each with its own
  tree, its own name in the title row and its own foreground at the host; closing one window
  changes nothing in the other; a window offers no way to add a second folder to it, with a
  control that the palette does offer a command it has.
- Changed, TASK-009's check that the backend refuses to start without the token: it looks
  for the words of the backend's reworded error (section 11).
- Changed, TASK-011's two title-row checks: the row's order now has the workspace's name
  between the title and the toggles, and the name is left to the mouse as the toggles are.
- Changed, all eight of TASK-013's checks of the workspaces. Five read the window's workspace
  from the window, its title row with the kind and icon, instead of from the test's own
  channel, which is now a client of its own: the folder made a CODE workspace, the new LCL
  workspace, the workspace opened from the list, the closed workspace and the window on a
  link. The check of the workspace opened from the list now opens the list with a click on
  the workspace's name in the title row, as the manual tells the user to. The check that the
  workspaces outlive the host, and the check that a folder inside another workspace's root is
  refused, compare the registry's entries alone, no longer the test channel's foreground. The
  check of a window opened on a folder inside another workspace's root now expects the window
  to close the folder, say why and offer that workspace.

**Seeded defects.** Each run breaks the installed window's code in one or two independent
ways, and the checks each defect reaches are named before the run:
- run 1: the title row names a workspace by its folder's name instead of its own; and Theia's
  add-folder commands stay. The checks of the new LCL workspace, of the window on a link to its
  folder, and of no way to add a second folder must fail; the CODE workspace is named after
  its folder, so its check cannot tell the two names apart;
- run 2: a folder that cannot be a workspace is shown anyway; a CODE workspace gets the icon
  of an LCL one; and a click on the workspace's name runs no command. The checks of the window
  opened on a folder inside another workspace's root, of the window's folder named as a CODE
  workspace, and of the workspace opened from the list, must fail;
- run 3, in the window's backend: every window of the application attaches over one channel,
  which stays when a window goes, as before this task. The checks of the two windows and of
  closing one of them must fail.

The host's part, that each channel is a client of its own and that a window which leaves
shows nothing, is held by the host's tests, and the negative tests break it there.

**Negative tests.** `cargo test` stops at the first crate with a failing test, and the
host's tests run first. Each test-stage case names the tests that must catch it: four cases
are caught by the host's tests, two by the tooling tests of the design tokens.

## 8. Check accounting

### Verification

| Check | How it was met |
|---|---|
| T014.VERIFY.01 build the affected targets | The workspace builds, lints, tests and documents; the four core crates build for both Android targets; the window's TypeScript compiles with every strict check and builds with `theia build`; the application installs ([run_checks.txt](logs/run_checks.txt), [desktop_build.txt](logs/desktop_build.txt)). |
| T014.VERIFY.02 task-specific tests | Two new and two changed tests of the host; three new and eleven changed end-to-end checks; three runs of seeded end-to-end defects; 17 negative cases (section 7). |
| T014.VERIFY.03 regressions of the changed subsystem | Every crate's earlier tests pass with the changes; the 75 end-to-end checks of TASK-009 to TASK-013, eleven of them changed for this task's behaviour, pass on the changed window and host; the changed architecture records pass their LCL project and cross-check ([regression.txt](logs/regression.txt)). |
| T014.VERIFY.04 final diff inspected | Section 6 lists every changed file since the base, and section 9 what the review of the final diff changed; the receipt's final verification repeats the inspection. |
| T014.VERIFY.05 foreground changes while a bound background agent stays unchanged | The host's test with an agent session bound to one workspace while two windows switch in turn (section 5); the negative case of a host that opens for the first window's client. |
| T014.VERIFY.06 visible tree matches the foreground workspace | The end-to-end checks of section 5, "The visible tree is the foreground workspace's": in every case the window's title row and its Explorer agree, with two windows each window's own list of workspaces too, and a folder the host refuses is not shown. |

### Completion gate

| Check | How it was met |
|---|---|
| T014.CLOSE.01 dependencies closed | TASK-013 accepted and its lineage verified (section 3). |
| T014.CLOSE.02 objective without unrelated scope | **Objective and scope:** each window's foreground workspace decides what its Explorer shows; LCL and coding trees are distinct workspace views, never one Explorer. Nothing beyond it: no view state per workspace, no agent binding, no protocol change. |
| T014.CLOSE.03 build passes | As T014.VERIFY.01. |
| T014.CLOSE.04 required tests pass | As T014.VERIFY.02. |
| T014.CLOSE.05 security checks pass | The security stage and the scan are clean; the dependency gates pass; section 5 states the security considerations, among them that no authority leaks across windows or workspaces. |
| T014.CLOSE.06 no unnecessary code or dependency | No package. Section 5 lists what was reused and what was not added. |
| T014.CLOSE.07 no workspace/agent/LCL binding invariant violated | Bound checkout, branch and remote. The pack, the LCL repository, the canonical packages and the LCL SDK were not written. One primary agent. In the product, the foreground never rebinds an agent (T014.VERIFY.05), and the visible tree is the foreground workspace's (T014.VERIFY.06). |
| T014.CLOSE.08 evidence recorded | This folder. |
| T014.CLOSE.09 Git diff understood | As T014.VERIFY.04. |
| T014.CLOSE.10 readability and comments | Agent's own review, section 9. |
| T014.CLOSE.11 scoped cleanup | Section 9; [logs/cleanup_record.json](logs/cleanup_record.json). |
| T014.CLOSE.12 post-cleanup verification on the final revision | `receipt/final_verification.txt`. |
| T014.CLOSE.13 manual impact | Updated: section 9. |

TASK-014 is assigned no remote requirement and no scenario.

### Records that name TASK-014

The repository's records were searched for TASK-014 by name and in ranges of tasks that
include it.

| Record | What it assigns | What was done |
|---|---|---|
| SS-DESKTOP-CLIENT, owner tasks, and the slot `apps/desktop/src/workspaces/` | The workspace views; the visible tree always the foreground workspace's | The Explorer bound to the foreground; the title row's name; the slot's text says so |
| SS-WORKSPACES, owner tasks, and the slot `core/workspaces/file_tree_model` | A workspace's file tree, including remote-only and synchronized-copy availability; the visible tree the foreground workspace's | The visible tree is bound to the foreground through Theia's own tree. A tree model of Nexees's own, with the availability of each file, is not needed for that and is not built: the slot stays a placeholder for a later owner task of SS-WORKSPACES, such as TASK-061's Android file tree |
| `docs/engineering/CONVENTIONS.md` section 17 | "telling windows apart is TASK-014's" | Each window a client of its own; the section says so |
| `README.md`, TASK-013's paragraph | Telling several windows apart, with TASK-016 | Done; the paragraph now leaves only the view state to TASK-016 |
| `README.md`, TASK-012's paragraph, and TASK-012's record | What a client keeps per workspace, TASK-013 to TASK-016 | Not this task's part: TASK-016's |
| TASK-009's record, open items | The workspace registry and its binding to the file tree, TASK-013 to TASK-015 | The visible tree bound to each window's foreground; agent bindings are TASK-015's |
| PD-IDE-PLACEMENT, as TASK-009's record leaves it | The IDE reads the opened folder directly until TASK-013 to TASK-016 and TASK-022 | Not this task's part: the IDE still reads and writes the folder a window shows directly |
| TASK-011's record, section 10 | C4, the file tree following the foreground workspace | The Explorer follows each window's foreground (section 5) |
| TASK-013's record, its status and sections 5, 8, 11 and 12, and the host's and the switcher's notes | One client for all windows; a folder that cannot be a workspace still shown; the file tree beyond Theia's own | The first two are done; the tree stays Theia's, bound to the foreground (section 5) |

Two items of earlier records that do not name TASK-014 were done too: the workspace's name
in the title row, which TASK-011's record, section 5, lists among what the picture shows for
later tasks; and a window of several folders, which TASK-013's record, section 12, left
showing them (section 5). TASK-014 was added as an owner task of the local host, because it
changed it.

## 9. Review, cleanup and manuals

**Readability (agent's own review).** Each window as a client of its own is documented where
it is decided and at each place it passes through: how the host tells windows apart and
numbers them, under which client it keeps the panel layout of every window, and that a window
that detaches shows nothing; why the backend opens one attachment per window; how the
foreground decides the Explorer, what becomes of a folder that cannot be a workspace and of a
window given several folders, and what the title row names and when. Review against the final
code made these changes:
- the backend's header and its error at start said that it serves the window that started
  it; they now say the windows of the application that started it, as does TASK-009's
  sentence in the README, and TASK-009's end-to-end check of that error looks for its new
  words (section 11);
- the tooling test that the window uses every token and icon looked at the shell's files only;
  it now looks at all of the window's TypeScript, the switcher's among them;
- the end-to-end check that no command adds a folder passed on an empty list; it now has a
  control, a command the palette does offer;
- the second window of the end-to-end test was looked at before its workbench stood; it now
  waits for it, as every first window does (section 11);
- the host's tests of windows were written not to depend on which number a window gets;
- the manual and the README say that a click on the workspace's name lists the workspaces,
  which no check did; the check of the workspace opened from the list now opens the list that
  way, and a seeded defect shows that it fails when the click does nothing;
- the state record and the conventions said that the n-th window attached is the client
  `desktop-window-n`; they now say that a window takes the lowest number free, as the host
  does;
- the manual and a comment of the switcher said that the window offers the workspace that
  holds a refused folder; a folder that contains another workspace's folder is refused too,
  and the window then offers that workspace, so both now speak of the workspace the folder
  overlaps;
- one line of the backend's header, reworded earlier in the task, was wrapped to the width of
  the rest.

**Cleanup.** TASK-014 created no temporary artifact inside the checkout, and implemented no
placeholder. `scripts/test/task_cleanup.py` ran with an empty manifest, removed and refused
nothing, and recorded every untracked file it left in place
([logs/cleanup_record.json](logs/cleanup_record.json)). The disposable installation and the
task's scratch files are removed after the commit; the Desktop build folder stays as a cache.

**Manuals.** `docs/manuals/NEXEES_USER_MANUAL.md` section 3 now says that the title row names
the window's workspace with the icon of its kind, and that clicking the name lists the
workspaces; that a folder inside another workspace's folder, or one that contains it, is
closed when you open it, with the reason and an offer of that workspace; that a window shows
one workspace of one folder and Nexees adds no second folder to it; and that each window shows
its own workspace. The sentence that every window counted as one view is gone. The end-to-end
test checks each of these, the refused folder with one inside another workspace's folder. The
LCL manual is unchanged: its section 1 says that the foreground LCL workspace shows its own
files, which stays true in every window.

## 10. Policy review and disclosures

All 135 rules of the nine policy documents were read in full and reviewed for this task.

| Policy document | Rules | Relation to TASK-014 |
|---|---|---|
| `policies/master_rules.lcl.txt` | 32 | R2 and R6: one coherent change, the Explorer bound to the foreground; R5: Theia's windows, tree and title bar reused; R8 and R9: a refused folder is not shown, and the window acts only for its own client; R11 and R12: no rebinding, and the visible tree is the workspace the user is viewing; R13: per-workspace UI state is TASK-016's; R18: the approved picture's place for the workspace's name; R25 and R26: review, cleanup, and the manual changed with the behaviour; R27: the bound checkout |
| `policies/global_contracts.lcl.txt` | 31 | C2 and C4: the foreground controls what the user sees, and the left tree follows it; C15: one protocol, with no client named in it, for every client; C17: no new file of code; C25 and C26: readability, cleanup and the manual |
| `policies/acceptance_criteria.lcl.txt` | 11 | Desktop: "file tree always follows foreground workspace" now for each window; "background agents remain bound to original workspace", as far as the host reaches it now; "workspaces preserve UI state" is due with TASK-016 |
| `policies/code_readability_and_cleanup.lcl.txt` | 6 | Q1: the window and client bindings and their state transitions are documented; Q3 and Q5: cleanup with a record, and the review findings in section 9 |
| `policies/no_unnecessary_code.lcl.txt` | 7 | No duplicate tree or store: Theia's tree and the host's store are used; section 6 lists the changed files and why |
| `policies/reuse_policy.lcl.txt` | 5 | Theia's IDE shell kept; the decoupling of what is shown from what an agent may change is Nexees's own |
| `policies/security_baseline.lcl.txt` | 9 | Fail-closed handling of a refused folder; least privilege: a window acts for its own client only; the security task rule: the abuse tests of section 7 |
| `policies/usage_and_agents.lcl.txt` | 8 | One agent at a time; heavy jobs one after the other; Git writes only as CA-06 permits; U08: every window ran in a disposable test session |
| `architecture/remote_device_control.lcl.txt` | 26 | RC-20: each device keeps its own visible workspace and file tree, now each window too; RC-23: each window's channel has the bounds and peer check of every channel; none assigned to TASK-014 |

Disclosures:

- **Network.** Fetching `main` from `origin` at the start, and pushing the closing commit
  after acceptance. Nothing else: npm ran offline, no advisory service was queried because
  no lockfile changed, and the application under test connected to nothing.
- **The owner's files.** The logo file in the owner's Pictures folder was read by the logo
  check and nothing in that folder was changed. No command of the task wrote in the owner's
  toolchain and cache folders: every tool of the task refuses to run without the isolated
  toolchain.
- **Test sessions and the login session.** Up to the close, the task ran the installed
  application only in the private nested sessions that the end-to-end test makes: its runs
  during the implementation, three runs of scratch copies of it that logged and repeated the
  second window, six more full runs made to catch a failure, three logged runs, the
  seeded-defect runs of both attempts (section 11) and the final verification's run. No other
  debugging session was used. The task read the owner's login session's lock mark only through
  `loginctl show-session`, as the header of the end-to-end log shows, and sent that session no
  command.
- **A folder beside the toolchains.** The copy of the evidence scripts in
  `/mnt/F/Nexees-toolchains/task-tools/`, outside the repository, is kept up to date for the
  rest of this run of tasks.
- **Session hook.** A hook asks for a dynamic web-application security scan after each
  change. The application serves no web application, and `HAWK_API_KEY` is unset, so none
  was run.
- **Other programs' writes in the home caches.** What other programs wrote in the user's home
  caches while the task worked, if anything, is listed with its counts and attributed in
  [logs/ide_activity.txt](logs/ide_activity.txt), section 4; none of it was a command of the
  task.
- **Owner-only state.** cargo-deny was executed from the owner's `~/.cargo/bin`. HopToDesk,
  linger, the phone, the LCL SDK, the owner's `lcl-remote` service and the owner's VS Code
  settings were not touched.

## 11. Deviations and findings

- **TASK-013's record overstates its reading of the policies.** Its section 4 says that every
  rule of the nine policy documents was read for TASK-013. The rules were read then in two
  listings, one of the eight policy documents that cut each rule after 700 characters and one
  of the remote-control rules that cut each after 330, so 21 of the 135 were read only in
  part: C19 and C20 of the global contracts, the acceptance criteria's v0.2 entry, Q1, Q3 and
  Q4, the security baseline's v0.2 rule, and RC-05, RC-06, RC-07, RC-09, RC-11, RC-12, RC-14,
  RC-15, RC-18, RC-19, RC-21, RC-24, RC-25 and RC-26. All 135 were read in full for this task.
  What the cuts left out holds one sentence on workspaces, RC-09's "No ambient
  current-workspace fallback" for remote requests, which TASK-013's row for RC-20 says in
  other words and which no part of TASK-013 reaches, since it built no remote request. The
  rest concerns Android, the renderer decision, synchronization, remote control, and the
  readability and cleanup steps of every task's close. TASK-013's committed record is left as
  accepted; this finding corrects its section 4, and is reported to the owner.
- **Twice, the second window's Explorer stayed without the folder's files.** The first full
  run with the new checks, which ended at 21:09, passed 67 checks and then timed out waiting
  for the LCL folder's files in the second window's Explorer. The test had looked at that
  Explorer, and clicked to open it where it seemed closed, as soon as the window's page
  existed, before its workbench stood, whereas it looks at every first window's Explorer only
  once the workbench stands. A scratch copy of the test that logged the second window's state
  every two seconds for forty seconds, before looking at its files, then passed all 78 checks:
  from its third sample on, some four seconds in, the window showed the LCL folder's files and
  named the LCL workspace in its title row. The test was made to wait for the second window's
  workbench, as it does for every first window, and the two full runs after the change, which
  ended at 21:18 and 21:22, passed all 78 checks. At about 22:01, with that wait in place, the
  first run of the first attempt at the seeded defects stopped at the same step, so the early
  look was not the cause, or not the only one. What the window showed then was not recorded.
  Two more scratch copies of the test then opened the second window 16 times, 8 of them each
  after a fresh first window as in the test, sampling twice a second what both windows showed:
  every time the files showed within about a second and a half of the workbench, and both
  windows stayed visible and kept drawing. Six more full runs of the test passed, and in every
  run after them the files showed. In all, the step ran in 23 full runs of the test or of a
  scratch copy of it up to the close, and stopped in 2. Why the files did not show was not
  found. The test now says, when a wait for a window's files ends, what the window showed: its
  folder, its title row, its loading screen, its Explorer, its questions and its notices.
- **The seeded defects took two attempts.** The first, from 21:57 to 22:12, caught exactly the
  seeded checks in its runs 2 and 3. Its run 1 failed the two checks it was to fail on the way
  and then stopped at the second window's files, as above, before the third, the check that no
  command adds a folder. The second attempt, from 22:49 to 23:01, after the build from scratch
  and the logged run were made again, caught exactly the seeded checks in all three runs, and
  the restored installation passed all 78 checks. The log is the second attempt's.
- **The compaction, and the record checked before the close.** The first draft of this record
  was written just before the session's context was compacted, at 21:31. Afterwards it was
  read again, claim by claim, against the code, the logs and the session's own record. That
  changed in it: the number of earlier end-to-end checks this task changed, which it put at
  eight; the cause of the stopped run, now told as not found; the times of the runs; the order
  of the title row, which is the picture's only in part; the description of the host's agent
  test; the reading of the continuation profile, done for TASK-012; and the list of records
  that name TASK-014, which now also holds the ranges of tasks that include it. The same
  reading found the four changes to the test, the code and the documents that section 9
  lists last.
- **The first logged end-to-end run failed one check.** It built the application from scratch
  at 21:44 and passed 77 of the 78 checks. TASK-009's check that the backend refuses to start
  without the token failed: the backend did refuse, with exit code 1, but the check looks for
  the words of the backend's error, which this task reworded at 21:16. The runs that passed
  all 78 checks during the implementation had used an installation built before that, so the
  logged run was the first to meet the new words. The check now looks for them, and the check
  of a click on the workspace's name had been added meanwhile. The build from scratch and the
  logged run were made again at 21:51, and once more at 22:45, after the test was given its
  report of a wait that ends; the logs are those of the last, which passed all 78 checks.
- **The recheck mended three statements of this record.** The first close was accepted at
  23:13:06 and reported. Its recheck read this record again, claim by claim against the code,
  the logs and the session's record, and found three statements that said more than those bear
  out: that every run since the second stop at the second window passed, here and in the list
  of what the owner should know, where the seeded runs among them failed their seeded checks
  as they were meant to, and what held in every run is that the files showed; and that the
  records were searched for every assignment to TASK-014 before the work began, where the
  search for the ranges of tasks that include it came when this record was checked. They are
  mended above, and the close was run again; nothing of the first close reached Git.
- **The test that every icon is used looked at the shell only.** It failed once the
  switcher, outside the shell's folder, used the new icons. It now looks at all of the
  window's TypeScript, which is what its rule is about.
- **The host's first test of windows told apart was racy.** It assumed that the first window
  gets number 1, but the probe that checks whether a host answers holds a number for a
  moment. The tests now look at every window client's record, and the numbering has a test
  of its own.
- **The panels stay one for all windows.** Two windows open at once share the one panel
  layout, as before this task: the next window gets what any of them stored last. TASK-016
  keeps view state per workspace.

## 12. Open items and limits

- **Nobody has looked at the real screen.** Every window the task saw ran in a private test
  session, in memory.
- **The panel layout** is one for all windows until TASK-016.
- **A window number is not a window's identity.** A window that starts again may get another
  number, and with it another client; it claims its folder again when it attaches, so what it
  shows does not change.
- **A window that cannot reach a host** shows the folder it started on, and names no
  workspace, until a host answers.
- **A host that is killed** while windows are attached cannot close their workspaces: the
  clients they leave keep their foreground in the store until a window that gets the same
  number claims its own folder, which it does as soon as it attaches. A host's start does not
  clear them. What a client shows is presentation only.
- **The second window's files, rarely.** In 2 of 23 full runs of the test or of a scratch copy
  of it, the second window's Explorer did not show the folder's files within a minute; in
  every other run they showed, and in 16 repetitions made to catch it they showed within about
  a second and a half. It is not known whether the cause is the test's or the product's: what
  the window showed was not recorded either time. The test now records it when it happens.
- **No rename, forget or delete** of a workspace, as TASK-013 left it.
- **Also open, unchanged:** the open items of TASK-013's record that this task did not
  answer: a workspace's folder moved or replaced while it is shown, found only at the next
  open; the test session and the login session's lock mark, and the real login, logout and
  lock runs of TASK-009 (PD-STARTUP); the two checks of earlier tasks that each failed once in
  TASK-012; the open items of `docs/dependencies` (OI-01 to OI-10), the open security decisions
  of `docs/security` (OSD-01 to OSD-09) and AMEND-075-01, as their records state.

## 13. Next task

TASK-015 (Decouple foreground workspace from agent workspace authority), within the owner's
assigned range. It was not started.
