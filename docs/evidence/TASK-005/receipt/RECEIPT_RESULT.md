# TASK-005 receipt result

**TASK-005 is accepted.** The pack's acceptance program accepts the corroborated
receipt: `status.succeeded`, exit code 0.

This folder was written after everything else in the task, because the receipt has to
contain a snapshot of the rest of the worktree. It is the only folder the snapshot
leaves out, so writing it did not change the state that was verified.

## Corroboration

Under CA-05 of the continuation profile, routine corroboration is done by the primary
session's own local tools. [corroboration.txt](corroboration.txt) records 36
observations, all confirmed:

- the pack's manifest and content identity;
- the checkout, branch, HEAD (`0337be0`, the TASK-004 commit) and remote, with nothing
  staged;
- that the dispatch ran task 5 only;
- the accepted TASK-001 to TASK-004 predecessors, with their evidence unchanged and the
  charter unchanged since TASK-001;
- that the changed and removed tracked files are exactly those the evidence record lists;
- the presence and hashes of the 13 evidence files then listed (`corroboration.txt`, which
  that step wrote, was added afterwards, making 14);
- that no file of the checkout is ignored by Git, so the commit carries every evidence
  file;
- the exit codes and results recorded in every log: the check runner, the negative tests,
  the regression, the native suite, the security scan, the toolchain, the preflight, the
  reading record and the final verification;
- that the receipt's 18 check IDs equal the task's, each accounted for in the evidence
  record;
- the 135 policy IDs;
- the renderer decision against the owner's recorded answer;
- that TASK-005 has no remote requirement;
- the closure record (next section);
- the snapshot, recomputed.

On that record, and on nothing else, `evidence_verified_by_host` was set to true and the
blocker `UNVERIFIED_RECEIPT` was cleared in [receipt.json](receipt.json), and
`corroboration.txt` was added to its evidence files. Every other value is unchanged.

This is the agent's self-corroboration with deterministic tools. **It is not an
independent human or model review, and the owner has not reviewed TASK-005.**

## Closure record

TASK-005 is the first task whose receipt carries a closure record: the object `closure` in
[receipt.json](receipt.json), schema `nexees-task-closure` version 1, as
[CONVENTIONS.md](../../../engineering/CONVENTIONS.md) section 10 defines it. The runner's
own validator accepts it, and so does the evidence stage of `run_checks.py`.

| Part | Recorded |
|---|---|
| Readability | The agent's own review against the final code, with the changes it led to (evidence record, section 9). No independent review. |
| Cleanup | [logs/cleanup_record.json](../logs/cleanup_record.json): applied with an empty manifest. Nothing removed or refused; 24 untracked files left in place, all TASK-005 deliverables and evidence, each with a hash that equals the snapshot listing. No ignored file. |
| Manual impact | `not_applicable`: no user-visible behaviour changed, and both manual sources are unchanged. |
| Final verification | [final_verification.txt](final_verification.txt), `passed`, on this receipt's snapshot. |

## Order of the close

- **Final edits.** The evidence record was corrected at 00:51. Then the negative tests and
  the check runner were run for the last time, writing their logs at 00:53.
- **Cleanup.** A first cleanup run at 00:49:53 had removed nothing. Edits followed it, so
  the cleanup was run again at close, at 01:00:45, so that its record matches the final
  tree. The earlier record was replaced.
- **What changed after the runner's log.** The cleanup at close rewrote only
  `logs/cleanup_record.json`. So [logs/run_checks.txt](../logs/run_checks.txt) is the run
  before cleanup.
- **Final verification.** It ran all nine stages again at 01:00:58, on the final tree. An
  earlier pass, at 00:58:07, was repeated after the cleanup at close and its log replaced.

## What was evaluated

