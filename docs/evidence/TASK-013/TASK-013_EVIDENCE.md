# TASK-013 evidence: Implement workspace registry and lifecycle

| | |
|---|---|
| Task | TASK-013 — Implement workspace registry and lifecycle |
| Date | 2026-10-06 |
| Performed by | Coding agent (Claude Code), one primary session; no sub-agents, reviewers or background AI jobs. The model was Claude Opus 5.5 throughout |
| Checkout | `/mnt/F/Nexees/`, branch `main`, base `d40b75b2d7d6904893adf8c64c169b1fcbadaba3` |
| Repository | `https://github.com/aivars11232/Nexees` |
| Specification pack | `/mnt/F/Nexees_LCL_Implementation_Pack_v0.5.3/`, archive revision 0.5.3, specification version 0.5.0 |
| Pack content identity | `bd5e8fce26f542c44a891e4340da3b2a84f79ab368b5cb868beb48126c2db3e1` |
| Pack manifest SHA-256 | `8d9d0080ac13a3fb17cc70603b77eb3b11abcd3fb6d2fad515cfac136fcf07e4` |
| Procedure | Continuation profile 0.5.3 (CA-01 to CA-12), as adopted for TASK-002 and continued for this task |
| Predecessors | TASK-001 (`284f97a`), TASK-002 (`d1a673f`), TASK-003 (`2eaa199`, receipt decided in `bccc9b1`), TASK-004 (`0337be0`), TASK-005 (`e83dcd7`), TASK-006 (`fc42a93`), TASK-007 (`9f28c37`, follow-up `1c85864`), TASK-008 (`2bf47a2`), TASK-009 (`bba49c3`), TASK-010 (`53448f2`), TASK-011 (`8bfd07a`) and TASK-012 (`d40b75b`), all accepted; `d00bb83` is the owner-directed correction, and `f0c713a`, `7414e6a`, `34ac37c` and `fd28f27` are checkpoints of TASK-010 and TASK-012, none of them a task commit |
| Commits of this work | None while the task worked. The closing commit is made after acceptance, as CA-06 permits, and is recorded by Git, not in this file |

## Status

The deliverables are complete and every check passed on its final run. Acceptance is recorded
in [receipt/RECEIPT_RESULT.md](receipt/RECEIPT_RESULT.md), which is written after this file
because the receipt contains a snapshot of everything else.

TASK-013 made the device's workspaces a registry that the Nexees host keeps.
- **A workspace** has a stable ID, a kind (CODE or LCL), a name, a root folder on this device
  and whether its content is here. Its record and this device's replica of it live in the
  device's state store, so the workspaces outlive the host.
- **Create, open, close, list.** A new workspace gets an ID of sixteen random bytes that is
  never given again. Opening a workspace makes it the one a client shows, its foreground
  workspace; closing it leaves the client showing none. A list comes in pages that each fit
  one message of the window's channel.
- **A folder is one workspace's.** A root is an existing folder, kept in its canonical form.
  It never is, contains or lies inside another workspace's root on the device, compared by
  the identity of the directories, so no spelling, link or mount makes one workspace's files
  another's. A workspace whose folder was moved or replaced by a link is not opened.
- **The window shows its foreground workspace.** The folder a Desktop window shows is made
  its client's foreground workspace, and a CODE workspace when it is none yet; a folder that
  cannot be one is shown with a warning and no workspace. The command palette creates, opens
  and closes workspaces. Theia shows the folder.
- **Nothing else follows.** Opening, closing or creating a workspace binds no agent and
  changes no other record. A Rust test of the registry and one of the host show an agent
  session bound before the switches unchanged after them.
- **One typed contract.** Four new messages of the window's channel and the host's answer
  made protocol version 3. A window of TASK-012's build keeps its panels and is refused the
  workspace messages.
- **Checked on the installed application.** Eight new end-to-end checks create, open, list
  and close workspaces as a user would, restart the host, open a window on a link to a
  workspace's folder and one on a folder inside another workspace's, and ask the host what it
  keeps. Defects seeded against those checks are caught.

What TASK-013 does not do: tell several windows apart, bind the file tree beyond Theia's own
(TASK-014), bind agents (TASK-015), or keep view state per workspace (TASK-016).

Five things the owner should know before relying on this record, each told in full in
sections 4, 5, 11 and 12:
- every window of the user is still one client, so the client's foreground workspace is the
  one opened last in any window, and a window that starts on no folder closes it for all;
- a window opened on a folder inside another workspace's folder warns and has no workspace,
  but still shows that folder's files; making the tree follow the foreground there too is
  TASK-014's;
- a folder opened in a window becomes a workspace the first time, named after the folder;
  there is no command yet to rename, forget or delete a workspace;
- the session's context was compacted once during the task, and the work after it went on
  from a summary and the session's own record (section 4);
- nobody has seen this on a real screen: every window ran in a private test session.

## 1. Authority

The owner's instructions, verbatim, with the times they arrived (CEST).

On 2026-10-04, during TASK-009, giving the range and the order of each task's close
(TASK-009's to TASK-012's records quote them too):

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

TASK-013 is inside that range and began when TASK-012 was committed, pushed and cleaned up.
It needed no decision of the owner. At 19:16:27, while the task was being implemented, the
owner asked:

```
How far done are u?
```

and was told where the task stood. This record covers TASK-013 only.

## 2. Checkout preflight (binding B26)

