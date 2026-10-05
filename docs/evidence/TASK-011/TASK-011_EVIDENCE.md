# TASK-011 evidence: Implement approved Desktop layout and single-line top-right sidebar toggles

| | |
|---|---|
| Task | TASK-011 — Implement approved Desktop layout and single-line top-right sidebar toggles |
| Date | 2026-10-05, with a pause of an hour and a half in the morning (section 1) |
| Performed by | Coding agent (Claude Code, model Claude Fable 5.1), one primary session; no sub-agents, reviewers or background AI jobs |
| Checkout | `/mnt/F/Nexees/`, branch `main`, base `53448f2feaf33a0c88b31e139039ccdff1f55f28` |
| Repository | `https://github.com/aivars11232/Nexees` |
| Specification pack | `/mnt/F/Nexees_LCL_Implementation_Pack_v0.5.3/`, archive revision 0.5.3, specification version 0.5.0 |
| Pack content identity | `bd5e8fce26f542c44a891e4340da3b2a84f79ab368b5cb868beb48126c2db3e1` |
| Pack manifest SHA-256 | `8d9d0080ac13a3fb17cc70603b77eb3b11abcd3fb6d2fad515cfac136fcf07e4` |
| Procedure | Continuation profile 0.5.3 (CA-01 to CA-12), as adopted for TASK-002 and continued for this task |
| Predecessors | TASK-001 (`284f97a`), TASK-002 (`d1a673f`), TASK-003 (`2eaa199`, receipt decided in `bccc9b1`), TASK-004 (`0337be0`), TASK-005 (`e83dcd7`), TASK-006 (`fc42a93`), TASK-007 (`9f28c37`, follow-up `1c85864`), TASK-008 (`2bf47a2`), TASK-009 (`bba49c3`) and TASK-010 (`53448f2`), all accepted; `d00bb83` is the owner-directed correction, and `f0c713a` and `7414e6a` are TASK-010's two checkpoints, none of them a task commit |
| Commits of this work | None while the task worked. The closing commit is made after acceptance, as CA-06 permits, and is recorded by Git, not in this file |

## Status

The deliverables are complete and every check passed on its final run. Acceptance is recorded
in [receipt/RECEIPT_RESULT.md](receipt/RECEIPT_RESULT.md), which is written after this file
because the receipt contains a snapshot of everything else.

TASK-011 gave the Desktop window the approved layout.
- **One thin title row** stands in place of the system's title bar: the logo and name, the
  menu, the window's title, the two sidebar toggles and, immediately after them, the window
  controls. Nothing has a row of its own above or below it.
- **The regions are arranged as approved:** the activity bar and the left sidebar over the
  whole height; beside them the editor and the right sidebar over the bottom panel, which
  runs under both; the status bar below. A new profile shows every region, in the shares
  measured on the approved picture.
- **The right sidebar** shows its five areas as a row of text tabs: AI Agent, LCL, Tasks,
  Chat and Logs. No task has filled an area yet; each says that it is not available and
  holds no control.
- **Every region is the foundation's own panel.** Four small modules arrange Theia's shell,
  side panels and title bar through the places Theia offers for that. No package was added.
- **Checked on the installed application.** Sixteen new end-to-end checks compare the places,
  order and sizes of what the window shows with the design tokens, and use the toggles, the
  tabs and the window controls as a user would. Defects seeded against those checks are
  caught.

