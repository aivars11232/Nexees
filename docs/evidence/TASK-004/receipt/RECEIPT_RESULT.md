# TASK-004 receipt result

**TASK-004 is accepted.** The pack's acceptance program accepts the corroborated
receipt: `status.succeeded`, exit code 0.

This folder was written after everything else in the task, because the receipt has to
contain a snapshot of the rest of the worktree. It is the only folder the snapshot
leaves out, so writing it did not change the state that was verified.

## Corroboration

Under CA-05 of the continuation profile, routine corroboration is done by the primary
session's own local tools. [corroboration.txt](corroboration.txt) records 30
observations, all confirmed:

- the pack's manifest and content identity;
- the checkout, branch, HEAD and remote, with the two owner-requested TASK-003 receipt
  commits on top of `2eaa199`;
- that the dispatch ran task 4 only;
- the accepted TASK-001, TASK-002 and TASK-003 predecessors, unchanged (TASK-003 as the
  owner decided it);
- the presence and hashes of the 13 evidence files then listed (`corroboration.txt`, which
  that step wrote, was added afterwards, making 14);
- that no file of the checkout is ignored by Git, so the commit carries every evidence
  file;
- the exit codes and results recorded in every log: the threat model's LCL checks and
  cross-check, the negative tests, the regression, the native suite, the security scan,
  the preflight, the reading record, the foundation facts and the final verification;
- that the receipt's 33 check IDs equal the task's, each accounted for in the evidence
  record;
- the 135 policy IDs;
- the renderer decision against the owner's recorded answer;
- the design-only status of the remote requirements;
- the snapshot, recomputed.

On that record, and on nothing else, `evidence_verified_by_host` was set to true and the
blocker `UNVERIFIED_RECEIPT` was cleared in [receipt.json](receipt.json), and
`corroboration.txt` was added to its evidence files. Every other value is unchanged.

This is the agent's self-corroboration with deterministic tools. **It is not an
independent human or model review, and the owner has not reviewed TASK-004.**

## What was evaluated

| Item | Value |
|---|---|
| Receipt | [receipt.json](receipt.json): the coding agent's declaration, corroborated as above |
| Acceptance program | `/mnt/F/Nexees_LCL_Implementation_Pack_v0.5.3/accept_task.lcl.txt` |
| Pack content identity | `bd5e8fce26f542c44a891e4340da3b2a84f79ab368b5cb868beb48126c2db3e1` |
| Engine | `lcl 0.9.1`, canonical Core 0.3.0 (identity `7c8d46931933fa28…1aff`) |
| Worktree snapshot | `3e87a9a7f540b1724d5d2d8a297fe66ac0e3458c85793ea1e7f5ea0b474ee6aa`, over 444 files |
| Snapshot listing | [snapshot_files.sha256](snapshot_files.sha256) |
| Predecessors | TASK-001, TASK-002 and TASK-003 accepted; TASK-003's receipt corrected and decided by the owner in `a590eab` and `bccc9b1` (evidence record, section 2) |
| Renderer decision | `electron_permitted`, approved: the owner's answer to DESKTOP-RENDERER-01, recorded by TASK-003 |
| Remote observations | All eight `unverified`: they describe the shipped product and are required only for tasks 70 and 73 to 75; TASK-004's remote requirements are design only |
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
| `output.receipt` | Published for `TASK-004`, with this snapshot and content identity |
| Engine record | [evaluation.record.json](evaluation.record.json), SHA-256 `2b7406b82bf63a524943a87c60cf322ec17291bfaf800efd23676e0097bbc83f` |

- **Language acceptance: yes.**
- **Verification of the task evidence:** done by the primary session's local tools, as
  CA-05 permits; no other party reviewed the work.

### Before corroboration

The same receipt, with `evidence_verified_by_host: false` and the blocker
`UNVERIFIED_RECEIPT`, was refused before anything ran (exit code 1). It failed on exactly
two gates, `validate.host_verified` and `validate.no_blocker`; every other `VALIDATE` gate
passed. That record was not kept. Restoring those two values in `receipt.json` reproduces
it.

## Final verification

[final_verification.txt](final_verification.txt) was produced after cleanup against the
frozen worktree:

- `lcl check`, `lcl validate` and `lcl run` of the threat model, the architecture, the
  dependency inventory and the charter: exit 0, `status.succeeded`.
- The cross-checks pass: `check_threat_model.py`, TASK-002's `check_architecture.py`,
  TASK-003's `check_dependencies.py` and TASK-001's `check_charter_against_pack.py`, the
  last against the preserved 0.5.2 baseline.
- The pack is intact, with no file newer than its manifest. The LCL repository is clean.
- Since TASK-004 began, no file in the LCL SDK or the home caches was written. No task
  process is running.
- Only `FILE_TREE.txt`, `README.md` and `SECURITY.md` differ from HEAD. Every new file is
  under `docs/security/` or `docs/evidence/TASK-004/`. Nothing is staged.
- No file is ignored by Git, and the files a commit would hold equal the snapshot listing.
- 278 placeholder files still carry their marker line. The PNG files are unchanged, the
  checkout holds no temporary file, and `FILE_TREE.txt` matches the actual tree.
- The snapshot was the same before and after the verification.

## To reproduce

1. Confirm the pack:

   ```bash
   sha256sum /mnt/F/Nexees_LCL_Implementation_Pack_v0.5.3/MANIFEST.json
   # 8d9d0080ac13a3fb17cc70603b77eb3b11abcd3fb6d2fad515cfac136fcf07e4
   ```

2. Recompute the snapshot. It prints `3e87a9a7…e6aa` for as long as no file outside this
   folder has changed:

   ```bash
   cd /mnt/F/Nexees && find . -path ./.git -prune -o -path ./docs/evidence/TASK-004/receipt -prune -o -type f -print0 | LC_ALL=C sort -z | xargs -0 sha256sum | sha256sum
   ```

3. Re-run the checks:

   ```bash
   cd /mnt/F/Nexees
   SPEC="--spec /mnt/F/LCL/canonical/LCL_Core_0.1.0 --project-spec /mnt/F/LCL/canonical/LCL_Core_0.3.0"
   lcl check $SPEC docs/security/security.lcl.txt
   lcl validate $SPEC docs/security/security.lcl.txt
   lcl run $SPEC docs/security/security.lcl.txt
   python3 -B docs/evidence/TASK-004/check_threat_model.py /mnt/F/Nexees_LCL_Implementation_Pack_v0.5.3
   ```

4. Evaluate the receipt. The expected result is `status.succeeded`, exit code 0:

   ```bash
   python3 - <<'EOF'
   import json, subprocess
   inputs = json.load(open('/mnt/F/Nexees/docs/evidence/TASK-004/receipt/receipt.json'))['inputs']
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
expected: this receipt describes the state at which TASK-004 was accepted, which is the
state committed together with it. Committing does not change the snapshot, because
`.git/` is outside it.
