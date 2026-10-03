# TASK-001: owner authorizations

All instructions below were given by the owner (Aivars) to the coding agent (Claude
Code) in one session on 2026-10-03. They are copied verbatim, in the order given. They
are the authority for the Git setup of this checkout, the correction of the
specification pack and the execution of TASK-001.

Nothing here authorizes a commit, a push, a change of remotes, a renderer decision or
any work on TASK-002 or later.

## 1. Task assignment

```
Go to - /mnt/F/Nexees_LCL_Implementation_Pack_v0.5.1/      - get your self familiar with it's contence and start with task1
```

## 2. Scope

```
That's the whole point of LCL, to not allowed Agent go outside the scope or extend the scope, so if agent is tasked with task 1 it should focus on task 1 not other tasks
```

## 3. Git setup, step by step

The checkout had no Git metadata, so TASK-001 stopped at its start gate (B26, STOP-11).

Instruction to inspect first:

```
First show the actual Git root, branch, HEAD, remotes and working-tree state for /mnt/F/Nexees/, including whether it belongs to a parent repository. Do not initialize Git, move branch references or restore files yet. Preserve all existing files and “start here” markers. Then propose the smallest setup change needed to satisfy the repository binding. Keep this separate from the LCL pack investigation.
```

Approval of `git init` and `git remote add origin https://github.com/aivars11232/Nexees.git`:

```
Approved. Run exactly the two proposed commands: initialize Git in /mnt/F/Nexees/ and add the specified origin. Preserve every existing file unchanged. Then rerun the read-only preflight, show the result, and stop. No fetch, file restoration, commit, push, or implementation yet.
```

Approval of `git fetch --no-tags --no-recurse-submodules origin refs/heads/main:refs/remotes/origin/main`:

```
Approved for this setup step only:

In /mnt/F/Nexees/, fetch main from the existing origin into
origin/main. Do not fetch other branches, tags or submodules.

Keep all existing working files unchanged. Do not change local
main, HEAD or the index, and do not check out or restore files.

Compare the fetched tree with the existing local files.
Report:
- The fetched commit hash.
- Missing or additional files.
- How many differences are only “start here” markers.
- Every other difference, including .directory.

Then propose how to attach local main to that existing history
while preserving the current working files.

Stop after the report. No commit, push, file restoration,
LCL pack changes or TASK-001 implementation.
```

Approval of `git reset --mixed origin/main`, with the purpose of the marker lines:

```
The start here marker line is there for me to see the files in github, because if file is empty, github don't show them. And also it's appproved
```

Rule for the marker lines:

```
I mean, when that particular file is edited, than that line can be removed, simply because it serves no purpose after that
```

## 4. Consolidated authorization

Sent as pasted text and then confirmed by the owner as his own instruction (section 5).

