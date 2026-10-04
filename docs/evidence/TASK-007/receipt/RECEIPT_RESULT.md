# TASK-007 receipt result

**TASK-007 is accepted.** The pack's acceptance program accepts the corroborated
receipt: `status.succeeded`, exit code 0.

This receipt replaces the one of TASK-007's first close (commit `9f28c37`, engine record
`3b1ed6075afa85715198b7002a0ba4724162d3397b8aaf5d4d059038added23b`). That close left
TASK-004's cross-check failing one assertion as an expected result. After the owner asked to
finish rather than stop, a follow-up repaired the check and refreshed the evidence (evidence
record, section 11). The first receipt stays in Git history at `9f28c37`.

This folder was written after everything else in the follow-up, because the receipt has to
contain a snapshot of the rest of the worktree. The snapshot leaves this folder out, so
writing it did not change the state that was verified.

## Corroboration

Under CA-05 of the continuation profile, routine corroboration is done by the primary
session's own local tools. [corroboration.txt](corroboration.txt) records 41
observations, all confirmed:

- the pack's manifest and content identity;
- the checkout, branch and remote, with nothing staged. HEAD is `9f28c37`, TASK-007's first
  commit on the TASK-006 commit `fc42a93`, unchanged during the follow-up;
- that the first close's accepted engine record is kept in Git history at `9f28c37`;
- the follow-up's preflight log: its start at `9f28c37`, equal to `origin/main`, and TASK-004's
  check failing there as recorded;
- that the dispatch ran task 7 only;
- the accepted TASK-001 to TASK-006 predecessors and their evidence. TASK-004's evidence is
  unchanged except `check_threat_model.py`, which equals its accepted text with exactly the
  recorded repair. The charter is unchanged since TASK-001, and `docs/security` and the manual
  sources are unchanged by the task;
- that the files the task changed and removed since `fc42a93` are exactly those the evidence
  record lists;
- the presence and hashes of the 14 evidence files then listed (`corroboration.txt`, which
  that step wrote, was added afterwards, making 15);
- that the only ignored files are the owner's IDE output, so the commit carries every
  evidence file;
- the results recorded in every log:
  - the check runner, with the Android builds of both crates;
  - the negative tests, each test-stage case caught by named failing tests;
  - the regression: 25 commands exit 0. TASK-004's repaired check passes on the final tree
    and on TASK-004's own accepted tree, and fails on a copy where one enforcement point has
    no file;
  - the native suite, the security scan, the IDE record, the preflight and the reading
    record;
  - the final verification, with every exit 0;
- that the receipt's 31 check IDs equal the task's, each accounted for in the evidence
  record;
- the 135 policy IDs;
- the renderer decision against the owner's recorded answer;
- that each of the eight assigned remote requirements and three scenarios has its row in
  the evidence record, with what TASK-007 closed and its runtime owners;
- the closure record (next section);
- the snapshot, recomputed.

On that record, and on nothing else, `evidence_verified_by_host` was set to true and the
blocker `UNVERIFIED_RECEIPT` was cleared in [receipt.json](receipt.json), and
`corroboration.txt` was added to its evidence files. Every other value is unchanged.

This is the agent's self-corroboration with deterministic tools. **It is not an
independent human or model review, and the owner has not reviewed TASK-007.**

## Closure record

[receipt.json](receipt.json) carries the closure record `closure`, schema
`nexees-task-closure` version 1. The runner's own validator accepts it, and so does the
evidence stage of `run_checks.py`.