| Item | Observed |
|---|---|
| Working directory | `/mnt/F/Nexees/`; the Git worktree root, with no parent repository, superproject or second worktree |
| `origin` fetch and push | `https://github.com/aivars11232/Nexees.git`; no separate push URL |
| Branch and HEAD at task start | `main` at `d40b75b`, equal to `origin/main` after a fetch of that branch only, without tags |
| Hooks | None besides Git's samples; `core.hooksPath` unset |
| At task start | Clean: no staged, modified or untracked file, no stash, 679 tracked files. The owner's IDE keeps ignored output in the checkout: 562 files under `target/` and 17 under TASK-003's Gradle prototype (`.gradle/`), and the editor's link `apps/desktop/node_modules`; five IDE processes |
| Result | Passes ([logs/preflight.txt](logs/preflight.txt)) |

TASK-013 began at 2026-10-06T18:37:25+02:00, after TASK-012 was pushed and its scratch files
were removed. That time is the baseline of every "nothing written since" check. The session
was not paused during the task.

## 3. Pack, native verification, dispatch and predecessors

**Pack.** All 201 manifest entries match their SHA-256, with no unlisted or missing file. The
content identity recomputes to `bd5e8fce…b3e1`. `tasks/task_013.lcl.txt` (`0405721c…5aa7`)
equals the `after_sha256` of `PROCEDURAL_CHANGES.json`.

**Engine.** `lcl 1.0.0`, the engine that evaluated TASK-012's acceptance, by its SHA-256
(`15a3e939…`); the LCL repository is at `ada0b5c`, clean, with the revision the pack binds in
its history and the canonical packages unchanged since it.

**Native verification.** The pack's runner passed all 51 synthetic language cases with that
engine ([logs/native_language_suite.json](logs/native_language_suite.json)). These are
language tests, not evidence for any task.

**Dispatch.** `lcl run … --input input.task_number=13 main.lcl.txt` ran `task.dispatch` and
`task.task_013` only, with `status.succeeded` and no diagnostics. It published the TASK-013
packet (SHA-256 of the packet text `65800346…7c18`), with `procedure_amendment_applies: TRUE`
([logs/dispatch_task_013.record.json](logs/dispatch_task_013.record.json)).

**Predecessors.** All twelve engine records are intact and `status.succeeded`, and all their
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

At task start no predecessor deliverable or evidence file had an uncommitted change.

## 4. Required reading

The task's declaration and the workspace model, which the declaration names as required
reading, were read in full. The continuation profile had been read again in full when the
session resumed earlier the same day, for TASK-012. The contracts, bindings and rules on
workspaces were read again for this task: C1 to C8, C15 and C17 of the global contracts; B2
to B6, B17, B18, B20 and B22 of the bindings; R11 and R13 of the master rules; and the reuse
matrix's row of the IDE and workspace foundation. Every other document of the required
reading and of the mandatory read order is byte-identical to its reading for TASK-012, which
read it in full, and was reused ([logs/required_reading.txt](logs/required_reading.txt)).
For the policy review of section 10, every rule of the nine policy documents was read for
this task.

The session's context was compacted once during the task, at 19:26 CEST, while the evidence
scripts were being adapted. The work after it went on from a summary of what was done before.
Where the summary was unsure, the session's own record was read, and the statements of this
record about what was read, run and observed before the compaction come from it.

The repository's records were searched for every assignment to TASK-013 before the work
began; section 8 answers each. The code the registry builds on was read: the domain's
workspace, client and agent session modules, the state store with its records and
transactions, the protocol's messages and version negotiation, the Desktop host and the
window's backend. Theia's own code was read, as the Desktop build folder holds it, for how its
workspace service opens and closes a workspace in a window, its file URIs, its quick input and
its messages.

## 5. What was decided and built

### Where a workspace lives

The shared domain model had the records since TASK-006 and the store has kept them since
TASK-008: a workspace (`nexees.workspace.workspace`: ID, kind, name), which every
participating device knows; a device's replica of it (`nexees.workspace.replica`: the root on
that device, if its content is there, and its availability); and a client's foreground
selection (`nexees.client.state`). What was missing was the code that makes and changes them
under rules. That is the new crate `nexees-workspaces` in `core/workspaces`, the subsystem
SS-WORKSPACES of the architecture, with one module, `workspace_registry`:

- `create` checks the root and writes the record and the replica in one transaction, after
  checking in that same transaction that the ID is new and that no root of the device
  overlaps. So two creations cannot both pass the checks, and a workspace exists with both
  records or not at all.
- `open` makes a workspace a client's foreground workspace. It refuses a workspace the device
  does not know, one of which only a listing is here (A3), and one whose recorded root is no
  longer the same folder.
- `close` leaves the client showing no workspace. Only the workspace the client shows can be
  closed for it.
- `list` and `find_by_root` read the device's workspaces; a root is found under any of its
  spellings.

No new record and no schema change: the store stays at schema version 2.

### IDs, kinds, names and roots

- **IDs.** The registry takes the ID from its caller and refuses one it already holds; it
  never changes one. The Desktop host gives each new workspace `ws-` and sixteen bytes from
  `/dev/urandom` in hexadecimal. An ID is never derived from a name or a path.
- **Kinds and names** are the domain's: CODE or LCL, and a label of one line of at most 256
  bytes.
- **Roots.** A root must be an absolute path to an existing directory. It is kept in its
  canonical form, all links and `..` resolved, and must then be text of at most 1024 bytes:
  short enough that the largest workspace still fits one message of the window's channel
  with its name and ID, which a host test shows. The domain allows device roots of 4096
  bytes; the registry's bound is its own and the tighter one.
