# TASK-002 receipt result

**TASK-002 is accepted.** The pack's acceptance program accepts the corroborated
receipt: `status.succeeded`, exit code 0.

This folder was written after everything else in the task, because the receipt has to
contain a snapshot of the rest of the worktree. It is the only folder the snapshot
leaves out, so writing it did not change the state that was verified.

## Corroboration

Under CA-05 of the continuation profile the owner adopted for this task, routine
corroboration is done by the primary session's own local tools.
[corroboration.txt](corroboration.txt) records 34 observations, all confirmed: the pack's
manifest and content identity; the checkout, branch, HEAD and remote; that the dispatch
ran task 2 only; the accepted TASK-001 predecessor; the presence and hashes of all
evidence files; the exit codes and results recorded in every log; that the receipt's 31
check IDs equal the task's and each is accounted for in the evidence record; the 135
policy IDs; and the snapshot, recomputed.

On that record, and on nothing else, `evidence_verified_by_host` was set to true and the
blocker `UNVERIFIED_RECEIPT` was cleared in [receipt.json](receipt.json), and
`corroboration.txt` was added to its evidence files. Every other value is unchanged.

This is the agent's self-corroboration with deterministic tools. **It is not an
independent human or model review, and the owner has not reviewed TASK-002.**

## What was evaluated

| Item | Value |
|---|---|
| Receipt | [receipt.json](receipt.json): the coding agent's declaration, corroborated as above |
| Acceptance program | `/mnt/F/Nexees_LCL_Implementation_Pack_v0.5.3/accept_task.lcl.txt` |
| Pack content identity | `bd5e8fce26f542c44a891e4340da3b2a84f79ab368b5cb868beb48126c2db3e1` |
| Engine | `lcl 0.9.1`, canonical Core 0.3.0 (identity `7c8d46931933fa28…1aff`) |
| Worktree snapshot | `0756d8cf5a8138a81fe1926dcef2aa32f3846fdc978770df3c3be70ae2dd051b`, over 335 files |
| Snapshot listing | [snapshot_files.sha256](snapshot_files.sha256) |
| Evaluated on | 2026-10-03 |

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
| `output.receipt` | Published for `TASK-002`, with this snapshot and content identity |
| Engine record | [evaluation.record.json](evaluation.record.json), SHA-256 `c5fadc8310565f2a73c1f16209987b9a1a086c99919cd6cb101c010eb243bd13` |

- **Language acceptance: yes.**
- **Verification of the task evidence:** done by the primary session's local tools, as
  CA-05 permits; no other party reviewed the work.

### Before corroboration

The same receipt, with `evidence_verified_by_host: false` and the blocker
`UNVERIFIED_RECEIPT`, was refused before anything ran (exit code 1) on exactly two gates,
`validate.host_verified` and `validate.no_blocker`; every other gate passed. That record
was not kept. Restoring those two values in `receipt.json` reproduces it.

## Final verification

[final_verification.txt](final_verification.txt) was produced after cleanup against the
frozen worktree:

- `lcl check`, `lcl validate` and `lcl run` of the architecture: exit 0, `status.succeeded`.
- Cross-check of the architecture against the repository and the pack: all checks pass.
- The charter's `lcl` checks still pass.
- The pack is intact, and no file in it is newer than its manifest.
- Only `FILE_TREE.txt` and `README.md` differ from commit `284f97a`; every new file is
  under `docs/architecture/` or `docs/evidence/TASK-002/`; nothing is staged.
- 278 placeholder files still carry their marker line; the PNG files are unchanged; the
  checkout holds no temporary file; `FILE_TREE.txt` matches the actual tree.
- The snapshot was the same before and after the verification.

## To reproduce

1. Confirm the pack:

   ```bash
   sha256sum /mnt/F/Nexees_LCL_Implementation_Pack_v0.5.3/MANIFEST.json
   # 8d9d0080ac13a3fb17cc70603b77eb3b11abcd3fb6d2fad515cfac136fcf07e4
   ```

2. Recompute the snapshot. It prints `0756d8cf…051b` for as long as no file outside this
   folder has changed:

   ```bash
   cd /mnt/F/Nexees && find . -path ./.git -prune -o -path ./docs/evidence/TASK-002/receipt -prune -o -type f -print0 | LC_ALL=C sort -z | xargs -0 sha256sum | sha256sum
   ```

3. Re-run the checks:

   ```bash
   cd /mnt/F/Nexees
   SPEC="--spec /mnt/F/LCL/canonical/LCL_Core_0.1.0 --project-spec /mnt/F/LCL/canonical/LCL_Core_0.3.0"
   lcl check $SPEC docs/architecture/architecture.lcl.txt
   lcl validate $SPEC docs/architecture/architecture.lcl.txt
   lcl run $SPEC docs/architecture/architecture.lcl.txt
   python3 -B docs/evidence/TASK-002/check_architecture.py /mnt/F/Nexees_LCL_Implementation_Pack_v0.5.3
   ```

4. Evaluate the receipt. The expected result is `status.succeeded`, exit code 0:

   ```bash
   python3 - <<'EOF'
   import json, subprocess
   inputs = json.load(open('/mnt/F/Nexees/docs/evidence/TASK-002/receipt/receipt.json'))['inputs']
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
expected: this receipt describes the state at which TASK-002 was accepted, which is the
state committed together with it. Committing does not change the snapshot, because
`.git/` is outside it.
