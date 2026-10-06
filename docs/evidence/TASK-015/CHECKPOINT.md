# TASK-015 — checkpoint, not a closure

**Status: in progress. TASK-015 is not accepted and has no receipt.** This file marks a save
point that the owner asked for on 2026-10-06 at 23:35:14, before a pause of the development
session: "pause everything, commit and sync the changes. we'll continu tomorrow." The task's
evidence record replaces it when the task is closed.

## What this commit holds

- **The preflight of TASK-015**, recorded when the task began at 23:35:13
  ([logs/preflight.txt](logs/preflight.txt)): the checkout at `4b68f31`, clean and equal to
  `origin/main`; the pack intact; the engine `lcl 1.0.0`; TASK-001 to TASK-014 accepted, with
  their engine records intact.
- **Nothing else of TASK-015.** No source file, test, record or manual was changed.

## Where the work stands

TASK-014 is closed, committed and pushed (`4b68f31`). TASK-015 (Decouple foreground workspace
from agent workspace authority) had just begun, and its declaration was read in full. Its
scope: persist an explicit `agent_session_id -> bound_workspace_id`, and reject implicit
rebinding caused by UI focus changes. It also carries its share of RC-09, RC-10 and RC-20 and
of the scenarios RC-T08 and RC-T13, and leaves their later integration to the tasks that own
it.

## What is still to do

When the owner resumes the session: a preflight of the resumption, the native verification
and the dispatch; the search of the records for TASK-015, by name and in the ranges of tasks
that include it; the required reading; the implementation with its tests, negative tests and
seeded end-to-end defects; the evidence record and the close; then the owner's order for each
task: report, recheck, report again, commit and sync, and cleanup.
