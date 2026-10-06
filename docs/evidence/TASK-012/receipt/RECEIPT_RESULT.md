# TASK-012 receipt result

**TASK-012 is accepted.** The pack's acceptance program accepts the corroborated receipt:
`status.succeeded`, exit code 0.

This folder was written after everything else in the task, because the receipt has to
contain a snapshot of the rest of the worktree. The snapshot leaves this folder out, so
writing it did not change the state that was verified.

## This is the second close

TASK-012 was closed twice on 2026-10-06, and the close sequence was started three times.

- **At 17:51:11** it ran through the check runner, the negative tests, the regression, the
  security scan, the record of other activity on the machine and the cleanup, and stopped
  at its final verification, at 17:58. That found 52,891 files written in the user's home
  caches while the task worked that its rules could not attribute: the npx folder of an MCP
  server of the owner's Claude Code plugins, rewritten at 13:23 by a program that was not
  the task's (evidence record, sections 10 and 11). No receipt existed. The folder with
  the failed verification's output was removed, the verification and the activity record
  were taught to attribute npm's writes by npm's own logs, and nothing of the task changed.
- **At 18:07:15** the close was run again from its start. It was accepted at 18:16:35:
  snapshot `95b1b87a1097a457c1bacf08109bb7baa7b7e2ff234b3606457dadfd2f53e4a3` over 673
  files, engine record `70fa20002bf46e4ba6e89b54a135b29ba0a8c1314dc5d2ff2bcac84e3ec2c0a3`,
  54 corroborated observations. The receipt's declaration of who performed the task was
  then made exact, since it had said that the model changed at "the owner's pause" and
  there were two pauses; that text is no input of the acceptance program, and the record
  evaluated again came out byte for byte the same. That close was reported to the owner
  and was not committed, because the owner's order puts a recheck between the report and
  the commit.
- **The recheck**, at 18:19:16, ran every check again with that receipt present, and all of
  it held. Its manual part read the code and the evidence record again, claim by claim
  against the logs, and found three statements the logs did not bear out exactly: how long
  a window's page target exists before its page, what the nested compositor's log shows
  during the failed step of TASK-011's check, and how long before a stop the observed
  changes were made (evidence record, sections 5 and 11). They were corrected. That changed
  a file of the snapshot, so the first receipt no longer described the tree: its folder was
  removed, and nothing of the first close reached Git.
- **At 18:22:05** the close was run again from its start, and completed at 18:31:59.

Everything below describes the second close.

## Corroboration

Under CA-05 of the continuation profile, routine corroboration is done by the primary
session's own local tools. [corroboration.txt](corroboration.txt) records 54 observations,
all confirmed:

- the pack's manifest and content identity;
- the checkout, branch and remote, with nothing staged. HEAD is `fd28f27`, the second of
  the two checkpoints the owner asked for before pausing, equal to `origin/main`. Both
  checkpoint commits are marked as TASK-012 in progress and hold only files of this task;
- that the dispatch ran task 12 only;
- the accepted TASK-001 to TASK-011 predecessors, each with its evidence unchanged. The
  charter is unchanged since TASK-001. No dependency record, manifest or lockfile changed;
  of the security records, the boundary and the threat of the window's channel did. The
  user manual is the one manual source the task changed;
- that the files the task added, changed and removed since its base `8bfd07a` are exactly
  those the evidence record lists: 1 new and 24 changed, none removed;
- the presence and hashes of the 19 evidence files then listed (`corroboration.txt`, which
  that step wrote, was added afterwards, making 20), and that the checkpoint note is gone;
- that the only ignored files are the owner's IDE output and the editor's link, so the
  commit carries every evidence file;
- the results recorded in every log:
  - the check runner, with the Android builds of the three core crates;
  - the negative tests: 22 seeded defects caught, the 10 test-stage cases by named failing
    tests, and the two controls;
  - the defects seeded against the end-to-end checks of the panel memory: in each of the
    three runs exactly the checks of the seeded defects fail, 17 over the three runs, and
    the restored installation passes again;
  - the regression: 25 commands exit 0;
  - the Desktop build and installation, where the installed application carries the panel
    memory in its page, the layout messages in its backend and the record of a client's
    layout in its host, and the 67 checks of the installed application;
  - the live observations: one debugging session with three observations, all as
    expected;
  - the logo script's own check of the committed logo files, and the committed Desktop copy
    of the design tokens, which is what the sources give;
  - the native suite, the security scan, the preflight at the start and the two at the
    resumptions, and the reading record;
  - the record of other activity on the machine: no Rust manifest changed, the lockfile
    equals an offline resolution, and every home-cache write is attributed, none of them
    to the task;
  - the final verification, with every exit 0 and the installed application's 67 checks;