- **One folder, one workspace.** On its device a root never is, contains or lies inside
  another workspace's root. The registry compares the identity of the directories, device
  and inode, along the chain from the root to the file system's root, so a symbolic link, a
  bind mount or another spelling of the same folder cannot get round it. A recorded root
  that is no longer on the disk is compared by its path.
- **Moved roots.** Opening checks the recorded root again: a root that now resolves to
  another folder, because it was moved or replaced by a link, is refused, so a workspace
  never shows another folder's files under its name. A root that is gone is refused as
  missing.

### Opening and closing: the foreground, and nothing else

Opening a workspace writes one record, the client's foreground selection. It writes no agent
session, no grant and no other workspace's record. An agent names its workspace itself, and
nothing a client shows changes that (C2, C3, R11, B3, B4). Two tests prove it with an agent
session bound to one workspace while a client switches between two: the registry's test
compares every agent session, every grant and every workspace before and after the client
opens one workspace, then the other, closes it and opens it again; the host's test puts the
session in the store between two runs of the host, switches the window four times through
the channel, and finds the session unchanged afterwards (T013.VERIFY.05). A negative test
seeds a registry that rebinds every agent session to the workspace it opens, and the host's
test catches it (section 7).

### The messages: protocol version 3

The window asks with four new messages, `list_workspaces` (with a cursor), `create_workspace`,
`open_workspace` (by ID, or by the root the window shows) and `close_workspace`. The host
answers each with one `workspaces` message: a page, the request carried out with the
foreground the client then has, or a refusal with its reason. Every type decodes strictly:
an unknown field, a client, an agent or a grant in a request is refused, and the host then
closes the channel. A window or host that agreed on version 2 neither sends nor accepts
them; TASK-012's window keeps its panels with this host.

### The host

The host keeps the registry for its windows. The foreground it sets is that of its one
Desktop client, `desktop-window`, under which it already keeps the window's panels: a window
names no client. A refusal of the registry is answered with its reason and changes nothing.
When the store fails, or the registry reports what no window's request should cause, such as
a newly drawn ID that already exists, the host closes the channel instead of answering with a
guess. A list is sent in pages: the host adds workspaces in ID order while the page still
fits one message, and the window asks for the next page after the last ID it got.

### The window

The new frontend module `apps/desktop/src/workspaces/workspace_switcher.ts`:
- **Claims the window's folder.** When the window starts on one folder, and again whenever it
  attaches to a host, it asks the host to open the workspace whose root that folder is, and
  creates a CODE workspace named after the folder when there is none. A window without a
  folder, with several, or with a folder that cannot be a workspace, such as one inside
  another workspace's root, closes the workspace its client showed, and in the last case says
  why. The claims run one after the other, so two cannot both create the workspace.
- **Commands.** "Nexees: New Workspace…" asks for the kind, the folder and the name, creates
  the workspace and shows it. "Nexees: Open Workspace…" lists the device's workspaces with
  their kind and folder and shows the chosen one. "Nexees: Close Workspace" closes the
  workspace and the folder. A refusal is shown in words; the workspace a root overlaps is
  named.
- **Theia shows the workspace.** Showing a workspace asks the host to open it first, so a
  workspace the host refuses is never shown, and then opens its root as Theia's workspace in
  the same window, which starts again on that folder. The tree is Theia's file tree of that
  folder, for a CODE workspace as for an LCL one (B5, B6).

The window's backend carries the four messages, checks the host's answers before the
frontend sees them, refuses to send a request that does not fit one message, and asks only
a host that agreed on version 3. The window checks a name and a root as the host does, control
characters of the C1 range included, so it never sends what the host would refuse by closing
the channel.

### The visible tree is the foreground workspace's

Once the host has answered, the folder a window shows is its client's foreground workspace,
and Theia's file tree is that folder's (B2, C4). The end-to-end test checks it each way
(T013.VERIFY.06): the window's folder becomes the foreground; a new LCL workspace is shown
at once with its own tree and without the other's files; opening the CODE workspace from the
list shows its tree again; a window on a link to the LCL folder shows the LCL workspace; and
closing the workspace leaves the window without a folder and the client without a foreground
workspace. One case differs: a window on a folder inside the CODE workspace's root gets no
workspace, its client none, and says why, but Theia still shows the folder it was opened on
(section 12).

### What was reused, and what was not added

- **Reused:** the domain's workspace, replica and client records; the state store and its
  transactions; the protocol's strict decoding, bounds and version negotiation; the window's
  channel; Theia's workspace service, which opens and closes a folder in a window, its file
  tree, its command palette, quick input and notifications. The reuse matrix names Theia as
  the IDE and workspace foundation; Nexees keeps the identity, the kind, the root and the
  foreground, Theia the files and editors.
- **No package** was added; `Cargo.lock` gained only the new crate's own entry.
- **Not added:** commands to rename, forget or delete a workspace, which the objective does
  not name; a second store or table; a file tree of Nexees's own (TASK-014); agent bindings
  (TASK-015, slot `workspace_binding`); per-workspace view state (TASK-016, slot
  `view_state`); a setting. The slots `workspace_binding`, `workspace_revision`,
  `file_tree_model` and `view_state` stay placeholders.

### Security considerations

The task's security requirement is to prove that no authority leaks across workspaces.
- **No authority in the foreground.** Opening and closing write only the client's foreground
  selection; the tests of the section above show agent sessions and grants untouched, and a
  negative test shows the check catches a rebinding.