```
NEXEES — CONSOLIDATED SETUP, PACK REPAIR AND TASK-001 AUTHORIZATION

WORKING DIRECTORY: /mnt/F/Nexees/
REPOSITORY: https://github.com/aivars11232/Nexees
TASK: TASK-001 — Freeze project charter and authoritative requirements

AUTHORIZATION AND STOPPING BEHAVIOR

This is my explicit approval to complete the remaining bounded setup, correct
the confirmed LCL-pack authoring/test-runner defects if still present, and then
perform TASK-001 through verification, cleanup and its completion report.

This supersedes the earlier instructions to report and stop after each Git
setup operation or scratch-copy investigation. Read-only inspection and a brief
implementation plan still come first, but do not wait for another approval
before executing this authorized sequence. Continue automatically between its
steps. Do not stop merely because a routine check passed.

Use the existing task pack and your current findings. Do not restart completed
investigations, recreate verified fixes, or rewrite the 75-task plan. This
permission does not waive security boundaries or authorize unrelated work.
Tool-enforced permission prompts must still be respected.

1. VERIFY CURRENT STATE ONCE; REUSE WHAT IS ALREADY DONE

Read the actual checkout, branch, HEAD, origin, index and working-tree state.
The last reported baseline was main at
0d6d3e5ee422182a38b793756e001ddd75df507f, matching origin/main, with 284 tracked
files and only missing-marker changes. Treat that as a prior observation, not
a substitute for current inspection.

Use the v0.5.1 pack and any scratch correction already identified in this
session. Record their exact paths and identities. Do not confuse the pack
folder with /mnt/F/Nexees/. Resolve locations from existing session evidence
and local files before asking me where they are. Never select an ambiguous
copy or overwrite a newer one.

Follow guidance/usage.lcl.txt and bindings/read_order.lcl.txt, then TASK-001's
required reading. Reuse verified reads of unchanged revisions. Read the actual
current files before editing; do not infer their contents from earlier reports.

2. FINISH MARKER RESTORATION ONLY IF IT IS STILL NEEDED

If restoration is already complete, verify and move on. Do not repeat Git init,
remote setup, fetch or reset. A missing upstream alone is not a TASK-001 blocker;
leave upstream configuration alone.

Marker restoration is authorized only if HEAD is still the exact baseline
above, the index has no staged changes, and each proposed restored path is
verified byte-for-byte to differ from that commit solely by its missing
"start here" marker. Confirm that the path has not changed again before writing.

Restore only those verified paths from that exact commit into the worktree,
sequentially. Do not restore all paths blindly. Preserve the three PNGs,
unknown files, new work and unrelated edits. If the preconditions fail, do not
restore over them; inspect whether authorized work can proceed without touching
them, and ask only if a real conflict prevents it.

After restoration, verify the restored bytes and unchanged images, confirm HEAD
and index are unchanged, then continue. No intermediate approval is required.

3. RESOLVE THE KNOWN PACK DEFECTS WITHOUT CHANGING LCL

First reuse your existing scratch-copy findings and fixes if their evidence is
current. The authorization here covers the Nexees specification pack, its test
fixtures/runner and resulting integrity/provenance metadata—not the LCL engine,
canonical packages, installed binaries or user's LCL settings.

If unresolved, preserve the original pack and work in an identified correction
copy. You may create that copy and task-owned test-output folders outside both
repositories; record their exact locations. Do not overwrite unrelated folders.

Inspect the current input semantics against the actual LCL manual/canonical
rules. Repair invocation-varying INPUT declarations that incorrectly use fixed
VALUE fields, using the documented overridable-input form where appropriate.
Preserve genuinely fixed bindings. Missing or unverified receipt information
must still fail closed; do not make evidence default to success.

Correct the runner to inspect actual machine-readable outcomes, not only exit
codes. Reuse the real LCL CLI and existing utilities; do not write a replacement
language evaluator. Demonstrate:
- Requested tasks 1, 45, 70 and 75 return their own task IDs and corresponding
  instruction content, not four successful dispatches of task 1.
- A complete, explicitly synthetic valid receipt is accepted.
- Default/unverified receipts are rejected.
- Invalid task selection and negative receipt cases are rejected for the
  intended reason, not an unrelated input, environment or typing error.

Run affected existing language tests sequentially. Capture full output and
producer exit status once. Fix further reproducible defects in this same
input/dispatch/receipt path only when the correction follows established LCL
semantics without changing task requirements or weakening acceptance.

Preserve the task-record typing correction, all 75 task identities/scopes,
read order and security requirements. Update affected pack hashes/identity and
fixtures consistently, preserving the original source provenance. Never
relabel historical verification as verification of the corrected bytes.

Once the correction passes its required checks, you are authorized to adopt
that verified copy for this session without asking again. Record its path and
identity; retain the original. Synthetic results never count as Nexees task
completion evidence. Do not postpone TASK-001 for unrelated pack enhancements.

4. START AND COMPLETE THE ACTUAL TASK-001

Use the corrected pack's real main.lcl.txt entry to dispatch task 1 and inspect
output.instructions. Read all authoritative material it requires. Do not run
project-part documents as standalone entries. Dispatch success means only that
the instructions were produced.

Execute TASK-001's existing objective: consolidate the authoritative Nexees
charter, scope and requirement-to-task traceability. Reuse and update existing
documents rather than creating competing specifications. Use LCL for governing
specification additions, following the actual pack conventions; do not invent
language syntax or pretend prose is executable verification.

Cover the agreed Desktop/Android behavior, optional LCL modes, independent
workspaces, providers/model handoff, imports, manuals, security, reuse,
readability/cleanup and the two-way remote-control requirements already assigned
to TASK-001. Account for every required TASK-001 check ID with real evidence.

This is a requirements/documentation task. Do not implement production features,
choose the Desktop renderer, finalize third-party dependencies, or begin later
tasks. Record Electron-versus-native-only as unresolved for TASK-003; do not
ask that future question now merely to complete the charter.

Record the later agreed Supervisor/Review feature as a deferred amendment to
revisit at TASK-075 before the final bug-fixing/release pass. Do not implement it
now, spawn a reviewer, or silently rewrite the current 75-task execution graph.

TASK-001's remote-control evidence concerns the design, contracts and future
runtime-test owners. Do not claim an unbuilt receiver was runtime-tested. For
build/tests genuinely inapplicable to documentation-only changes, record the
specific rationale and perform the relevant structural/traceability checks;
never report an unrun check as passed or waive a check that actually applies.

5. WORKING RULES AND PERMISSIONS

Use one primary agent. No background, parallel, delegated or reviewer AI agents.
Use one heavy process at a time. Supervised tool/test processes are not extra
AI agents. No hidden workers, blind retries or retry-until-green loops.

Work on one file and one meaningful change at a time. State its path and purpose,
read current content, make the smallest sufficient change and verify it. Use
accurate comments/docstrings for non-obvious code; no filler or unnecessary code.
Preserve user edits, valid implementations, tests, assets and historical evidence.

An expected failing regression permits fixing that demonstrated cause. Unexpected
failures require diagnosis before unrelated work; do not stack changes past them.
A retry requires a reasoned change or demonstrated transient cause. Capture logs
once, use focused checks during edits and required final gates after cleanup.

You may read relevant local source/manuals, edit TASK-001 deliverables in the
bound checkout, perform the scoped pack correction above, and run corresponding
local verification. Do not commit, stage, push, rewrite history, change remotes,
install dependencies, use paid services, publish, change system configuration or
start/enable remote services. No new network operation is authorized here.

6. CLEANUP, ACCEPTANCE AND ONE FINAL REPORT

Remove only positively identified task-owned temporary/debug/obsolete material.
Preserve original packs, corrected deliverables, useful logs and evidence. Re-run
affected checks after cleanup against the final relevant revision.

Generate the real TASK-001 receipt only from performed checks and corroborated
files/logs. Record this prompt as authorization for the bounded setup/pack changes.
Do not fabricate approvals, snapshots, test outcomes or independent review; your
own review must not be described as an independent human audit. Keep synthetic
receipt tests separate from real completion evidence.

Evaluate the real receipt with accept_task.lcl.txt. Record whether language
acceptance and actual task-evidence verification both passed. A genuinely
mandatory external approval cannot be replaced by a self-asserted TRUE.

Intended uncommitted task changes are expected. "Clean diff" means understood,
scoped changes—not deleting deliverables or committing without permission just
to make git status empty.

At the end, report once: setup result; pack repair/reused evidence and active
identity; TASK-001 deliverables; checks and outcomes; justified applicability
notes; cleanup/reverification; actual acceptance result; final Git state; blockers.
Then stop before TASK-002. Do not manufacture an intermediate stopping point.

INTERRUPT ONLY FOR A GENUINE BLOCKER

Ask only when required information cannot be recovered from authorized sources,
user work would be overwritten, scope/permissions must expand, a hard gate cannot
be satisfied within this authorization, or a new decision is necessary for the
current work. Group related questions into one precise blocker report. No routine
"shall I continue?" requests between the authorized steps.

Begin by checking what is already complete, then continue through the sequence.
```