- that the receipt's 17 check IDs equal the task's, each accounted for in the evidence
  record;
- the 135 policy IDs;
- the renderer decision against the owner's recorded answer;
- that the task is assigned no remote requirement and no scenario;
- the closure record (next section);
- the snapshot, recomputed.

On that record, and on nothing else, `evidence_verified_by_host` was set to true and the
blocker `UNVERIFIED_RECEIPT` was cleared in [receipt.json](receipt.json), and
`corroboration.txt` was added to its evidence files.

This is the agent's self-corroboration with deterministic tools. **It is not an
independent human or model review, and the owner has not reviewed TASK-012.** It confirms
that the logs say what the record says, not that the record asks the right questions. In
particular, nobody has looked at the panels on a real screen (evidence record, section 12).

## Closure record

[receipt.json](receipt.json) carries the closure record `closure`, schema
`nexees-task-closure` version 1. The runner's own validator accepts it.

| Part | Recorded |
|---|---|
| Readability | The agent's own review against the final code, with the changes it led to, and what the examination of the seeded defects mended (evidence record, sections 9 and 11). No independent review. |
| Cleanup | The task left no temporary artifact in the checkout; the checkpoint note was removed when the evidence record replaced it. `scripts/test/task_cleanup.py` ran with an empty manifest, removed and refused nothing, and recorded the one untracked file it left in place, the preflight of the second resumption: everything else of the task was already in the two checkpoints ([logs/cleanup_record.json](../logs/cleanup_record.json)). |
| Manual impact | `updated`: `docs/manuals/NEXEES_USER_MANUAL.md` section 2 says what the window remembers of its panels. The LCL manual is unchanged. |
| Final verification | [final_verification.txt](final_verification.txt), `passed`, on this receipt's snapshot. |

## Order of the work

**Before the first pause**, on 2026-10-05: the preflight at 13:05:31, the native
verification and the dispatch, the reading record, the implementation, and the first two
runs of the seeded defects, at 14:14 and 15:04. The checkpoint `34ac37c` was
committed and pushed at 15:18.

**Between the pauses**, on 2026-10-06 from 12:52 to 13:24: the preflight on resumption at
12:53:32, the probes of the window's early paint, the Desktop build and the end-to-end test
of 13:00, and the third run of the seeded defects from 13:02, whose run of the restored
installation failed; then the probes of the window controls, the build of 13:20 and the
end-to-end test after it. The second checkpoint `fd28f27` was committed and pushed at 13:24.

**After the second pause**, from 17:23:

- the preflight on resumption, at 17:23:46;
- the fourth run of the seeded defects, from 17:24 to 17:34, as expected throughout
  ([logs/seeded_defects.txt](../logs/seeded_defects.txt));
- two debugging sessions: the first, at 17:35:52, saw a view of Theia's own stored layout
  lost where the expectation of that time had it survive, which led to Chromium's rule for
  writing a page's storage; the second, at 17:40:12, is the log
  ([logs/live_observations.txt](../logs/live_observations.txt));
- the correction of a comment of `panel_memory.ts` to that rule, the Desktop build of the
  final sources at 17:41 and the end-to-end test after it; the page and the backend built
  are byte-identical to those the seeded runs tested;
- a full rehearsal of the negative tests, from 17:44 to 17:50;
- the two closes and the recheck between them, as above.

## What was evaluated

| Item | Value |
|---|---|
| Receipt | [receipt.json](receipt.json): the coding agent's declaration, corroborated |
| Acceptance program | `/mnt/F/Nexees_LCL_Implementation_Pack_v0.5.3/accept_task.lcl.txt` |
| Pack content identity | `bd5e8fce26f542c44a891e4340da3b2a84f79ab368b5cb868beb48126c2db3e1` |
| Engine | `lcl 1.0.0`, unchanged since TASK-011's acceptance and at both resumptions |
| Worktree snapshot | `4c379c9a2ed65e9b9fbcc3ae6ced88b0aa3e433c50a2462bd663b106ab2f0eb9`, over 673 files |
| Snapshot listing | [snapshot_files.sha256](snapshot_files.sha256) |
| Predecessors | TASK-001 to TASK-011 accepted, with their engine records intact (evidence record, section 3) |
| Renderer decision | `electron_permitted`, approved: the owner's answer to DESKTOP-RENDERER-01, recorded by TASK-003 |
| Remote observations | All eight `unverified`: they describe the shipped product and are required only for tasks 70 and 73 to 75. TASK-012 is assigned no remote requirement and no scenario. |
| Evaluated on | 2026-10-06 |