- **No workspace's folder inside another's.** The overlap rule, by directory identity, holds
  for links, `..`, nested and containing folders, in the registry's tests and, through the
  window, in the end-to-end test. It is why the registry is now one of the points that
  enforce TH-14: a path resolved beneath one workspace's root reaches no other workspace's
  folder. Moved and replaced roots are refused at open.
- **Strict, bounded input at both ends.** The host decodes requests strictly, closes the
  channel on anything else, and bounds names, roots and pages; the window checks the host's
  answers as input. A window names no client, no agent and no grant.
- **The same boundary as before.** The messages cross the window's channel, which admits only
  the same OS user. A program of that user can make a folder it can already read a
  workspace, and open or close one, which changes what the window shows and nothing more;
  TH-01 and TB-01 now say so. No residual risk or open decision was added.
- **No secret.** A workspace holds an ID, a kind, a name and a folder's path on the device.
- **Old and new builds.** A window of version 2 is refused the workspace messages, as the
  host's test shows, and the window asks for them only from a host that agreed on version 3.

## 6. Deliverables

**New files:**

| Path | Purpose |
|---|---|
| `core/workspaces/Cargo.toml` | The manifest of the crate `nexees-workspaces` |
| `core/workspaces/lib.rs` | The crate root: what the crate is, its module and the slots that later tasks fill |
| `core/workspaces/workspace_registry.rs` | Create, open, close and list workspaces, with the rules on IDs and roots; eight tests |
| `apps/desktop/src/workspaces/workspace_switcher.ts` | The window's workspaces: the claim of its folder and the three commands |
| `docs/evidence/TASK-013/` | This record, `logs/` and, written last, `receipt/` |

**Removed files:** the two placeholders now implemented under the same stems:
`core/workspaces/workspace_registry.source` and
`apps/desktop/src/workspaces/workspace_switcher.source`.

**Changed files:**

| Path | Change |
|---|---|
| `Cargo.toml`, `Cargo.lock` | The new crate joins the workspace; the lockfile gains its entry and nothing else |
| `apps/desktop/Cargo.toml` | The host depends on the new crate |
| `apps/desktop/src/application_host.rs` | The host serves the workspace messages for its window client; five tests |
| `core/domain/workspace.rs` | One comment: the note on path aliases names the tasks that write paths, no longer this one |
| `core/protocol/messages.rs` | The workspace entry, request and answer types; four messages of the window and the host's answer; a test |
| `core/protocol/version_negotiation.rs` | Protocol version 3 |
| `apps/desktop/src/main.protocol.ts` | The workspaces in the contract between the window's backend and frontend, with the checks of their shape; the identifier check renamed `isId` |
| `apps/desktop/src/main.ts` | The backend carries the workspace messages to the host and back; protocol version 3 |
| `apps/desktop/src/shell/main_window.ts` | Binds the switcher, and passes the host's state changes to it |
| `apps/desktop/src/shell/panel_memory.ts` | Uses the renamed identifier check |
| `apps/desktop/src/tsconfig.json` | Compiles the `workspaces` folder |
| `tests/e2e/desktop/local_application.test.mjs` | Eight new checks; bounded waits for the page's DevTools; helpers for the command palette's questions and lists |
| `tests/tooling/test_build_desktop.py` | The window's workspace numbers are the shared core's |
| `docs/architecture/interfaces.lcl.txt` | IF-ATTACH carries the workspaces; TASK-013 among its schema tasks |
| `docs/architecture/state.lcl.txt` | ST-WORKSPACE and ST-VIEW note what TASK-013 keeps |
| `docs/architecture/subsystems.lcl.txt` | The new crate's slots; the texts of the slots the workspaces pass through; TASK-013 among the owner tasks of the window, the host, and the shared model and protocol |
| `docs/security/threats.lcl.txt` | TH-01 names the workspace requests; TH-14 is enforced by the registry too |
| `docs/security/trust.lcl.txt` | TB-01 names the workspace requests that cross it |
| `docs/engineering/CONVENTIONS.md` | The new section 17 on workspaces |
| `docs/manuals/NEXEES_USER_MANUAL.md` | Section 3: the Desktop's workspaces |
| `README.md` | What TASK-013 built |
| `FILE_TREE.txt` | Regenerated |

No outside dependency was added and none removed.

## 7. Checks run

| Log | What ran | Result |
|---|---|---|
| [run_checks.txt](logs/run_checks.txt) | Every stage of the check runner, and the four core crates built for both Android targets | Nine stages pass; 96 domain, 33 protocol, 31 state, 12 platform, 17 host and 8 workspaces tests; 59 tooling tests |
| [desktop_build.txt](logs/desktop_build.txt) | The token and logo checks, the TypeScript check, the offline build from scratch, the installation | Eight commands exit 0; the installed application carries the workspace commands in its page, the workspace messages in its backend, and those messages and the registry's refusals in its host |
| [e2e_local_application.txt](logs/e2e_local_application.txt) | The end-to-end test of the installed application | 75 of 75 checks pass: the 67 of TASK-009 to TASK-012 and 8 of the workspaces |
| [seeded_defects.txt](logs/seeded_defects.txt) | Defects seeded against the new end-to-end checks, in three runs | In each run exactly the checks of the seeded defects fail; the restored installation passes again |
| [negative_tests.txt](logs/negative_tests.txt) | 24 defects seeded into scratch copies of the checkout, and 2 controls | All caught; the 12 test-stage cases by the named failing test |
| [regression.txt](logs/regression.txt) | The predecessors' LCL projects and cross-checks, every crate's tests, the tooling tests | 27 commands exit 0 |
| [security_scan.txt](logs/security_scan.txt) | The security stage and the scan of the changed files | No finding; no file made executable |
| [ide_activity.txt](logs/ide_activity.txt) | What else ran on the machine during the task | No Rust manifest changed but the task's own; the lockfile equals an offline resolution; every home-cache write is attributed, none to the task (section 10) |

