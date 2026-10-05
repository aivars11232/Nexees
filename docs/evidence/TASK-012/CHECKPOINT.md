# TASK-012 — checkpoint, not a closure

**Status: in progress. TASK-012 is not accepted and has no receipt.** This file marks a save
point that the owner asked for on 2026-10-05 at about 15:15, before a pause of the development
session: "Commit and sync everything. And pause. we'll continue later". The task's evidence
record replaces it when the task is closed.

## What this commit holds

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
  logs written so far. What the draft says of the final checks, the seeded defects, the
  debugging sessions and the receipt is not yet true.

At this commit `scripts/test/run_checks.py` passes all nine stages, and the end-to-end test
passes its 67 checks against the application built and installed from these sources
([logs/desktop_build.txt](logs/desktop_build.txt),
[logs/e2e_local_application.txt](logs/e2e_local_application.txt)).

The logs of the preflight, the native language suite, the dispatch, the required reading, the
Desktop build, the end-to-end test and the seeded defects are real. The other seven files in
`logs/` are placeholders that say so: the check runner, the negative tests, the regression,
the security scan, the IDE activity, the live observations and the cleanup record.

## What was found on the afternoon of the checkpoint

The seeded defects of 14:14 had not failed the checks expected of them. Examining why gave
these results, which the draft record does not hold in full yet:

- **The settle works.** The window samples its panels once they have stayed unchanged for
  300 milliseconds. A probe in the page showed the timer firing 301 milliseconds after the
  last change. The end-to-end test's first window changes its panels in steps at most 151
  milliseconds apart for about a second and a half, so nothing was sampled there until the
  window closed. That is the settle doing what it says, not a defect.
- **Fixed: a sidebar hidden before any sample lost its view.** A hidden sidebar shows no
  view, and the layout took its view from the last sample. With no sample taken while the
  sidebar was shown, the host kept it without a view, and it came back with its first one.
  The layout now names the tab Theia notes as the one a collapsed sidebar returns to. The new
  check 1 of the draft's section 7 covers it.
- **Fixed in the test: the check of an unreachable host did not reach that path.** The test
  let a host be reached again at once after the user's change, before the window had tried
  to give the change to anyone. The window's first attempt then met a reachable host, and a
  backend seeded to forget what it could not deliver passed the check. The test now waits
  until the window's profile holds the change, and that seeded defect is caught.
- **The seeded runs were redesigned** so that each expected set follows from what each defect
  reaches. Their last run, 15:04 to 15:14, is [logs/seeded_defects.txt](logs/seeded_defects.txt):
  runs 1 and 3 fail exactly their checks, and the restored installation passes all 67. Run 2
  does not, for the two reasons below.

## What is still to do before TASK-012 can close

- **Run 2 of the seeded defects.** With the window's last moment seeded to keep nothing, a
  change of an earlier window is lost (a sidebar shown again and the window closed at once),
  the host's older layout with the right sidebar hidden wins in the windows after it, and the
  test stops with an error where it reads the right sidebar's place, instead of failing a
  check. To do: make the test fail a check there, or seed that defect in a run of its own,
  and run again.
- **One failure not understood.** In the same run the earlier check "the window is dark while
  it loads, before any theme is applied" failed with "first painted nothing yet". It passed in
  every other run of this task. Whether the test can ask the page before its first paint was
  not examined; no cause is established.
- The observation log of one debugging session, the check runner's log, the 22 negative
  tests, the regression run, the security scan, the IDE activity log, the cleanup record and
  the final verification.
- The evidence record: the findings above, the seeded runs as they end, and the count of the
  debugging and probing sessions from the session's record.
- The receipt with its closure record, the corroboration and the acceptance.
- Then, in the owner's order: report, recheck, report again, the closing commit and push, and
  the cleanup.