The snapshot is what a commit of the task holds (CONVENTIONS.md section 10). It is the
SHA-256 of one `sha256sum` line per file Git would commit, tracked or untracked but not
ignored, outside this folder, with paths sorted under `LC_ALL=C`. The owner's IDE output, the
editor's link and the Desktop build, which lives outside the checkout, are not part of it.

It is the same value before and after the final verification, and at the corroboration and
the acceptance.

## Result

| | Value |
|---|---|
| Purpose | `implementation_review` |
| Exit code | 0 |
| Terminal status | `status.succeeded` |
| Diagnostics | None |
| `output.receipt` | Published for `TASK-012`, with this snapshot and content identity |
| Engine record | [evaluation.record.json](evaluation.record.json), SHA-256 `00a34d0bc6b83bfd212a880551ee6a4014aab9da4ff7f08f39166ae020c095c2` |

- **Language acceptance: yes.**
- **Verification of the task evidence:** done by the primary session's local tools, as
  CA-05 permits; no other party reviewed the work.

### Before corroboration

The same receipt was first evaluated with `evidence_verified_by_host: false` and the
blocker `UNVERIFIED_RECEIPT`. It was refused: exit code 1, outcome `rejected`, with exactly
two diagnostics, the gates `validate.host_verified` and `validate.no_blocker`. That record
was not kept.

## Final verification

[final_verification.txt](final_verification.txt) was produced after cleanup against the
frozen worktree, from 18:29:16:

- **The check runner:** `scripts/test/run_checks.py` passes all nine stages. That covers the
  96 domain, 32 protocol, 31 state, 12 platform and 12 host tests, the 2 doctests and the 59
  tooling tests. It also covers the strict TypeScript compile, the npm lockfile gate, 382
  relative Markdown links, and the `lcl` check, validate and run of the four LCL projects.
  `nexees-domain`, `nexees-protocol` and `nexees-state` build for `aarch64-linux-android`
  and `x86_64-linux-android`.
- **The shared visual system:** the committed Desktop copy of the design tokens is current,
  and the logo files are the recorded source and exactly its icons, compared with the
  owner's file.
- **The Desktop application:** built offline from the final sources, installed into a
  disposable prefix with its two fuses set, and tested end to end: all 67 checks of the
  installed application pass, the 8 of the panel memory among them.
- **Predecessor cross-checks:** all four pass: TASK-002's, TASK-003's and TASK-004's, and
  TASK-001's against the preserved 0.5.2 baseline.
- **Pack, LCL and SDK:** the pack is intact, with no file newer than its manifest. The LCL
  repository is clean at `ada0b5c`, with no commit made while the task worked; the revision
  the pack binds is in its history and the canonical packages are unchanged since it. No
  LCL SDK file was written since TASK-012 began.
- **The logo source:** the owner's file was only read: last written on 2026-10-02, with the
  SHA-256 of the repository copy.
- **Home caches:** 53,300 files were written while the task worked, none by the task: five
  by the owner's VS Code and the rest by npm runs that npm's own logs name, among them the
  MCP servers of the owner's Claude Code plugins
  ([logs/ide_activity.txt](../logs/ide_activity.txt), section 4). 24,334 were written while
  the session was paused. The application itself never ran against the owner's home. No
  test folder is left in the temporary folder, and no task process is running.
- **Git, against the base `8bfd07a`:** exactly the 24 expected files are changed and the one
  new file and this evidence folder are added; nothing is removed. HEAD is `fd28f27`, and the
  two commits since the base are the checkpoints the owner asked for; nothing is staged.
  The 580 ignored files are the owner's IDE output and the editor's link, left in place.
- **Layout:** 245 files still carry their marker line: the task implemented no placeholder.
  No picture changed or was added. No temporary file or build output is among the files a
  commit would hold.
- **Snapshot:** unchanged by the verification.

## Recheck

The first close was rechecked after the first report, as told above. The second close was
rechecked at 18:32:54 on 2026-10-06, with its receipt present:

- every stage of the check runner passes, the closure record of TASK-012 among the checked;
- the end-to-end test passes its 67 checks on the installation the final verification made;
- the snapshot recomputed is the receipt's, over the same 673 files;
- the corroboration repeated makes the same 54 observations with the same result;
- the acceptance program, run again on the receipt, gives the stored record: accepted,
  `status.succeeded`;
- nothing is staged, and HEAD is `fd28f27` and equals `origin/main`;
- no test folder is left in the temporary folder, and no task process runs.

Its manual part read again the three statements the first recheck had corrected, and this
file. It started no debugging session.

## Commit

The commit of TASK-012 follows the owner's order: report, recheck, report, and only then the
commit and push. Git records it; this file does not.
