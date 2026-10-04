# TASK-007 receipt result

**TASK-007 is accepted.** The pack's acceptance program accepts the corroborated
receipt: `status.succeeded`, exit code 0.

This folder was written after everything else in the task, because the receipt has to
contain a snapshot of the rest of the worktree. The snapshot leaves this folder out, so
writing it did not change the state that was verified.

## Corroboration

Under CA-05 of the continuation profile, routine corroboration is done by the primary
session's own local tools. [corroboration.txt](corroboration.txt) records 39
observations, all confirmed:

- the pack's manifest and content identity;
- the checkout, branch, HEAD (`fc42a93`, the TASK-006 commit) and remote, with nothing
  staged;
- that the dispatch ran task 7 only;
- the accepted TASK-001 to TASK-006 predecessors, with their evidence unchanged, the charter
  unchanged since TASK-001, and `docs/security` and the manual sources unchanged;
- that the changed and removed tracked files are exactly those the evidence record lists;
- the presence and hashes of the 13 evidence files then listed (`corroboration.txt`, which
  that step wrote, was added afterwards, making 14);
- that the only ignored files are the owner's IDE output, so the commit carries every
  evidence file;
- the results recorded in every log:
  - the check runner, with the Android builds of both crates;
  - the negative tests, each test-stage case caught by named failing tests;
  - the regression, including TASK-004's one expected failure and its full pass with
    stand-ins;
  - the native suite, the security scan, the IDE record, the preflight and the reading
    record;
  - the final verification, whose only non-zero exit is that same expected failure;
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
| Cleanup | [logs/cleanup_record.json](../logs/cleanup_record.json), applied with an empty manifest. Nothing was removed or refused. 20 untracked files were left in place, all TASK-007 deliverables and evidence, each with a hash that equals the snapshot listing. 279 ignored files were also left in place, all the owner's IDE output (`target/` and TASK-003's `.gradle/`). |
| Manual impact | `not_applicable`: the protocol is not yet used by any running host, and both manual sources are unchanged. |
| Final verification | [final_verification.txt](final_verification.txt), `passed`, on this receipt's snapshot. |

## Order of the close

The evidence record and the last logs were final first:
- the IDE record;
- the security scan of the final files;
- [logs/run_checks.txt](../logs/run_checks.txt).

The scoped cleanup ran after them, at 13:49:36. The final verification followed at 13:50:24.
An attempt at 13:49:45 stopped on a defect of the verification script itself. Its filter for
TASK-004's failed checks also caught that check's closing `FAILED:` summary, so it counted
two failures instead of one. The filter was made exact, and the attempt's output was removed
before the recorded run.

## What was evaluated

| Item | Value |
|---|---|
| Receipt | [receipt.json](receipt.json): the coding agent's declaration, corroborated as above |
| Acceptance program | `/mnt/F/Nexees_LCL_Implementation_Pack_v0.5.3/accept_task.lcl.txt` |
| Pack content identity | `bd5e8fce26f542c44a891e4340da3b2a84f79ab368b5cb868beb48126c2db3e1` |
| Engine | `lcl 0.9.1`, canonical Core 0.3.0 (identity `7c8d46931933fa28…1aff`) |
| Worktree snapshot | `048eff4032f7f91bc50d4f0bc607bd69fc186328dea419e51db004f44d8b2c07`, over 525 files |
| Snapshot listing | [snapshot_files.sha256](snapshot_files.sha256) |
| Predecessors | TASK-001 to TASK-006 accepted, with their engine records intact (evidence record, section 3) |
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
| Engine record | [evaluation.record.json](evaluation.record.json), SHA-256 `3b1ed6075afa85715198b7002a0ba4724162d3397b8aaf5d4d059038added23b` |

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
- **Predecessor cross-checks:** TASK-001's, against the preserved 0.5.2 baseline, TASK-002's
  and TASK-003's pass. TASK-004's frozen check exits 1 as expected: 37 checks pass and only
  the one that counts an enforcement point solely as a `.source` placeholder fails, because
  three of its points are now source. On a copy of the files a commit holds, with empty
  stand-ins for those three placeholders, it passes in full.
- **Pack, LCL and SDK:** the pack is intact, with no file newer than its manifest. The LCL
  repository is clean at `fa1592b`. No LCL SDK file was written since TASK-007 began.
- **Home caches:** 11 files were written since then, all by the owner's IDE
  (logs/ide_activity.txt):
  - cargo's registry-use record in `~/.cargo`, from rust-analyzer;
  - the Gradle daemon logs, and the caches touched by the Gradle extension's own builds.

  No task process is running.
- **Git:**
  - exactly the 15 expected tracked files differ from HEAD;
  - the four implemented placeholders are removed;
  - every new file is one of the 10 new source files or in `docs/evidence/TASK-007/`;
  - nothing is staged;
  - the only ignored files are the owner's IDE output.
- **Snapshot listing:** it agrees with `sha256sum` over the same 525 files.
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

2. Recompute the snapshot. It prints `048eff40…2c07` for as long as no committable file
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