New tests of the crates:
- workspaces: created workspaces keep their IDs, kinds, names and roots across a restart of
  the store; an ID is given once; a root must be an absolute path to an existing folder of
  bounded length; roots never share files, whatever their spelling; opening sets the
  client's foreground and closing clears it; switching the foreground leaves a bound agent
  and every other record untouched; a listing without content here is listed but not
  opened; a root that was moved or replaced by a link is not opened, and another device's
  replica is never opened here;
- protocol: the window's workspace messages are strict and of protocol version 3; a window
  of version 2 agrees on version 2, below the workspace messages;
- host: a window creates, opens, lists and closes workspaces that outlive the host; a request
  that cannot be carried out is refused and changes nothing, and a request that names a
  client, an agent or a grant closes the channel; a window of version 2 is served but refused
  the workspace messages; a window's foreground switches leave a bound agent session
  unchanged; the largest workspace fits one page and a long list comes in pages.

The eight new end-to-end checks, in the order they run, on a private host kept running:
1. the folder a window shows is a CODE workspace of the host, with a stable ID, and the
   window's client shows it. The test asks the host on its own channel, here and below;
2. a new LCL workspace created from the command palette is shown at once, with its own tree,
   without the CODE workspace's files;
3. opening a workspace from the list shows its tree, and the window's client shows that
   workspace;
4. the workspaces outlive the host: a new host lists the same ones, under the same IDs,
   kinds, names and roots;
5. a folder inside another workspace's root is refused, and the window says why, naming that
   workspace; the workspaces stay as they were;
6. closing the workspace leaves the window's client showing none and the window no folder;
7. a window on a link to a workspace's folder shows that workspace, and no second workspace
   is made of the folder;
8. a window on a folder inside another workspace's root shows no workspace, says why, and
   makes none of the folder.

**Seeded defects.** Each run breaks the installed window's workspace code in two or three
independent ways, and the checks each defect reaches are named before the run:
- run 1: the window claims the folder it starts on as an LCL workspace; it creates a CODE
  workspace whatever kind the user chose; and a window whose folder the host refuses keeps
  its client's workspace and says nothing. Checks 1, 2 and 8 must fail;
- run 2: the window says nothing when the host refuses a new workspace; closing the workspace
  closes the window's folder but not the client's workspace at the host; and a window that
  starts on no folder leaves its client showing the workspace it showed. Checks 5 and 6 must
  fail;
- run 3, in the window's backend: the host is asked for an LCL workspace whatever kind the
  window asked for; and every refusal of the host reaches the window as "no such
  workspace". Checks 1, 5 and 8 must fail.

The build, end-to-end and seeded-defect logs were made before the last edit of the sources,
the comment of `core/domain/workspace.rs` (section 9), which changes no code; the final
verification builds the final revision again, installs it and runs the end-to-end test on
it (`receipt/final_verification.txt`).

Checks 3, 4 and 7 are not seeded, for the reasons the log gives: the window opens a chosen
workspace twice, before it shows the folder and when it starts on it, so a defect in one path
is repaired by the other; and the host program, which this tool does not patch, keeps the
workspaces and resolves links. The host's and the registry's tests hold those, and the
negative tests break them there.

**Negative tests.** `cargo test` stops at the first crate with a failing test, and the
host's tests run first, the new crate's last. So a rule of the registry that the host's
tests also hold is caught there, and each case names the test that does catch it: four
cases are caught by the registry's own tests, six by the host's, and two by the tooling
test.

## 8. Check accounting

### Verification

| Check | How it was met |
|---|---|
| T013.VERIFY.01 build the affected targets | The workspace builds, lints, tests and documents; the four core crates, the new one among them, build for both Android targets; the window's TypeScript compiles with every strict check and builds with `theia build`; the application installs ([run_checks.txt](logs/run_checks.txt), [desktop_build.txt](logs/desktop_build.txt)). |
| T013.VERIFY.02 task-specific tests | Eight tests of the registry, one of the protocol and five of the host; three more comparisons in the tooling test; eight new end-to-end checks; three runs of seeded end-to-end defects; 24 negative cases (section 7). |
| T013.VERIFY.03 regressions of the changed subsystem | Every crate's earlier tests pass with the changes; the 67 end-to-end checks of TASK-009 to TASK-012 pass on the changed window and host; the changed architecture and security records pass their LCL projects and cross-checks ([regression.txt](logs/regression.txt)). |
| T013.VERIFY.04 final diff inspected | Section 6 lists every new, changed and removed file since the base, and section 9 what the review of the final diff changed; the receipt's final verification repeats the inspection. |
| T013.VERIFY.05 foreground changes while a bound background agent stays unchanged | The registry's and the host's tests with an agent session bound to one workspace while the client switches between two (section 5); the negative case of a rebinding registry. |
| T013.VERIFY.06 visible tree matches the foreground workspace | End-to-end checks 1 to 3, 6 and 7: after each step the host's foreground workspace and the files the window's Explorer shows agree (section 5). Check 8 shows the one case where they do not: a folder that cannot be a workspace is shown with a warning while the client shows none (section 12). |