What TASK-011 does not do: fill the areas (their own tasks), decide what Nexees keeps of a
layout the user leaves behind and where (TASK-012), and build the other things the approved
picture shows in its title row and bars (section 5, "What the picture shows and this task did
not build").

Seven things the owner should know before relying on this record, each told in full in
sections 5, 11 and 12:
- the owner's UI samples, named while the task was closing, differ from the approved Desktop
  picture in three points of the layout; the task kept to the approved picture and asks the
  owner whether any of the three should change;
- nobody has seen the layout on a real screen, and moving or resizing the window with a real
  pointer was not tried;
- the icons of the five areas and of the two toggles were chosen by the task;
- the first sizes of the title row and the window controls were read off the picture roughly
  and corrected once the picture was measured properly;
- a profile that a build before this task had used keeps the system's title bar until the
  title bar style is changed in the settings; no such profile exists outside the test
  sessions;
- one command of the task ran the system's cargo with the owner's cargo home by mistake, and
  cargo updated one bookkeeping file there, `~/.cargo/.global-cache`;
- the session stood still for an hour and a half when the owner's plan ran out of usage.

## 1. Authority

The owner's instructions, verbatim, with the times they arrived (CEST).

On 2026-10-04, during TASK-009, giving the range and the order of each task's close
(TASK-009's and TASK-010's records quote them too):

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

TASK-011 is inside that range and began when TASK-010 was committed, pushed and cleaned up.
It needed no decision of the owner.

**The pause.** The owner's plan has a usage limit, which the task cannot see. At about
10:01:47, after a command had returned, the session stopped; nothing ran until the owner wrote
at 11:32:36:

```
Resume where u stopped because of usage drop
```

The machine was not restarted in between and the session's scratch folder was intact. The
checkout, the pack, the LCL repository and the engine were checked again before the work went
on ([logs/preflight_resume.txt](logs/preflight_resume.txt)).

**The UI samples.** At 12:11:46, while the task was being closed, the owner wrote:

```
also when it comes to android and PC UI, here u have samples - /mnt/F/Nexees_PostCore_LCL_Implementation_Pack_v1.2.0/   - there are 2 olders dedicated for just UI
```

That pack is the second Nexees package, for the tasks after TASK-075; until this message the
session had kept away from it. The task took the message to mean that the two reference
folders of that pack, `Nexees web UI design/` and `Nexees pets/`, are samples of how the
Android and PC interfaces should look, and read them at once; none of that pack's tasks was
started. At 12:17:46 the owner added:

```
Pets will come way later, in post core
```

So the pets are no part of the present work, and their folder was only listed. Section 5 says
what the other samples show and what the task did with them.

This record covers TASK-011 only.

## 2. Checkout preflight (binding B26)

| Item | Observed |
|---|---|
| Working directory | `/mnt/F/Nexees/`; the Git worktree root, with no parent repository, superproject or second worktree |
| `origin` fetch and push | `https://github.com/aivars11232/Nexees.git`; no URL rewrite, no separate push URL |
| Branch and HEAD at task start | `main` at `53448f2`, equal to `origin/main` after a fetch of that branch only, without tags |
| Hooks | None besides Git's samples; `core.hooksPath` unset |
| At task start | Clean: no staged, modified or untracked file, no stash, 627 tracked files. The owner's IDE keeps ignored output in the checkout: 562 files under `target/` and 17 under TASK-003's Gradle prototype (`.gradle/`), and the editor's link `apps/desktop/node_modules`; seven IDE processes |
| On resumption | `main` at `53448f2`, equal to `origin/main`; nothing staged; the task's own work in progress uncommitted, and no change in any predecessor's evidence or in the charter; the pack and the lineage intact |
| Result | Passes ([logs/preflight.txt](logs/preflight.txt), [logs/preflight_resume.txt](logs/preflight_resume.txt)) |

TASK-011 began at 2026-10-05T08:33:23+02:00, after TASK-010 was pushed and its scratch files
were removed. That time is the baseline of every "nothing written since" check. The session
ran no command between 10:01:47 and 11:32:36.

## 3. Pack, native verification, dispatch and predecessors

**Pack.** All 201 manifest entries match their SHA-256, with no unlisted or missing file. The
content identity recomputes to `bd5e8fce…b3e1`. `tasks/task_011.lcl.txt` (`35b1b79b…4d0e`)
equals the `after_sha256` of `PROCEDURAL_CHANGES.json`.

**Engine.** `lcl 1.0.0`, the engine that evaluated TASK-010's acceptance; the LCL repository
is at `ada0b5c`, clean, with the revision the pack binds in its history and the canonical
packages unchanged since it. Both were the same when the session resumed.

**Native verification.** The pack's runner passed all 51 synthetic language cases with that
engine ([logs/native_language_suite.json](logs/native_language_suite.json)). These are
language tests, not evidence for any task.

**Dispatch.** `lcl run … --input input.task_number=11 main.lcl.txt` ran `task.dispatch` and
`task.task_011` only, with `status.succeeded` and no diagnostics. It published the TASK-011
packet (SHA-256 of the packet text `08cf240d…ea29`), with `procedure_amendment_applies: TRUE`
([logs/dispatch_task_011.record.json](logs/dispatch_task_011.record.json)).

**Predecessors.** All ten engine records are intact and `status.succeeded`, and all their
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

At task start no predecessor deliverable or evidence file had an uncommitted change.

## 4. Required reading

The session's context had been compacted between TASK-010 and this task. So the documents the
task names as required reading were read again from the pack, in full: the task itself as the
packet delivers it, the Desktop UI contract, the continuation profile, the two code policies,
the manual contract, the master rules, the global contracts, the bindings, the stop
conditions, the reuse matrix and the usage guide, and the Desktop part of the acceptance
criteria. The rest of the mandatory read order is byte-identical to its last recorded read
and was reused ([logs/required_reading.txt](logs/required_reading.txt)).

The repository's records were searched for every assignment to TASK-011 before the work
began; section 8 answers each. Theia's own code was read for how it builds its title bar, its
shell's layout, its side panels and their tab bars, and for how it restores a layout.

## 5. What was decided and built

### The approved layout on the foundation's panels

The Desktop UI contract names seven regions: one thin title row, the activity bar, the left
sidebar with the file tree, the editor, the right sidebar, the bottom panel and a thin status
bar. Theia has every one of them. The task's reuse-first requirement is to reuse "the selected
IDE/workspace foundation and inherited editor/layout primitives wherever possible", so the
task built no region: it arranges Theia's. Theia offers a place for each thing that had to
change:

| What the approved layout needs | Theia's place for it | Module |
|---|---|---|
| The title row in the window, without the system's frame | Its title bar for a frameless window, chosen by the application's configuration (`window.titleBarStyle`); a contribution that subclasses can extend | `title_bar` |
| The two toggles | Its commands that show and hide a side panel | `panel_controls` |
| The bottom panel under the editor and the right sidebar | The shell's method that assembles the layout, which its own comment gives as the place to change the arrangement | `panel_layout` |
| The share each panel takes | The shell's options | `panel_layout` |
| What a new profile shows | The default layout a contribution gives when none is stored | `panel_layout` |
| The right sidebar's row of text tabs | The handler of a side panel, which Theia creates through a factory | `right_sidebar` |

All four modules are in `apps/desktop/src/shell/`, and `main_window` binds them.

### The title row

- **Theia's own title bar.** The application's configuration asks for Theia's `custom` title
  bar style. Theia then makes the window without the system's frame and draws its title bar
  in the page, with the menu, the window's title and the three window controls, and keeps
  them working. A new profile gets it; Theia keeps the choice in its own store afterwards.
- **What Nexees adds:** the logo with the application's name where Theia keeps a place for a
  logo, and the two toggles, added right after the title, so that they stand immediately
  before the window controls, which Theia adds last.
- **One line.** Theia places the title and the window controls by fixed positions that
  assume its own row. Here they are items of one line like the others: the title takes the
  room between the menu and the toggles, and the toggles end where the controls begin. In a
  window too narrow for the whole row the name and the menu give way, so that the toggles
  and the window controls stay in view.
- **Thin.** The row is 25 pixels high, with its border, where Theia's is 32. The height is a
  design token (below), as are the widths of the toggles and the window controls.
- **The logo** is the 128-pixel icon derived from the bound source (B1), the same file of the
  installation the About dialog shows, drawn at 16 pixels. No new size was derived, and the
  file's name stands in one place for both.
- **Moving the window.** The free part of the row, the logo with the name and the title are
  the parts that move the window when dragged; the menu, the toggles and the controls take
  the mouse.

### The sidebar toggles

- A toggle decides nothing itself. A click runs the command Theia already has for that side
  (`View: Toggle Left Panel`, `View: Toggle Right Panel`), the one the command palette runs.
- A toggle shows what its sidebar did, whoever did it: its icon, its pressed state and its
  tooltip follow the sidebar's own state. Hidden from the activity bar or by a command, a
  sidebar is shown as hidden by its toggle.
- While a sidebar holds no view there is nothing to show, and its toggle is disabled.
- A click with the mouse leaves the keyboard where it is, in the editor for instance. The
  toggles are buttons, so the keyboard reaches them too: from the menu the Tab key stops at
  the left one and then at the right one, and Enter and the space bar work them (both
  observed, section 7).
- Hiding the left sidebar leaves the activity bar, as in Theia. Hiding the right sidebar
  hides it whole, because its tabs are part of it.

### The arrangement and the shares

Theia puts the bottom panel under the editor alone, between the two side panels. The approved
picture shows the left sidebar over the whole height and the bottom panel under the editor
and the right sidebar. The shell's layout is assembled accordingly, from the same panels.

The share each panel takes when it is first shown, and the sizes of the title row, were
measured on the approved picture and are design tokens, in `assets/theme/design_tokens.json`
with the tokens of TASK-010. [measure_approved_layout.py](measure_approved_layout.py) repeats
the measurement from the picture, so that anyone can check the numbers
([logs/approved_layout_measurements.txt](logs/approved_layout_measurements.txt)):

| Token | Value | Measured on the picture |
|---|---|---|
| `desktop.title_bar_height` | 25 px | 25.25 |
| `desktop.sidebar_toggle_width` | 24 px | 23.44 |
| `desktop.window_control_width` | 30 px | 30.30 |
| `desktop.activity_bar_width` (TASK-010) | 48 px | 48.33 |
| `desktop_percent.left_sidebar_width` | 19 % of the window's width, with the activity bar | 18.88 |
| `desktop_percent.right_sidebar_width` | 41 % of the width beside the left sidebar | 40.93 |
| `desktop_percent.bottom_panel_height` | 27 % of the height between the title row and the status bar | 27.03 |

`desktop.title_logo_size` (16 px) is not measured: the picture shows another mark than the
Nexees logo, and 16 pixels is the size of the row's other icons.

A share is a new kind of token: a whole percent in a group whose name ends in `_percent`. The
token script refuses a share outside 1 to 99, and its test proves it.

A share is taken when a panel is first shown. After that a panel keeps its width in pixels
when the window changes, as in Theia, and the user resizes it by dragging its edge.

### The right sidebar and its areas

- **Five areas, in the contract's order:** AI Agent, LCL, Tasks, Chat, Logs. Each is an
  ordinary view of Theia's right side panel. So showing, hiding and sizing the sidebar,
  selecting an area, moving a view to another region and restoring all of it with a stored
  layout are Theia's own.
- **A row of text tabs** across the sidebar's top, as the approved picture shows it. Theia
  stands a bar of icons at the window's edge instead; the right side panel's handler is
  arranged differently for that, and its tab bar is Theia's side tab bar laid out as a row.
  Beside the tabs the row holds the selected view's own actions, where a view has any.
- **Not filled yet.** An area's view is one sentence: "The AI Agent area is not available in
  this version of Nexees." It holds no button, field or link, so nothing in it can be taken
  for a working control. The task that owns an area gives its ID its own view.
- **Always there.** Whatever layout the window starts with, a stored one of an earlier
  version too, the window gets every area it does not hold (observed with a stand-in for
  such a profile, section 7).
- **A narrow sidebar.** The tabs keep their width. A sidebar too narrow for them scrolls its
  row: by the mouse wheel, and to the tab that becomes selected.
- **Stored layouts are input.** Theia names a stored view by a factory and its options. An
  area this version does not know is refused, and Theia restores the layout without it.

### What a new profile shows

The approved layout with every region open: the Explorer, the editor area, the right sidebar
with AI Agent selected, and the bottom panel with the terminal. Theia's own default leaves
the three panels closed.

Theia opens its Outline view in the right sidebar by default. The approved layout keeps that
sidebar for the five areas, so in a new profile the Outline view starts closed. The command
palette still opens it, as a sixth tab.

### What the picture shows and this task did not build

The approved picture is a picture of the finished product. TASK-011 builds its layout: the
regions, their arrangement and shares, and the toggles. These things the picture also shows
belong to the features of later tasks, and the window does not have them:

- in the title row: a search field, the name of the workspace, a status dot with a number, a
  cloud and "Update", and a Run menu;
- in the activity bar: more icons than the Explorer's and the settings';
- in the bottom panel: Output, Debug Console and Ports beside the terminal and Problems;
- in the status bar: the branch and the items of a language server;
- in the right sidebar: the content of the areas.

Three small differences from the picture were left on purpose:
- the row of the right sidebar's tabs is as high as every other row of tabs and headers
  (35 pixels), where the picture draws it a little higher, so that the rows line up across
  the window;
- the bottom panel's tabs keep TASK-010's height and Theia's order, Problems before the
  terminal;
- the title row shows the window's title in its middle, where the picture has the search
  field. The contract calls the row the "top/title/menu row".

### The owner's UI samples

The folder `Nexees web UI design/` of the post-core pack holds twelve pictures of Nexees Web,
the browser version that the tasks after TASK-075 build. The task read the pack's own notes on
the two folders and looked at four of the pictures: the web code editor, the dashboard, and
the two that show every page at once. The other folder holds 25 concept pictures of the
optional pets, which the owner places after the core (section 1); it was only listed.

The pack's notes give the pictures their weight: they are visual references for layout,
density and the treatment of panels, their words are placeholders, and the Web version should
feel like the same product as Desktop. They draw other marks than the Nexees logo; the logo
stays the bound file (B1).

What the web editor sample shows agrees with what this task built in the regions it has, in
the dark look, and in a right panel for the assistant with a row of text tabs. It differs from
the approved Desktop picture, which is this task's reference, in three points of the layout:

| | Approved Desktop picture, and built | Web editor sample |
|---|---|---|
| The top of the window | One thin row with the menu, the toggles and the window controls | A browser's bars and a taller header with the project, the branch, Share, Deploy and Run |
| The activity bar | Icons, 48 pixels wide | Icons with their names under them, about twice as wide |
| The bottom panel | Under the editor and the right sidebar | Under the editor alone; the right panel takes the whole height |

The right panel's tabs differ too: the sample has Chat, Plan, Context and Tools under a title,
where the Desktop UI contract names AI Agent, LCL, Tasks, Chat and Logs.

The task changed nothing because of the samples. The Desktop UI contract asks for the thin
row and for those five areas in so many words, and it names the approved Desktop picture as
the reference for the proportions. The samples are for the browser, where a page cannot draw
window controls or a menu row of its own. The three differences are put to the owner in
section 12. For the tasks that fill the areas and build the settings, the devices and the
Android screens, the samples are the look to follow.

### What was reused, and what was not added

- **Reused:** Theia's title bar, menu, window controls, application shell, side panels, tab
  bar renderer, toolbar of a side panel, dock panels, commands, layout storage and widget
  manager; the Codicon icons of the two toggles; the derived logo icon of TASK-010.
- **No package** was added, and the lockfile is unchanged.
- **Not added:** commands or key bindings of its own (Theia's exist); a view contribution per
  area with its menu entry (the areas are empty; their tasks add what they need); a
  placeholder search field or any other control that does nothing; a second way to hide a
  sidebar. Theia's menu of the side tabs that do not fit is not used on the right, where the
  row scrolls.
- **Two placeholders stay placeholders:** `activity_bar.source` and `status_bar.source`. The
  activity bar and the status bar are Theia's regions as they are, sized by TASK-010's
  tokens; no Nexees module is needed for them yet.

### Security considerations

- **No control implies an authority.** The toggles change this window's view and nothing
  else: no setting of the host, no grant, no state another device sees. The areas hold no
  control at all.
- **Text, not HTML.** The area's sentence, the name in the title row and the toggles' labels
  are set as text. The logo's address is a constant of the code.
- **Serialized input is validated.** The only input the new code takes is the name of an
  area in a stored layout; an unknown one is refused.
- **Nothing new reaches outside.** No network use, no new process, no file written by the
  new code. The window still connects to nothing beyond this machine, and the renderer's
  sandbox and the fuses are as TASK-009 left them, which the end-to-end test checks again.
- **The frame.** A window without the system's frame draws its own controls; they call the
  same functions of Theia's preload interface that Theia's own title bar calls.

## 6. Deliverables

**New files:**

| Path | Purpose |
|---|---|
| `apps/desktop/src/shell/title_bar.ts` | The title row: the logo and name, the place of the toggles, the row's arrangement |
| `apps/desktop/src/shell/panel_controls.ts` | The two sidebar toggles |
| `apps/desktop/src/shell/panel_layout.ts` | The arrangement of the regions, their shares, and a new profile's first layout |
| `apps/desktop/src/shell/right_sidebar.ts` | The right sidebar's row of text tabs and its five areas |
| `docs/evidence/TASK-011/` | This record, the measuring script, `logs/`, `screens/` and, written last, `receipt/` |

**Removed files:** the three placeholders now implemented:
`apps/desktop/src/shell/title_bar.source`, `apps/desktop/src/shell/panel_controls.source` and
`apps/desktop/src/shell/panel_layout.source`.

**Changed files:**

| Path | Change |
|---|---|
| `apps/desktop/package.json` | Theia's `custom` title bar style as the application's default |
| `apps/desktop/src/shell/main_window.ts` | Binds the four modules: the shell, its options, the side panel handler, the areas' factory, the title bar and its title |
| `apps/desktop/src/shell/theme.ts` | The title row's height among the sizes Theia takes from the tokens |
| `apps/desktop/src/shell/about_dialog.ts` | The logo file's name exported, so that the title row uses the same one |
| `apps/desktop/src/shell/design_tokens.ts` | The Desktop copy of the tokens and icons, generated again |
| `assets/theme/design_tokens.json` | The title row's sizes, and the shares of the three panels |
| `assets/theme/icon_mapping.json` | The icons of the two toggles, shown and hidden, and of the five areas |
| `scripts/build/design_tokens.py` | Shares: whole percents in a group named `…_percent`, from 1 to 99 |
| `tests/tooling/test_design_tokens.py` | Cases for shares |
| `tests/tooling/test_build_desktop.py` | The window names the logo file once, for the About dialog and the title row |
| `tests/e2e/desktop/local_application.test.mjs` | Sixteen new checks of the layout; a larger test screen; a window counts as ready only once its loading screen is gone |
| `docs/architecture/subsystems.lcl.txt` | The slot of the shell folder described as built |
| `docs/engineering/CONVENTIONS.md` | Section 14: shares among the tokens; the new section 15 on the Desktop layout |
| `docs/manuals/NEXEES_USER_MANUAL.md` | Section 2: the window's layout |
| `README.md` | What TASK-011 built |
| `FILE_TREE.txt` | Regenerated |

No dependency was added and none removed.

## 7. Checks run

| Log | What ran | Result |
|---|---|---|
| [run_checks.txt](logs/run_checks.txt) | Every stage of the check runner, and the three core crates built for both Android targets | Nine stages pass; 93 domain, 30 protocol, 29 state, 12 platform and 9 host tests; 58 tooling tests |
| [desktop_build.txt](logs/desktop_build.txt) | The token and logo checks, the TypeScript check, the offline build from scratch, the installation | Nine commands exit 0; the installed application asks for the custom title bar and holds the four modules |
| [e2e_local_application.txt](logs/e2e_local_application.txt) | The end-to-end test of the installed application | 59 of 59 checks pass: the 43 of TASK-009 and TASK-010 and 16 of the layout |
| [seeded_defects.txt](logs/seeded_defects.txt) | Defects seeded against the new end-to-end checks, in two runs | In each run exactly the checks of the seeded defects fail; the restored installation passes again |
| [negative_tests.txt](logs/negative_tests.txt) | 21 defects seeded into scratch copies of the checkout, and 2 controls | All caught; the 7 test-stage cases by the named failing test |
| [approved_layout_measurements.txt](logs/approved_layout_measurements.txt) | The approved picture measured again by the script of this folder | The seven measured tokens are within their tolerance |
| [live_observations.txt](logs/live_observations.txt) | Observations in three debugging sessions of what the end-to-end test does not repeat | All as expected (below) |
| [regression.txt](logs/regression.txt) | The predecessors' LCL projects and cross-checks, every crate's tests, the tooling tests | 25 commands exit 0 |
| [security_scan.txt](logs/security_scan.txt) | The security stage and the scan of the changed files | No finding; no file made executable |
| [ide_activity.txt](logs/ide_activity.txt) | What else ran on the machine during the task | No Rust manifest changed; the home-cache writes are the owner's IDE's, except one file that a command of the task wrote by mistake (section 11) |

The sixteen new end-to-end checks:
1. the window has no system frame but one thin title row, of the token's height, with the logo
   and name, the menu, the title, the toggles and the window controls in that order, one
   after the other without a gap, and the workbench begins right under it;
2. the row's free part, logo and title move the window, and the toggles and window controls
   take the mouse;
3. the row shows the logo as a file of the installation, the derived icon, beside the
   application's name;
4. both toggles stand immediately before the window controls, all in the tokens' widths;
5. the regions are arranged as approved, each edge where its neighbour's is;
6. a new profile shows the three panels in the shares of the tokens;
7. the bottom panel of a new profile shows the terminal;
8. the right sidebar shows its five areas as one row of text tabs, the first selected, with
   no bar of icons at the window's edge;
9. each area says that it is not available and holds no control;
10. the right toggle hides the right sidebar whole and shows it again in its width and with
    its area, and says each time what a click will do;
11. the left toggle hides the left sidebar, leaving the activity bar, and shows it again;
12. a toggle follows its sidebar when the activity bar hides and shows it;
13. the window controls work: maximised, the window fills the screen; restored, it has its
    size again;
14. in a window too narrow for the whole row the toggles and the window controls stay in
    view, in one thin row;
15. the row's close control closes the window;
16. a reopened window comes back as it was closed: the title row, the right sidebar hidden
    and its toggle saying so; shown again, it has its areas and its width.

**Seeded defects.** Run 1 breaks the installed application in ten ways: Theia's own
arrangement instead of the approved one; a gap between the toggles and the window controls;
a logo that no longer moves the window; another picture under the logo's name; one icon for
both states of the right toggle; an area renamed; the areas saying something else; a first
layout that shows Problems instead of the terminal; a maximise control and a close control
that do nothing. Run 2 leaves the application whole and gives the test other tokens and
another icon. Each run fails exactly the checks of its defects.

**Observed in debugging sessions**, with the installed application in a private session like
the test's, by scripted clicks, drags and keys. These are observations of one session each,
not tests the repository repeats:
- with the title row (15 observations): Ctrl+J hides and shows the bottom panel; Theia's
  Outline view takes a sixth tab and its cross closes it; the title row, the toggles and the
  tabs take the colours of Theia's light theme; the right sidebar's edge can be dragged, its
  row of tabs scrolls by the mouse wheel when it is too narrow, and a tab that becomes
  selected is scrolled into view; from the menu the Tab key reaches the left toggle and then
  the right one, and Enter and the space bar work them; a tab dragged into the bottom panel
  becomes a tab there, and dragged back it takes its place among the areas; with the
  Explorer dragged out of it the left sidebar holds no view, and its toggle is disabled; and
  a tooltip the foundation leaves behind, in the right sidebar's row and in its own bottom
  panel alike (section 11);
- with the system's frame (3 observations), started as after a restart with Theia's title
  bar style set to `native`: the page has no title row and no toggles, the layout is
  otherwise the same, and the two commands hide and show the sidebars;
- on a profile that a stand-in for an earlier build left behind (4 observations). No build
  before this task was run. A second installation of this build was changed in three places
  of its installed copy, so that its first layout does nothing of this task's, it adds no
  area and it asks for the system's frame; it ran once on a new profile and was closed, and
  then this build started on that profile. This build restores the stored layout and adds
  the five areas, before the Outline view that was stored in the right sidebar. The window
  keeps the system's frame (section 11), and the command shows the right sidebar.

One more observation was made by a scratch script in a debugging session of its own, at
12:41, and is in no log: after a mouse click on each toggle the keyboard was still in the
editor, and text typed then went into the editor's file.

After the logs of the end-to-end test and of the seeded defects were written, the list in the
header comment of the test was completed by one item, the narrow window and the close
control. The session's record shows no other change of the test after those runs, and the
final verification of the receipt runs the test again, on the final revision.

Three pictures of the window, taken in the first of those sessions from the installation the
logs describe: a new profile with a file opened
([screens/01_approved_layout.png](screens/01_approved_layout.png)), both sidebars hidden
([screens/02_sidebars_hidden.png](screens/02_sidebars_hidden.png)), and Theia's light theme
([screens/03_light_theme.png](screens/03_light_theme.png)). They are illustrations; the
checks are the end-to-end test's. The terminal's prompt in them shows the user and host name
of the development machine.

## 8. Check accounting

### Verification

| Check | How it was met |
|---|---|
| T011.VERIFY.01 build the affected targets | The window's TypeScript compiles with every strict check and builds with `theia build`; the workspace builds, lints, tests and documents unchanged; the application installs ([run_checks.txt](logs/run_checks.txt), [desktop_build.txt](logs/desktop_build.txt)). |
| T011.VERIFY.02 task-specific tests | 16 new end-to-end checks; two runs of seeded end-to-end defects; 21 negative cases; the tooling tests of the token script and of the one name of the logo file, extended (section 7). The window's TypeScript has no unit tests: the repository tests the window end to end, on the installed application. |
| T011.VERIFY.03 regressions of the changed subsystem | The 43 end-to-end checks of TASK-009 and TASK-010 pass on the changed window; the changed architecture record passes its LCL project and cross-check; every crate's tests pass unchanged ([regression.txt](logs/regression.txt)). |
| T011.VERIFY.04 final diff inspected | Section 6 lists every new, changed and removed file since the base; the receipt's final verification repeats the inspection. |

### Completion gate

| Check | How it was met |
|---|---|
| T011.CLOSE.01 dependencies closed | TASK-010 accepted and its lineage verified (section 3). |
| T011.CLOSE.02 objective without unrelated scope | **Objective:** the activity bar, the left Explorer, the editor, the right sidebar, the bottom panel and the status bar stand as approved, and the sidebar controls are in the one compact title row. **Scope:** both toggles are in that row, immediately before the window controls; the sidebars and panels are compact and take the measured shares. Nothing of persistence, workspaces or the areas' content was built. |
| T011.CLOSE.03 build passes | As T011.VERIFY.01. |
| T011.CLOSE.04 required tests pass | As T011.VERIFY.02. |
| T011.CLOSE.05 security checks pass | The security stage and the scan are clean; the dependency gates pass with an unchanged lockfile; section 5 states the security considerations. No control implies an authority, and the hardening checks of the end-to-end test pass on the changed window. |
| T011.CLOSE.06 no unnecessary code or dependency | No package added. Every token and icon is used by the window, which a test enforces. Section 5 lists what was not added. |
| T011.CLOSE.07 no workspace/agent/LCL binding invariant violated | Bound checkout, branch and remote. The pack, the LCL repository, the canonical packages and the LCL SDK were not written. The owner's logo file was only read. One primary agent. The layout changes no workspace or agent binding: there is none yet. |
| T011.CLOSE.08 evidence recorded | This folder. |
| T011.CLOSE.09 Git diff understood | As T011.VERIFY.04. |
| T011.CLOSE.10 readability and comments | Agent's own review, section 9. |
| T011.CLOSE.11 scoped cleanup | Section 9; [logs/cleanup_record.json](logs/cleanup_record.json). |
| T011.CLOSE.12 post-cleanup verification on the final revision | `receipt/final_verification.txt`. |
| T011.CLOSE.13 manual impact | Updated: section 9. |

TASK-011 is assigned no remote requirement and no scenario.

### Records that name TASK-011

| Record | What it assigns | What was done |
|---|---|---|
| SS-DESKTOP-CLIENT, owner tasks | TASK-011 among the tasks of the Desktop client | This task |
| SS-DESKTOP-CLIENT, `apps/desktop/src/shell/` | The title bar with the single-row top-right sidebar toggles, the activity bar, the panel layout and controls and the status bar, TASK-009 to TASK-012 | The title row, the toggles, the arrangement and the right sidebar; the slot's text now says so. The activity bar and the status bar are the foundation's (section 5) |
| TASK-010's record, section 12 | The title row and its logo, with the token for the row's height | Built; the token is `desktop.title_bar_height` |
| `README.md` and `main_window.ts`, as TASK-010 left them | "The approved layout and panels come with TASK-011 and TASK-012" | Both now say what was built and what TASK-012 decides |

The charter (CH-02) and the pack's Desktop acceptance name the single title row with the
toggles immediately before the window controls; no security or dependency record names
TASK-011.

## 9. Review, cleanup and manuals

**Readability (agent's own review).** Each of the four modules opens with what it is for,
which part of the Desktop UI contract it serves, and where Theia's part ends and its own
begins. Comments give the reason where the code alone would not: why the toggles run Theia's
commands and take their state from the sidebars; why the right sidebar's tabs are connected
as Theia connects a side bar's, with the two exceptions named; why the first layout waits for
the left sidebar before it shows the right one; why a stored area this version does not know
is refused. Review against the final code made these changes:
- a method of the toggles had the name of one of Theia's widget methods and was renamed;
- token names were written in full, as the test that every token is used looks for them;
- a focus style Theia already has for every control was removed, and a rule to hide a focus
  outline that had no effect;
- the list of areas is no longer exported, since nothing imports it;
- the logo file's name is exported by the About dialog and used by the title row, where it
  had been written twice, and a test now holds it to one place;
- one comment was rewritten in plain characters.

Review of the end-to-end test against its own seeded defects made three more:
- the wait for the first layout no longer requires the terminal, which has its own check;
- a missing tab and a maximise control that does nothing now fail their checks, where they
  made the run stop at a timeout;
- the toggle check compares the area shown again with the one selected before, not with a
  fixed name.

**Cleanup.** TASK-011 created no temporary artifact inside the checkout. Temporary lines that
measured the first layout were put into one source file and into the end-to-end test during
the work and taken out again before any log of this folder was written; the corroboration
searches the files a commit would hold and finds no trace of them.
`scripts/test/task_cleanup.py` ran with an empty manifest, removed and refused nothing, and
recorded every untracked file it left in place
([logs/cleanup_record.json](logs/cleanup_record.json)). The disposable installation and the
task's scratch files are removed after the commit; the Desktop build folder stays as a cache.
The second installation, made beside the first for the observation of an earlier build's
profile (section 7), was removed each time by the script that made it; none is left.

**Manuals.** `docs/manuals/NEXEES_USER_MANUAL.md` section 2 now describes the window's layout:
the title row and what it holds, the regions, what the toggles and Ctrl+J do, resizing by the
line between two regions, and the five areas of the right sidebar, which are not available
yet. The places, the toggles and the areas are what the end-to-end test checks; Ctrl+J and
resizing by an edge were observed in a debugging session (section 7). That dragging the row
moves the window rests on the row's drag regions, which the test checks; no test here can
move a window with a real pointer (section 12). The LCL manual is unchanged.

## 10. Policy review and disclosures

All 135 rules of the nine policy documents were reviewed for this task.

| Policy document | Rules | Relation to TASK-011 |
|---|---|---|
| `policies/master_rules.lcl.txt` | 32 | R5 to R7: the foundation's regions reused, nothing speculative added; R18: the approved compact direction; R25 and R26: review, cleanup, and the manual changed with the behaviour |
| `policies/global_contracts.lcl.txt` | 31 | C16 and C17: no package, and every new module answers a part of the Desktop UI contract; C20: still an installed local application with its own window; C4 (the file tree follows the foreground workspace) is TASK-014's |
| `policies/acceptance_criteria.lcl.txt` | 11 | "Top-right sidebar toggles are in the single title/header line": built here; the other product acceptance is not due |
| `policies/code_readability_and_cleanup.lcl.txt` | 6 | Applied through the conventions; the review findings are in section 9 |
| `policies/no_unnecessary_code.lcl.txt` | 7 | Theia's panels arranged, not replaced; no package; section 5 lists what was not added |
| `policies/reuse_policy.lcl.txt` | 5 | The foundation's shell, side panels, title bar, commands and layout storage |
| `policies/security_baseline.lcl.txt` | 9 | Stored layout input validated; text, not HTML; no authority in the new controls |
| `policies/usage_and_agents.lcl.txt` | 8 | One agent at a time; heavy jobs one after the other; Git writes only as CA-06 permits; placeholders removed with `rm` |
| `architecture/remote_device_control.lcl.txt` | 26 | None assigned to TASK-011 |

Disclosures:

- **Network.** Fetching `main` from `origin`, at the start and on resumption, and pushing
  the closing commit after acceptance. Nothing else: npm ran offline, no advisory service was
  queried because the lockfile is unchanged, and the application under test connected to
  nothing.
- **The owner's files.** The logo file in the owner's Pictures folder was read by the logo
  check and nothing in that folder was changed. The task read the owner's user journal for
  lines of the lock screen (below). Of the post-core pack it read the orientation files, the
  notes of the two reference folders and four pictures, after the owner pointed to them
  (section 1); it wrote nothing there and started none of that pack's tasks.
- **Test sessions and the login session.** The task ran the installed application only in
  private nested sessions: the end-to-end test's, and thirty-four debugging sessions of the
  same kind up to the close, counted from the session's record; what the recheck after the
  close looked at is told in `receipt/RECEIPT_RESULT.md`. The harness of a debugging session
  ends it four minutes after it starts the application, at the latest; most were ended
  sooner by the script that used them. The owner's login session carried its lock mark each
  time the task read it with `loginctl show-session` (`LockedHint=yes`, at 09:56 in the
  header of the end-to-end log, and again after the pause); the mark was there before the
  task began (TASK-010's record, section 11). The task sent that session no command and did
  not set or reset the mark. A nested session started under that mark starts a lock-screen
  program of its own, which writes one line to the user journal when the session ends; that
  is the only trace the sessions left outside their folders.
- **A second installation.** For the observation of an earlier build's profile the task
  installed this build a second time, in
  `/mnt/F/Nexees-toolchains/target/nexees/desktop-install-earlier`, beside the first, and
  changed three places of that copy. It was removed when its session ended, and is not there
  now.
- **A folder beside the toolchains.** The copy of the evidence scripts in
  `/mnt/F/Nexees-toolchains/task-tools/`, outside the repository, was kept up to date for
  the rest of this run of tasks.
- **Session hook.** A hook asks for a dynamic web-application security scan after each commit.
  The application serves no web application, and `HAWK_API_KEY` is unset, so none was run.
- **Owner-only state.** cargo-deny was executed from the owner's `~/.cargo/bin`. One command
  of the task ran the system's cargo with the owner's cargo home by mistake, and cargo
  updated its bookkeeping file `~/.cargo/.global-cache` (section 11); no other file of the
  owner's toolchain and cache folders was written by a command of the task. HopToDesk,
  linger, the phone, the LCL SDK, the owner's `lcl-remote` service and the owner's VS Code
  settings were not touched.

## 11. Deviations and findings

- **The first sizes were a rough reading.** The title row was first built 26 pixels high and
  the window controls 32 wide, from a first reading of the picture with one decimal of scale.
  When the measurement was written as a script that takes each boundary from the middle of
  its soft edge, the row measured 25.25 and the control 30.30, and the tokens were corrected
  to 25 and 30 before any log of this folder was written. The logs, the pictures and the
  checks are all of the corrected sizes.
- **The share of the bottom panel needs a window tall enough.** Theia does not let the bottom
  panel be lower than 135 pixels. In a window about 530 pixels high, 27 % is less than that,
  and the panel takes its minimum. The end-to-end test's screen was enlarged from 1280 by 800
  to 1600 by 1000, so that its window shows the shares themselves.
- **The end-to-end test clicked too early.** A reopened window fades its loading screen out
  over the workbench for a moment, and a click in that moment is lost. The test's first click
  on a toggle in a reopened window was lost that way. The test now counts a window as ready
  only once the loading screen is gone. The same race had been present in the test since
  TASK-009 without a click early enough to meet it.
- **Escape and the About dialog.** The test closed the About dialog with one press of Escape
  and did not look whether it closed; the command palette, still closing, sometimes takes
  that key. With the layout checks following the dialog, the test now presses until the
  dialog is gone.
- **A seeded defect that hid the toggles.** The first version of one seeded defect moved the
  toggles to the left end of the row by their painting order. That also put them under the
  layer that moves the window, so no click reached them and the run stopped at a timeout.
  The defect was replaced by a gap between the toggles and the window controls. What it
  showed is true of the row as built: an item of the row must come after that layer, as
  every item does.
- **Theia does not remember the area of a hidden right sidebar.** A window closed with its
  right sidebar hidden comes back with it hidden, and shown again it has its areas and its
  width, but the first area, not the one that was selected. Seen in a run of the test during
  the work. Theia stores which view a side panel shows only while it shows one. The check of
  the reopened window does not name the area for that reason. Whether Nexees remembers it is
  TASK-012's question.
- **Toggling a view hides its sidebar.** Theia's command that toggles a view, run while that
  view is the one shown, hides the side panel. For the right sidebar that means the whole
  sidebar. Seen with the Outline view; the toggle in the title row shows it again.
- **The system's title bar.** Theia lets the user choose its `native` title bar style in the
  settings. The window then has the system's frame and menu and no title row, so the toggles
  are not on the screen; the two commands still work. This was observed with a window started
  as after that choice, not by making the choice in the settings and restarting.
- **A profile keeps the frame of its last window.** Theia stores with a profile whether the
  profile's last window had the system's frame, and what it stored outranks the
  application's configuration. So a profile that a build before this task had used keeps the
  system's frame under this build: no title row and no toggles, with the layout restored,
  the five areas added and the two commands working. Seen with a stand-in for such a
  profile, not with a build of TASK-010 (section 7). The way to the title row is then
  Theia's setting of the title bar style and a restart, which the task did not try. No such
  profile exists outside the test sessions: no task has installed Nexees for the owner, and
  the owner's home folder holds no Nexees profile and no installed copy. Whether Nexees
  should decide the frame itself for an older profile belongs with what TASK-012 decides
  about stored state.
- **A tooltip the foundation leaves behind.** When a tab is closed by its cross and no other
  tab slides under the mouse, Theia shows that tab's tooltip half a second later at the
  window's top left corner, and it stays until the next click. The tab bar takes the press of
  the mouse for itself, so Theia's tooltip service never hears the click that should cancel
  the tooltip it was about to show. It happens in the right sidebar's row with the Outline
  view, and in the same way in Theia's own bottom panel, which this task did not touch
  ([logs/live_observations.txt](logs/live_observations.txt)). It is the foundation's
  behaviour in version 1.76.0 and was left as it is; the areas themselves have no cross.
- **The pause.** The session stood still from about 10:01:47 to 11:32:36 (section 1). The
  last command before it, a debugging session, had ended. A run of the seeded defects, of
  09:57, had ended as not as expected just before; it was repeated after the pause with the
  corrected defect.
- **One write in the owner's cargo home.** The task's commands use a toolchain of their own,
  in `/mnt/F/Nexees-toolchains`, which each command loads for itself. At 11:59:45 the task
  ran its script that records the machine's other activity in a command that had not loaded
  it. That script regenerates the lockfile in a scratch copy of the manifests, to show that
  the checkout's is current. Without the task's toolchain that was the system's cargo,
  `/usr/bin/cargo`, with the owner's cargo home. It ran offline and in the scratch copy, and
  it changed one file of the owner's: `~/.cargo/.global-cache`, in which cargo records when
  the files of its caches were last used. No other file under `~/.cargo` was written,
  nothing was downloaded, and the checkout was not touched. The write cannot be taken back,
  and the task did not try to. The log of that run said that the regenerated lockfile
  differed from the checkout's, as a resolution by another cargo from another cache can; the
  close wrote the log again with the task's toolchain
  ([logs/ide_activity.txt](logs/ide_activity.txt), which tells the write as the task's). The
  task found the mistake when it read that log again, about half an hour later. Its scripts
  now refuse to run without the task's toolchain.
- **Three closes did not complete.** The first two were stopped by the task while their
  negative tests ran. The first, because reading the documents again in the meantime had
  found three sentences to correct, in the user manual, in the conventions and in the header
  comment of the end-to-end test. The second, because the task had found the write in the
  owner's cargo home, which this record had to tell. Each time the processes of the close
  were stopped and its scratch copy of the checkout was removed. The third ran to its
  corroboration, which refused it: one negative test had not passed. The defect that test
  seeds, the shell folder's slot renamed in the subsystem catalogue, was caught by the layout
  stage, with a finding for each file of the folder; but the test expected that finding in
  other words than the stage uses. No run of the negative tests had completed before. The
  expected words were corrected to the stage's, the receipt folder of that close was
  removed, and the close was run again from its beginning. Every log the close writes, and
  the receipt, are of the close that completed.
- **A scratch script that hung.** After the pause, one run of the script that makes the live
  observations waited for a file that the harness of the session before it had just removed,
  and waited for ten minutes, while its second session ran its four minutes unobserved. The
  command was stopped by its process number, the waits of the script were given limits, and
  the run was repeated. Nothing of it reached the checkout or the logs.

## 12. Open items and limits

- **A question for the owner: the samples and the PC layout.** The web editor sample differs
  from the approved Desktop picture in three points (section 5): a taller header in place of
  the thin title row, names under the activity bar's icons, and the bottom panel under the
  editor alone. The task kept the approved Desktop picture. If the owner wants any of the
  three on the PC, it is a change to this layout: the second and third are small, the first
  would go against the Desktop UI contract's one thin row and needs the owner's word.
- **Nobody has looked at the real screen.** Every picture the task saw is of the window in a
  private test session, in memory. Whether the title row sits right under the owner's
  desktop, and how the frameless window casts its shadow there, was not tried: installing
  into the owner's session needs the owner.
- **Moving, resizing and minimising the window.** The test session's pointer is the test's,
  not a compositor's: the test checks which parts of the row are set to move the window, and
  that maximising and closing work, but it cannot drag the window, pull its edge or see it
  minimised.
- **The icons are the task's choice.** The two toggles use the Codicons Visual Studio Code
  uses for the same controls. The five areas have an icon each, shown only where Theia shows
  a view by its icon alone, for instance in the activity bar after a view was dragged there:
  a robot face, scales, a checklist, speech bubbles and a list of output. They are one line
  each in `assets/theme/icon_mapping.json`.
- **The keyboard and the right sidebar's tabs.** The toggles are reached with the Tab key.
  The tabs are not, as with Theia's other tab rows. No command selects an area yet; the
  tasks that fill the areas bring theirs.
- **What Nexees keeps of a layout** the user leaves, and where, is TASK-012's, with the area
  of a hidden sidebar (section 11). Theia's own storage restores the rest today.
- **The areas** are filled by their own tasks; each then gives its area's ID its own view
  (`docs/engineering/CONVENTIONS.md`, section 15).
- **The placeholders** `activity_bar.source` and `status_bar.source` wait for the first task
  that gives those regions something of Nexees.
- **The shared strings.** `assets/strings/english.source` is still a placeholder. The names of
  the five areas are vocabulary both clients will share; they stand in one place of the
  Desktop code until the first task that builds the second client's copy.
- **Also open, unchanged:** the test session and the login session's lock mark, and the real
  login, logout and lock runs of TASK-009 (PD-STARTUP), as TASK-010's record states them; the
  open items of `docs/dependencies` (OI-01 to OI-10), the open security decisions of
  `docs/security` (OSD-01 to OSD-09) and AMEND-075-01, as their records state.

## 13. Next task

TASK-012 (Implement core Desktop panel/view persistence), within the owner's assigned range.
It was not started.
