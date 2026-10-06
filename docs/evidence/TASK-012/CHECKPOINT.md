# TASK-012 — checkpoint, not a closure

**Status: in progress. TASK-012 is not accepted and has no receipt.** This file marks the save
points that the owner asked for before pauses of the development session: on 2026-10-05 at
15:10, "Commit and sync everything. And pause. we'll continue later" (commit `34ac37c`), and on
2026-10-06 at 13:21, "Pause, now, commit and Sync" (the commit that carries this version). The
task's evidence record replaces this note when the task is closed.

## What the commits hold

- **The panel memory, complete in code.** The Desktop window remembers which of its left
  sidebar, right sidebar and bottom panel are shown, their sizes and their selected views.
  The window gives that layout to the Nexees host, which keeps it in the device's state
  store, and asks for it when it starts (`apps/desktop/src/shell/panel_memory.ts`,
  `apps/desktop/src/main.ts`, `apps/desktop/src/application_host.rs`).
- **The shared model.** The layout is a typed record of `core/domain/client.rs`, carried by
  the new layout messages of `core/protocol` (protocol version 2) and kept as
  `nexees.client.layout` in `core/state` (schema version 2, the first migration step after
  the initial one).
- **Tests.** New tests of the domain, protocol, state and host crates, one new tooling test,
  and eight new end-to-end checks in `tests/e2e/desktop/local_application.test.mjs`.
- **Records and manuals.** The architecture and security records, the conventions, the README
  and the user manual say what is kept and where.
- **A draft of the evidence record** ([TASK-012_EVIDENCE.md](TASK-012_EVIDENCE.md)) and the
  logs written so far. Sections 1 to 4 of the draft tell the pause and the resumption; what
  it says of the seeded defects, the debugging sessions, the findings and the receipt is not
  yet complete or not yet true.

The logs of the preflight and of the preflight at the resumption, the native language suite,
the dispatch, the required reading, the Desktop build and the end-to-end test are real. So is
[logs/seeded_defects.txt](logs/seeded_defects.txt), whose last run did not pass (below). The
other seven files in `logs/` are placeholders that say so: the check runner, the negative
tests, the regression, the security scan, the IDE activity, the live observations and the
cleanup record.

## Done since the first checkpoint

The session resumed on 2026-10-06 at 12:50, with Claude Opus 5.5 as its model at the owner's
choice. The machine had been restarted during the pause, which emptied the session's scratch
folder; the task's tools were restored from their copy in
`/mnt/F/Nexees-toolchains/task-tools/`. The preflight at the resumption passed, and the LCL
engine and repository were unchanged ([logs/preflight_resume.txt](logs/preflight_resume.txt)).

- **Seeded run 2 no longer stops the test.** With the window's last moment seeded to keep
  nothing, the right sidebar stayed hidden in the windows after the first reopened one, and
  the test stopped where it measured that sidebar. The test now shows the sidebar first when
  it finds it hidden; the loss itself is caught by the check of a change made in the moment
  before the window closes.
- **The check that the window is dark while it loads** (TASK-010) had failed once, in seeded
  run 2 of 2026-10-05. Fourteen starts of the first window with probes showed the window's
  page target existing about a second before the application's page, with no URL, and the
  application's page preferring dark from its first answer; none of them explains the
  failure, which needs the page to have answered "not dark", or never. The probe now asks
  only the application's page, and the check's detail now says what the page prefers and
  what it painted.
- **The seeded runs of 13:02** each failed exactly the checks of their defects. The run of the
  restored installation after them then failed a check of TASK-011: "timed out waiting for
  the window to have its size again" after the window's maximise control. The compositor's
  log shows about forty quiet seconds there, which fits both of the step's twenty-second
  waits running out. Probes then maximised and restored the window 405 times in the same
  state without a fault, and the cause is not established. The test's layout waits now say
  what the window last showed when they time out.
- The build of 13:20 is of the sources of this commit
  ([logs/desktop_build.txt](logs/desktop_build.txt)), and the end-to-end run after it, with
  this commit's test file, is [logs/e2e_local_application.txt](logs/e2e_local_application.txt).

## What is still to do before TASK-012 can close

- **The seeded defects again**, with the final test file: [logs/seeded_defects.txt](logs/seeded_defects.txt)
  must end with every run as expected and the restored installation passing. If the window
  controls' check fails again, its detail now says where the window stood.
- The observation log of one debugging session, the check runner's log, the full negative
  tests, the regression run, the security scan, the IDE activity log, the cleanup record and
  the final verification.
- The evidence record: the seeded runs as they end, the findings above, the count of the
  debugging and probing sessions from the session's record, and the removal of this note and
  of the draft note; the corroboration of the preflight at the resumption.
- The receipt with its closure record, the corroboration and the acceptance.
- Then, in the owner's order: report, recheck, report again, the closing commit and push, and
  the cleanup.