### Completion gate

| Check | How it was met |
|---|---|
| T013.CLOSE.01 dependencies closed | TASK-012 accepted and its lineage verified (section 3). |
| T013.CLOSE.02 objective without unrelated scope | **Objective and scope:** workspaces are created, opened, closed and listed, with stable IDs, kinds, roots and persisted metadata, in the registry, through the host and in the window. Nothing beyond it: no rename or delete, no agent binding, no view state per workspace. |
| T013.CLOSE.03 build passes | As T013.VERIFY.01. |
| T013.CLOSE.04 required tests pass | As T013.VERIFY.02. |
| T013.CLOSE.05 security checks pass | The security stage and the scan are clean; the dependency gates pass; section 5 states the security considerations, among them the proof of no authority across workspaces. |
| T013.CLOSE.06 no unnecessary code or dependency | No outside package. Section 5 lists what was reused and what was not added; each new file serves the objective (section 6). |
| T013.CLOSE.07 no workspace/agent/LCL binding invariant violated | Bound checkout, branch and remote. The pack, the LCL repository, the canonical packages and the LCL SDK were not written. One primary agent. In the product, the foreground never rebinds an agent (T013.VERIFY.05), and the visible tree is the foreground workspace's (T013.VERIFY.06), but for the one case of section 12 that TASK-014 takes on. |
| T013.CLOSE.08 evidence recorded | This folder. |
| T013.CLOSE.09 Git diff understood | As T013.VERIFY.04. |
| T013.CLOSE.10 readability and comments | Agent's own review, section 9. |
| T013.CLOSE.11 scoped cleanup | Section 9; [logs/cleanup_record.json](logs/cleanup_record.json). |
| T013.CLOSE.12 post-cleanup verification on the final revision | `receipt/final_verification.txt`. |
| T013.CLOSE.13 manual impact | Updated: section 9. |

TASK-013 is assigned no remote requirement and no scenario.

### Records that name TASK-013

| Record | What it assigns | What was done |
|---|---|---|
| SS-WORKSPACES, owner tasks and the slot `core/workspaces/workspace_registry` | Create, open, close and list workspaces with stable ID, type, name, device-local root and content availability | The crate `nexees-workspaces` and its module; the slot's text says what the module holds |
| IF-REVISION, schema tasks | Content revisions of a workspace, with TASK-032 and TASK-069 | The workspace a revision is bound to (`core/domain/revision`) is now a workspace of the registry, under an ID that is never reused. Recording revisions is the write paths' own, which this task does not add; the slot `workspace_revision` stays a placeholder |
| ST-CONTENT, schema tasks | A replica's content under that device's authorized root, with TASK-006 and TASK-069 | The authorized root of each replica on this device: the registry records it and keeps one folder to one workspace. Writing content through workspace scope and revisions is not this task's (TASK-023, TASK-032) |
| ST-WORKSPACE, schema tasks | The workspace records, with TASK-006 and TASK-008 | Written by the registry; the record's note says how |
| PD-IDE-PLACEMENT, as TASK-009's record leaves it | "the IDE reads and writes the opened folder directly until TASK-013 to TASK-016 and TASK-022" | The opened folder is now a workspace of the registry. The IDE still reads and writes it directly: workspace scope and revisions are not built yet (TASK-022, TASK-023) |
| TASK-006's record, open items, and the comment of `core/domain/workspace.rs` | Filesystem alias checks for paths (case, Unicode normalization): TASK-013 and TASK-051 | For roots: every spelling of a folder that the file system resolves to the same directory is the same root, compared by directory identity (section 5). Paths inside a workspace are checked by the code that writes them, which this task does not add; the comment now names the workspace scope (TASK-023) and import (TASK-051) |
| TASK-009's record, open items | The workspace registry and its binding to the file tree, TASK-013 to TASK-015 | The registry, and the window's folder as the foreground workspace; the rest is TASK-014's and TASK-015's |
| TASK-012's record | A layout per workspace, TASK-013 to TASK-016 | Not this task's part: TASK-016's |
| `README.md` | The workspaces, TASK-013 to TASK-016 | The README now says what TASK-013 built |

TASK-013 was added as an owner task of the window, the local host and the shared model and
protocol, as a schema task of IF-ATTACH, and as an owner task of TH-14, because it changed
them.

## 9. Review, cleanup and manuals