| Item | Value |
|---|---|
| Receipt | [receipt.json](receipt.json): the coding agent's declaration, corroborated as above |
| Acceptance program | `/mnt/F/Nexees_LCL_Implementation_Pack_v0.5.3/accept_task.lcl.txt` |
| Pack content identity | `bd5e8fce26f542c44a891e4340da3b2a84f79ab368b5cb868beb48126c2db3e1` |
| Engine | `lcl 0.9.1`, canonical Core 0.3.0 (identity `7c8d46931933fa28…1aff`) |
| Worktree snapshot | `12b09b5122ee565ef576431384adf882d240ff5ecb9498d8c265855b7eabb3e0`, over 473 files |
| Snapshot listing | [snapshot_files.sha256](snapshot_files.sha256) |
| Predecessors | TASK-001 to TASK-004 accepted, with their engine records intact (evidence record, section 3) |
| Renderer decision | `electron_permitted`, approved: the owner's answer to DESKTOP-RENDERER-01, recorded by TASK-003 |
| Remote observations | All eight `unverified`: they describe the shipped product and are required only for tasks 70 and 73 to 75. TASK-005 has no remote requirement or scenario. |
| Evaluated on | 2026-10-04 |

The snapshot is the SHA-256 of the listing: one `sha256sum` line for every file under
`/mnt/F/Nexees/` except `.git/` and this folder, with paths sorted under `LC_ALL=C`. It is
the same value before and after the final verification and at corroboration.

## Result

| | Value |
|---|---|
| Purpose | `implementation_review` |
| Exit code | 0 |
| Terminal status | `status.succeeded` |
| Diagnostics | None |
| `VERIFY` checks | All six TRUE: `verify.selected_receipt`, `verify.snapshot_shape`, `verify.ordered_predecessors`, `verify.required_checks`, `verify.instruction_delivery`, `verify.receipt_delivery` |
| `output.receipt` | Published for `TASK-005`, with this snapshot and content identity |
| Engine record | [evaluation.record.json](evaluation.record.json), SHA-256 `9554641f1fb2383d936a58cedfdc345d896a21ac28e7208e135c3dc8aead7b9b` |

- **Language acceptance: yes.**
- **Verification of the task evidence:** done by the primary session's local tools, as
  CA-05 permits; no other party reviewed the work.

### Before corroboration

The same receipt, with `evidence_verified_by_host: false` and the blocker
`UNVERIFIED_RECEIPT`, was refused (exit code 1, outcome `rejected`). It had exactly two
diagnostics, the gates `validate.host_verified` and `validate.no_blocker`. That record was
not kept. Restoring those two values in `receipt.json` reproduces it.

## Final verification

[final_verification.txt](final_verification.txt) was produced after cleanup against the
frozen worktree:

- `scripts/test/run_checks.py`: all nine stages pass, with the 22 tooling tests and the
  `lcl` check, validate and run of the four LCL projects.
- The predecessors' cross-checks pass: TASK-002's `check_architecture.py`, TASK-003's
  `check_dependencies.py` and TASK-004's `check_threat_model.py`. TASK-001's
  `check_charter_against_pack.py` passes against the preserved 0.5.2 baseline.
- The pack is intact, with no file newer than its manifest. The LCL repository is clean
  at `fa1592b`.
- Since TASK-005 began, no file in the LCL SDK or the home caches was written. No task
  process is running.
- Git:
  - exactly the 14 expected tracked files differ from HEAD;
  - the two implemented placeholders are removed;
  - every new file is in a place TASK-005 creates;
  - nothing is staged and no file is ignored;
  - the files a commit would hold equal the snapshot listing.
- 273 placeholder files still carry their marker line. That is 278 before, less the six
  files TASK-005 edited or implemented, plus the new `local_ipc.source` placeholder.
- The PNG files are unchanged, and the checkout holds no temporary file or build folder.
- The snapshot was the same before and after the verification.

## To reproduce

1. Confirm the pack:

   ```bash
   sha256sum /mnt/F/Nexees_LCL_Implementation_Pack_v0.5.3/MANIFEST.json
   # 8d9d0080ac13a3fb17cc70603b77eb3b11abcd3fb6d2fad515cfac136fcf07e4
   ```

2. Recompute the snapshot. It prints `12b09b51…b3e0` for as long as no file outside this
   folder has changed:

   ```bash
   cd /mnt/F/Nexees && find . -path ./.git -prune -o -path ./docs/evidence/TASK-005/receipt -prune -o -type f -print0 | LC_ALL=C sort -z | xargs -0 sha256sum | sha256sum
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
   inputs = json.load(open('/mnt/F/Nexees/docs/evidence/TASK-005/receipt/receipt.json'))['inputs']
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

A later task that changes any file outside this folder changes the snapshot. That is
expected: this receipt describes the state at which TASK-005 was accepted, which is the
state committed together with it. Committing does not change the snapshot, because
`.git/` is outside it.