## 5. Confirmation and the deferred amendment

The agent asked whether the pasted text was the owner's own instruction, and how to
record the Supervisor/Review feature. The owner chose "Yes, all of it" and wrote:

```
Confirmed: the consolidated authorization is my own instruction.
It supersedes the earlier instructions to stop between Git steps
and keep pack repair outside this sequence.

Testing dispatch for tasks 45, 70 and 75 tests the instruction
generator ONLY. It does not authorize implementing those tasks.
Your implementation scope remains TASK-001.

Record this agreed feature as a deferred TASK-075 amendment:

OPTIONAL REVIEW / SUPERVISOR
- Provide a Review/Supervisor section accessible from Desktop
  and Android, with a user-selectable reviewer provider/model.
- At a reviewable stop condition or checkpoint, pause the coding
  agent and automatically send the reviewer the relevant task,
  LCL rules/contracts, blocker, current changes and actual
  verification evidence.
- The reviewer returns APPROVE, REPAIR, ASK USER or REMAIN STOPPED,
  together with a short continuation instruction.
- Nexees checks the decision against previously delegated
  permissions and the reviewed workspace/task/file snapshot.
  Resume automatically only when those checks permit it.
- A reviewer cannot approve away failed tests, override hard
  security stops or user cancellation, grant itself permissions,
  or decide an undelegated owner preference.
- Include enable/disable, delegated decision categories,
  review/repair-round and usage limits, and a visible review log.
- Run coding and review sequentially by default; unrestricted
  parallel agents are not implied.
- Integrate and test this at TASK-075, before the final bug-fixing
  and regression pass and release. Keep exactly 75 numbered tasks.
- TASK-001 only records this amendment. Do not implement it now
  or launch a reviewer/subagent in this development session.

Continue the authorized sequence:
1. Skip completed, verified setup. Restore only the verified
   marker differences if restoration is still needed.
2. Repair the pack and runner in a separate copy, preserving
   the original and checking actual dispatch/receipt outcomes.
3. Use the verified corrected copy to perform TASK-001.
4. Complete task-scoped cleanup, rerun affected verification,
   and provide one final completion report.

Retain the other safeguards: preserve unrelated work, no LCL
engine changes, no commit/push, no premature renderer decision,
and no TASK-002 implementation.

Do not request another confirmation for these authorized steps.
Stop only for a genuine unresolved blocker or after TASK-001
is complete.
```