**Readability (agent's own review).** The workspaces are documented where they are defined
and at each place they pass through: what a workspace is on a device and why its two records
are written in one transaction; where an ID comes from; why roots are compared by the
identity of their directories; that opening and closing set a client's foreground and
nothing else; which protocol version has the messages, why a window names no client and
binds no agent, and what each refusal means; under which client the host keeps the
foreground, how a list is paged and what the host does when its store fails; what becomes of
a request that does not fit one message. The frontend module opens with how the window's
folder and its client's foreground workspace are kept one, what Theia keeps and what Nexees
keeps, and that every window is still one client. Review against the final code made these
changes:
- the window's identifier check, named for views when TASK-012 added it and now also
  checking workspace IDs, became `isId`, with the tooling test that holds its bound to the
  shared core's; the helper that accepts an identifier or none, named for the foreground,
  became `idOrNull`;
- the switcher's header said the tree is always the foreground workspace's; it now says so
  once the host has answered, and names the one exception, a folder that cannot be a
  workspace, which the README now names too;
- the protocol's note on a workspace entry said that a window names a workspace by its ID
  only; it now says by its ID or its root, never by its name;
- the host's function that creates a workspace got its documentation;
- the domain's note on path aliases named this task for checks of paths that it does not
  write; it now names the workspace scope and import.

During the implementation the window's checks of a name and a root were made to refuse
every control character the host refuses, the C1 range included; the window's claims of its
folder were made to run one after the other; and the identity comparisons of the registry
were written without indexing.

**Cleanup.** TASK-013 created no temporary artifact inside the checkout. The two placeholders
it implemented were removed with `rm`. `scripts/test/task_cleanup.py` ran with an empty
manifest, removed and refused nothing, and recorded every untracked file it left in place
([logs/cleanup_record.json](logs/cleanup_record.json)). One end-to-end run, cut off by its
time limit, left its kept host and its profile; both were removed (section 11). The
disposable installation and the task's scratch files are removed after the commit; the
Desktop build folder stays as a cache.

**Manuals.** `docs/manuals/NEXEES_USER_MANUAL.md` section 3 now says how the Desktop's
workspaces work: the folder a window shows is its workspace, made a Code workspace named
after it the first time; the three commands and what each does; that a folder belongs to one
workspace only and Nexees names the workspace it would overlap; that a moved or replaced
folder is not opened; that a workspace keeps its identity across restarts; and that every
window still counts as one view. The end-to-end test checks what the window does of these;
the registry's and the host's tests check the moved folder and the restart. The LCL manual
is unchanged: its section 1 already says that an LCL workspace opens in the Nexees shell and
shows its own files, as the window now does; the Nexees manual's section 3 says how one is
made.

## 10. Policy review and disclosures

All 135 rules of the nine policy documents were reviewed for this task.

| Policy document | Rules | Relation to TASK-013 |
|---|---|---|
| `policies/master_rules.lcl.txt` | 32 | R2 and R6: one coherent change, the registry and its lifecycle; R5: the store, the channel and Theia's workspace service reused; R8 and R9: the registry's refusals, the strict decoding and the closed channel are tested; R11 and R12: the foreground never rebinds an agent, and the visible tree is the foreground's but for the one case of section 12; R13: per-workspace UI state is TASK-016's; R17: no secret; R25 and R26: review, cleanup, and the manual changed with the behaviour; R27: the bound checkout |
| `policies/global_contracts.lcl.txt` | 31 | C1: stable ID, root, type and the scope a root gives; C2 to C4: the foreground decides what is shown and nothing else; C5: the workspaces survive a restart; C15: one registry in the shared core, the same types for every client; C17: each new file maps to the objective; C25 and C26: readability, cleanup and the manual |
| `policies/acceptance_criteria.lcl.txt` | 11 | Desktop: "file tree always follows foreground workspace" and "background agents remain bound to original workspace", as far as the registry and the window reach them now; "workspaces preserve UI state" is due with TASK-016 |
| `policies/code_readability_and_cleanup.lcl.txt` | 6 | Q1: the workspace and device bindings and the state transitions are documented; Q3 and Q5: cleanup with a record, and the review findings in section 9 |
| `policies/no_unnecessary_code.lcl.txt` | 7 | No duplicate store or parser: the records are the domain's and the store's, and the window's second check of names and roots is held to the host's by the tooling test; the deletion rule: the two placeholders removed; section 6 lists the new files and why |
| `policies/reuse_policy.lcl.txt` | 5 | Theia, the selected IDE and workspace shell, shows the folder; workspace and agent decoupling is Nexees's own, as the policy says |
| `policies/security_baseline.lcl.txt` | 9 | The trust hierarchy: a window's request is user input, checked; project-root isolation and symlink escape protection: no root inside another, by identity; the security task rule: the abuse tests of section 7 |
| `policies/usage_and_agents.lcl.txt` | 8 | One agent at a time; heavy jobs one after the other; Git writes only as CA-06 permits; placeholders removed with `rm` |
| `architecture/remote_device_control.lcl.txt` | 26 | RC-20: each device keeps its own visible workspace and remote commands are not retargeted by it; the registry lists only this device's replicas and never opens another device's (B18); RC-23: the channel's bounds and peer check are unchanged; none assigned to TASK-013 |

Disclosures:

- **Network.** Fetching `main` from `origin` at the start, and pushing the closing commit
  after acceptance. Nothing else: npm ran offline, no advisory service was queried because
  no lockfile of npm changed, and the application under test connected to nothing.
- **The owner's files.** The logo file in the owner's Pictures folder was read by the logo
  check and nothing in that folder was changed. No command of the task wrote in the owner's
  toolchain and cache folders: every tool of the task refuses to run without the isolated
  toolchain.
- **Test sessions and the login session.** Up to the close, the task ran the installed
  application only in the private nested sessions that the end-to-end test makes: its runs
  during the
  implementation, two runs of a scratch copy of it that logged each step of the workspace
  part, the logged run, the seeded-defect runs and the final verification's run. One run was
  cut off by its time limit (section 11). No other debugging session was used. The task
  read the owner's login session's lock mark only through `loginctl show-session`, as the
  header of the end-to-end log shows, and sent that session no command.
- **A folder beside the toolchains.** The copy of the evidence scripts in
  `/mnt/F/Nexees-toolchains/task-tools/`, outside the repository, is kept up to date for the
  rest of this run of tasks.
- **Session hook.** A hook asks for a dynamic web-application security scan after each
  change. The application serves no web application, and `HAWK_API_KEY` is unset, so none
  was run.
- **Other programs' writes in the home caches.** While the task worked, one file was written
  in the user's home caches, by none of the task's commands
  ([logs/ide_activity.txt](logs/ide_activity.txt), section 4): `~/.cargo/.global-cache`, at
  18:46:48, by rust-analyzer in the owner's VS Code. That was one second after the task added
  the new crate to the Rust workspace's `Cargo.toml`; from 18:46:49 rust-analyzer checked the
  crate into the checkout's ignored `target/`. The task's own cargo commands of that minute
  ran with the isolated toolchain, and stop first when it is not loaded.
- **Owner-only state.** cargo-deny was executed from the owner's `~/.cargo/bin`. HopToDesk,
  linger, the phone, the LCL SDK, the owner's `lcl-remote` service and the owner's VS Code
  settings were not touched.

## 11. Deviations and findings

- **An end-to-end run hung, and was cut off.** The first run with the workspace checks, at
  19:03, passed 60 checks, the last of them the check that the panels are the host's to keep,
  just before the workspace part, and then printed nothing until its 500-second limit ended
  it. The test's questions to the page's DevTools had no time limit, so a question the page
  never answered, as while it starts again on another folder, would wait for ever: the likely
  cause, not a proven one. The test now gives up on a question after ten seconds and on
  opening the page's DevTools after ten, and no run since has hung. The cut-off run
  had not stopped its kept host or removed its profile: the host was stopped by its process
  ID once its program was seen to be the disposable installation's, and the profile folder
  was removed.
- **The check of the closed workspace looked for the wrong thing.** Two runs of a scratch
  copy of the test that logged each step of the workspace part, at 19:13 and 19:16, failed
  the close check alone: it waited for a button that the empty window does not show there.
  The second run recorded what the window does show, and the check now recognises the empty
  window by its Explorer saying that no folder is opened. The next full run, at 19:19,
  passed all 73 checks of that time.
- **A cross-check of TASK-004 failed once.** After TH-14 named the registry as one of its
  enforcement points, the threat-model cross-check refused it: the subsystem of that point
  was not among the threat's enforcers. SS-WORKSPACES was added to them, and the cross-check
  passes.
- **Two checks were missing.** The review of the final diff found that no end-to-end check
  opened a window on a folder that cannot be a workspace, or on a link to a workspace's
  folder, although the switcher and the manual describe both. Checks 7 and 8 of section 7
  were added.
- **Formatting.** Before the first run of the check runner, three Rust lines over a hundred
  characters were shortened; the runner then passed every stage on its first run.
- **The negative tests' rehearsal.** A rehearsal of the twelve test-stage cases, before the
  close, caught each by the test it names. Its control copy failed only the docs stage,
  because the file tree did not yet list the evidence logs written after it; the close
  writes the file tree first.
- **Several windows are one client.** Every window is the client `desktop-window`. With two
  windows open on different folders, the client's foreground workspace is the one claimed
  or opened last, and a window that starts on no folder closes it for both. Telling windows
  apart is TASK-014's.
- **The compaction.** The session's context was compacted once, at 19:26, while the evidence
  scripts were being adapted (section 4).
- **The recheck mended six statements of this record.** The first close was accepted at
  20:18:50 and reported. Its recheck read this record again, claim by claim against the code,
  the logs and the session's record, and found six statements that said more, or less, than
  those bear out: the completion gate and the policy table said the visible tree is always
  the foreground workspace's, which section 12 excepts; the disclosure of other programs'
  writes did not name the one write or say that the task's change of `Cargo.toml` set it off;
  the list of test sessions did not say it ends at the close; the cause of the hung run was
  told as shown, though it was inferred; and the note on the LCL manual could be read as
  that manual's describing how a workspace is made. They are mended above, and the close was
  run again; nothing of the first close reached Git.

## 12. Open items and limits

- **Nobody has looked at the real screen.** Every window the task saw ran in a private test
  session, in memory.
- **One client for all windows** until TASK-014, with the limits of section 11.
- **No rename, forget or delete.** A workspace made of a folder stays in the registry. Its
  name can be chosen only when it is created with "Nexees: New Workspace…".
- **A folder that cannot be a workspace is still shown.** A window opened on a folder inside
  another workspace's root warns and leaves its client without a workspace, but Theia shows
  the folder's files. Making the visible tree follow the foreground workspace in that case
  too is TASK-014's (B2, C4).
- **A Theia workspace of several folders** is no Nexees workspace: a window that opens one
  leaves its client without a workspace, and Theia shows the folders.
- **The root is checked when the workspace is created and opened.** A folder moved or
  replaced while a workspace is shown is found at the next open; what the window then shows
  is Theia's.
- **The file tree is Theia's.** Binding the visible tree to the foreground workspace beyond
  that, agent bindings and view state per workspace are TASK-014, TASK-015 and TASK-016.
- **Also open, unchanged:** the open items of TASK-012's record that this task did not
  answer: the real-screen look; the test session and the login session's lock mark, and the
  real login, logout and lock runs of TASK-009 (PD-STARTUP); the two checks of earlier tasks
  that each failed once in TASK-012; the open items of `docs/dependencies` (OI-01 to OI-10),
  the open security decisions of `docs/security` (OSD-01 to OSD-09) and AMEND-075-01, as
  their records state.

## 13. Next task

TASK-014 (Bind foreground workspace to its file tree), within the owner's assigned range.
It was not started.
