# TASK-001 receipt result

**TASK-001 is accepted.** On 2026-10-03 the owner stated that he had verified the task.
With his corroboration recorded, the pack's acceptance program accepts the receipt:
`status.succeeded`, exit code 0.

This folder was written after everything else in the task, because the receipt has to
contain a snapshot of the rest of the worktree. It is the only folder the snapshot leaves
out, so recording the corroboration here did not change the state the owner verified.
[../TASK-001_EVIDENCE.md](../TASK-001_EVIDENCE.md) is inside the snapshot and was left
exactly as he verified it; its Status section describes the state before corroboration.

## Owner's corroboration

The owner (Aivars) wrote to the coding agent on 2026-10-03:

```
Task 1 is complete, I have Verified, u can commit and Sync, then stop.
```

On this statement, and on nothing else, `evidence_verified_by_host` was set to true and
the blocker `UNVERIFIED_RECEIPT` was cleared in [receipt.json](receipt.json). Every other
value is unchanged. The agent did not observe which checks the owner made.

## What was evaluated

| Item | Value |
|---|---|
| Receipt | [receipt.json](receipt.json): the coding agent's declaration, corroborated by the owner |
| Acceptance program | `/mnt/F/Nexees_LCL_Implementation_Pack_v0.5.2/accept_task.lcl.txt` |
| Pack content identity | `e288da0e1b727e1b646ad1b03155f5a067e4d932ae026686e904d1ceb66d4672` |
| Engine | `lcl 0.9.1`, canonical Core 0.3.0 |
| Worktree snapshot | `d50c39f776c797efa771830b4f462a4ed77e43de627f6486079f3f081bd78556`, over 304 files |
| Snapshot listing | [snapshot_files.sha256](snapshot_files.sha256) |
| Evaluated on | 2026-10-03 |

The snapshot is the SHA-256 of the listing: one `sha256sum` line for every file under
`/mnt/F/Nexees/` except `.git/` and this folder, with paths sorted under `LC_ALL=C`. It was
recomputed immediately before and after the corroboration was recorded and was the same
value both times.

## Result

| | Value |
|---|---|
| Purpose | `implementation_review` |
| Exit code | 0 |
| Terminal status | `status.succeeded` |
| Diagnostics | None |
| `VERIFY` checks | All six TRUE: `verify.selected_receipt`, `verify.snapshot_shape`, `verify.ordered_predecessors`, `verify.required_checks`, `verify.instruction_delivery`, `verify.receipt_delivery` |
| `output.receipt` | Published for `TASK-001` |
| Engine record | [evaluation.record.json](evaluation.record.json), SHA-256 `bbda5d839f7713bbd43a0fb3c0c299043abe8f7b055e6e3ce8c110fde3ba07e7` |

- **Language acceptance: yes.**
- **Verification of the task evidence:** the checks were run and recorded by the coding
  agent, and the owner has stated that he verified them. No other party reviewed the work.

### Before corroboration

The same receipt, with `evidence_verified_by_host: false` and the blocker
`UNVERIFIED_RECEIPT`, was refused before anything ran (exit code 1) on exactly two gates:
`validate.host_verified` and `validate.no_blocker`. That receipt and its engine record
are kept outside the repository at
`/mnt/F/Nexees_LCL_Implementation_Pack_v0.5.2_native_results/task_001_receipt_before_corroboration/`
(SHA-256 `4de3d44f…1369` and `ebf7c402…030a`).

### Synthetic projection

`synthetic_projection.receipt.json` and its record were produced before corroboration,
under the pack's synthetic purpose `language_test`, to show that no other gate stood in
the way. They are not acceptance and are not evidence of completion. The acceptance is the
result above.

## Final verification

[final_verification.txt](final_verification.txt) was produced after cleanup, against the
frozen worktree:

- `lcl check`, `lcl validate` and `lcl run` of the charter: exit 0, both `VERIFY` checks TRUE.
- Cross-check of the charter against the pack: all checks pass.
- Only `FILE_TREE.txt`, `README.md` and `docs/manuals/NEXEES_USER_MANUAL.md` differ from
  commit `0d6d3e5`; every new file is under `docs/charter/` or `docs/evidence/TASK-001/`.
- 278 placeholder files still carry their marker line; the PNG files are unchanged.
- The snapshot was the same before and after the verification.

## To reproduce

1. Confirm the pack:

   ```bash
   sha256sum /mnt/F/Nexees_LCL_Implementation_Pack_v0.5.2/MANIFEST.json
   # e55ec6130b3916bd249cb7ad6abc5277e2d1acfdd0586226c4a9a14e6eef639a
   ```

2. Recompute the snapshot. It must print `d50c39f7…8556` for as long as no file outside
   this folder has changed:

   ```bash
   cd /mnt/F/Nexees && find . -path ./.git -prune -o -path ./docs/evidence/TASK-001/receipt -prune -o -type f -print0 | LC_ALL=C sort -z | xargs -0 sha256sum | sha256sum
   ```

3. Re-run the checks:

   ```bash
   cd /mnt/F/Nexees
   SPEC="--spec /mnt/F/LCL/canonical/LCL_Core_0.1.0 --project-spec /mnt/F/LCL/canonical/LCL_Core_0.3.0"
   lcl check $SPEC docs/charter/charter.lcl.txt
   lcl validate $SPEC docs/charter/charter.lcl.txt
   lcl run $SPEC docs/charter/charter.lcl.txt
   python3 -B docs/evidence/TASK-001/check_charter_against_pack.py /mnt/F/Nexees_LCL_Implementation_Pack_v0.5.2
   ```

4. Evaluate the receipt. The expected result is `status.succeeded`, exit code 0:

   ```bash
   python3 - <<'EOF'
   import json, subprocess
   inputs = json.load(open('/mnt/F/Nexees/docs/evidence/TASK-001/receipt/receipt.json'))['inputs']
   def literal(v):
       if isinstance(v, bool): return 'TRUE' if v else 'FALSE'
       if isinstance(v, (str, int)): return json.dumps(v, ensure_ascii=False)
       return '[' + ', '.join(literal(i) for i in v) + ']'
   argv = ['lcl', 'run', '--spec', '/mnt/F/LCL/canonical/LCL_Core_0.1.0',
           '--project-spec', '/mnt/F/LCL/canonical/LCL_Core_0.3.0',
           '/mnt/F/Nexees_LCL_Implementation_Pack_v0.5.2/accept_task.lcl.txt']
   for key, value in inputs.items():
       argv += ['--input', f'input.{key}={literal(value)}']
   raise SystemExit(subprocess.run(argv).returncode)
   EOF
   ```

A later task that changes any file outside this folder changes the snapshot. That is
expected: this receipt describes the state at which TASK-001 was accepted, which is the
state committed together with it. Committing does not change the snapshot, because `.git/`
is outside it.