| Part | Recorded |
|---|---|
| Readability | The agent's own review against the final code, with the changes it led to (evidence record, section 9). No independent review. |
| Cleanup | [logs/cleanup_record.json](../logs/cleanup_record.json), applied with an empty manifest. Nothing was removed or refused. One untracked file was left in place, the follow-up's preflight log, with a hash that equals the snapshot listing. Every other TASK-007 file is tracked since the first commit. 279 ignored files were also left in place, all the owner's IDE output (`target/` and TASK-003's `.gradle/`). |
| Manual impact | `not_applicable`: the protocol is not yet used by any running host, and both manual sources are unchanged. |
| Final verification | [final_verification.txt](final_verification.txt), `passed`, on this receipt's snapshot. |

## Order of the close

The follow-up began with its preflight log, then repaired TASK-004's check and ran the
regression. Then the evidence record and the last logs were made final:
- the IDE record;
- the security scan of every file the task changed since `fc42a93`;
- [logs/run_checks.txt](../logs/run_checks.txt).

The scoped cleanup ran after them, at 14:05:59, and a final verification passed at 14:06:06.
Two later corrections each reopened that order: the receipt folder was removed, the text
corrected, and the cleanup and final verification redone.
- The cleanup text named "the regression's copy", but the regression makes several copies. The
  wording was corrected in the evidence record and the receipt script.
- Corroboration then stopped on one observation. It expected the follow-up's preflight log to
  show a clean worktree, but the log shows its own first write as untracked: the read-only
  script ran twice, nine seconds apart. The evidence record and the corroboration now say
  exactly that.

The recorded cleanup ran at 14:07:36, and the final verification followed at once.

## What was evaluated

| Item | Value |
|---|---|
| Receipt | [receipt.json](receipt.json): the coding agent's declaration, corroborated as above |
| Acceptance program | `/mnt/F/Nexees_LCL_Implementation_Pack_v0.5.3/accept_task.lcl.txt` |
| Pack content identity | `bd5e8fce26f542c44a891e4340da3b2a84f79ab368b5cb868beb48126c2db3e1` |
| Engine | `lcl 0.9.1`, canonical Core 0.3.0 (identity `7c8d46931933fa28…1aff`) |
| Worktree snapshot | `d346ae6b1a4296e51b66a47444e82c014333645a7462ef325593efe2f5024273`, over 526 files |
| Snapshot listing | [snapshot_files.sha256](snapshot_files.sha256) |
| Predecessors | TASK-001 to TASK-006 accepted, with their engine records intact (evidence record, section 3); TASK-004's cross-check repaired by this task's follow-up |
| Renderer decision | `electron_permitted`, approved: the owner's answer to DESKTOP-RENDERER-01, recorded by TASK-003 |
| Remote observations | All eight `unverified`: they describe the shipped product and are required only for tasks 70 and 73 to 75. TASK-007's remote requirements and scenarios are closed for their protocol contracts, types and test plan only (evidence record, section 8). |
| Evaluated on | 2026-10-04 |

The snapshot is what a commit of the task holds (CONVENTIONS.md section 10). It is the
SHA-256 of one `sha256sum` line per file Git would commit, tracked or untracked but not
ignored, outside this folder, with paths sorted under `LC_ALL=C`. The owner's IDE output is
ignored and outside it.

It is the same value before and after the final verification and at corroboration.

## Result

| | Value |
|---|---|
| Purpose | `implementation_review` |
| Exit code | 0 |
| Terminal status | `status.succeeded` |
| Diagnostics | None |
| `VERIFY` checks | All six TRUE: `verify.selected_receipt`, `verify.snapshot_shape`, `verify.ordered_predecessors`, `verify.required_checks`, `verify.instruction_delivery`, `verify.receipt_delivery` |
| `output.receipt` | Published for `TASK-007`, with this snapshot and content identity |
| Engine record | [evaluation.record.json](evaluation.record.json), SHA-256 `3e0ce66c6fc460bd64eb7d1d435777ab52838a8f840541e83e95d3afbac619e8` |

- **Language acceptance: yes.**
- **Verification of the task evidence:** done by the primary session's local tools, as
  CA-05 permits; no other party reviewed the work.

### Before corroboration

The same receipt was first evaluated with `evidence_verified_by_host: false` and the
blocker `UNVERIFIED_RECEIPT`. It was refused: exit code 1, outcome `rejected`, with exactly
two diagnostics, the gates `validate.host_verified` and `validate.no_blocker`. That record was
not kept. Restoring those two values in `receipt.json` reproduces it.

## Final verification

[final_verification.txt](final_verification.txt) was produced after cleanup against the
frozen worktree:

- **The check runner:** `scripts/test/run_checks.py` passes all nine stages, with the 91
  domain and 30 protocol tests, the 26 tooling tests and the `lcl` check, validate and run of
  the four LCL projects. `nexees-domain` and `nexees-protocol` build for
  `aarch64-linux-android` and `x86_64-linux-android`.
- **Predecessor cross-checks:** all four pass: TASK-001's against the preserved 0.5.2
  baseline, then TASK-002's, TASK-003's and TASK-004's, repaired.
- **Pack, LCL and SDK:** the pack is intact, with no file newer than its manifest. The LCL
  repository is clean at `fa1592b`. No LCL SDK file was written since TASK-007 began.
- **Home caches:** 11 files were written since then, all by the owner's IDE
  (logs/ide_activity.txt):
  - cargo's registry-use record in `~/.cargo`, from rust-analyzer;
  - the Gradle daemon logs, and the caches touched by the Gradle extension's own builds.

  No task process is running.
- **Git:**
  - against the task base `fc42a93`: exactly the 16 expected files are modified, the 15 of
    the first close and TASK-004's repaired check; the four implemented placeholders are
    removed; the 10 new source files are added;
  - against HEAD `9f28c37`: the follow-up changes only TASK-004's check, `FILE_TREE.txt` and
    TASK-007's evidence;
  - nothing is staged;
  - the only ignored files are the owner's IDE output.
- **Snapshot listing:** it agrees with `sha256sum` over the same 526 files.
- **Layout:** 264 placeholder files still carry their marker line. That is 268 before, less
  the four implemented ones. The PNG files are unchanged, and no temporary file or build
  output is among the files a commit holds.
- **Stability:** the snapshot was the same before and after the verification.

## To reproduce

1. Confirm the pack:

   ```bash
   sha256sum /mnt/F/Nexees_LCL_Implementation_Pack_v0.5.3/MANIFEST.json
   # 8d9d0080ac13a3fb17cc70603b77eb3b11abcd3fb6d2fad515cfac136fcf07e4
   ```

2. Recompute the snapshot. It prints `d346ae6b…4273` for as long as no committable file
   outside this folder has changed:

   ```bash
   cd /mnt/F/Nexees && git ls-files -z --cached --others --exclude-standard -- . ':!docs/evidence/TASK-007/receipt' | LC_ALL=C sort -zu | while IFS= read -r -d '' f; do [ -f "$f" ] && printf './%s\0' "$f"; done | xargs -0 sha256sum | sha256sum
   ```

3. Re-run the checks, on this machine's isolated toolchain. The evidence stage also
   validates this receipt's closure record:

   ```bash
   cd /mnt/F/Nexees
   source /mnt/F/Nexees-toolchains/env.sh
   export PATH="$PATH:$HOME/.cargo/bin" NEXEES_LCL_CORE_01=/mnt/F/LCL/canonical/LCL_Core_0.1.0 NEXEES_LCL_CORE_03=/mnt/F/LCL/canonical/LCL_Core_0.3.0
   python3 -B scripts/test/run_checks.py --target-dir /mnt/F/Nexees-toolchains/target/nexees
   ```

4. Evaluate the receipt. The expected result is `status.succeeded`, exit code 0:

   ```bash
   python3 - <<'EOF'
   import json, subprocess
   inputs = json.load(open('/mnt/F/Nexees/docs/evidence/TASK-007/receipt/receipt.json'))['inputs']
   def literal(v):
       if isinstance(v, bool): return 'TRUE' if v else 'FALSE'
       if isinstance(v, (str, int)): return json.dumps(v, ensure_ascii=False)
       return '[' + ', '.join(literal(i) for i in v) + ']'
   argv = ['lcl', 'run', '--spec', '/mnt/F/LCL/canonical/LCL_Core_0.1.0',
           '--project-spec', '/mnt/F/LCL/canonical/LCL_Core_0.3.0',
           '/mnt/F/Nexees_LCL_Implementation_Pack_v0.5.3/accept_task.lcl.txt']
   for key, value in inputs.items():
       argv += ['--input', f'input.{key}={literal(value)}']
   raise SystemExit(subprocess.run(argv).returncode)
   EOF
   ```

A later task that changes any committable file outside this folder changes the snapshot.
That is expected: this receipt describes the state at which TASK-007 was accepted, which is
the state committed together with it. Committing does not change the snapshot.
