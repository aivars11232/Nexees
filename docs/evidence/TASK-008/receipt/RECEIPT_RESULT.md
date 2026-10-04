# TASK-008 receipt result

**TASK-008 is accepted.** The pack's acceptance program accepts the corroborated
receipt: `status.succeeded`, exit code 0.

This folder was written after everything else in the task, because the receipt has to
contain a snapshot of the rest of the worktree. The snapshot leaves this folder out, so
writing it did not change the state that was verified.

## Corroboration

Under CA-05 of the continuation profile, routine corroboration is done by the primary
session's own local tools. [corroboration.txt](corroboration.txt) records 40
observations, all confirmed:

- the pack's manifest and content identity;
- the checkout, branch and remote, with nothing staged. HEAD is `1c85864`, TASK-007's
  follow-up commit, unchanged during the task;
- that the dispatch ran task 8 only;
- the accepted TASK-001 to TASK-007 predecessors, each with its evidence unchanged. The charter
  is unchanged since TASK-001, and `docs/security` and the manual sources are unchanged by the
  task;
- that the files the task changed and removed are exactly those the evidence record lists;
- the presence and hashes of the 13 evidence files then listed (`corroboration.txt`, which
  that step wrote, was added afterwards, making 14);
- that the only ignored files are the owner's IDE output, so the commit carries every
  evidence file;
- the results recorded in every log:
  - the check runner, with the Android builds of the three crates;
  - the negative tests, each test-stage case caught by named failing tests, and the raised
    record version by the state store's own pin;
  - the regression: 23 commands exit 0;
  - the native suite, the security scan, the IDE record, the preflight and the reading
    record;
  - the final verification, with every exit 0;
- that the receipt's 26 check IDs equal the task's, each accounted for in the evidence
  record;
- the 135 policy IDs;
- the renderer decision against the owner's recorded answer;
- that each of the three assigned remote requirements and three scenarios has its row in the
  evidence record, with what TASK-008 closed and its runtime owners;
- the closure record (next section);
- the snapshot, recomputed.

On that record, and on nothing else, `evidence_verified_by_host` was set to true and the
blocker `UNVERIFIED_RECEIPT` was cleared in [receipt.json](receipt.json), and
`corroboration.txt` was added to its evidence files. Every other value is unchanged.

This is the agent's self-corroboration with deterministic tools. **It is not an
independent human or model review, and the owner has not reviewed TASK-008.**

## Closure record

[receipt.json](receipt.json) carries the closure record `closure`, schema
`nexees-task-closure` version 1. The runner's own validator accepts it, and so does the
evidence stage of `run_checks.py`.

| Part | Recorded |
|---|---|
| Readability | The agent's own review against the final code, with the changes it led to (evidence record, section 9). No independent review. |
| Cleanup | [logs/cleanup_record.json](../logs/cleanup_record.json), applied with an empty manifest. Nothing was removed or refused. 15 untracked files were left in place: the five source files of `core/state`, the evidence record and nine logs, each with a hash that equals the snapshot listing. 421 ignored files were also left in place, all the owner's IDE output (`target/` and TASK-003's `.gradle/`). |
| Manual impact | `not_applicable`: the state store is internal to the hosts, and both manual sources are unchanged. |
| Final verification | [final_verification.txt](final_verification.txt), `passed`, on this receipt's snapshot. |

## Order of the close

Two runs were repeated before the close began:
- **The negative suite.** Its first run caught the seeded raise of a record version only
  through the domain's own pin, so the state store's pin never ran. The case was changed to
  move the domain's pin along, and the whole suite ran again at 15:00:11 (evidence record,
  section 7).
- **The regression.** A sentence in DEP-SQLITE's verification text was corrected afterwards, so
  the regression ran again on the final records at 15:08:46.

The evidence record and the last logs were then made final:
- the IDE record, at 15:09:27;
- the security scan of every file the task changed, at 15:09:45;
- [logs/run_checks.txt](../logs/run_checks.txt), at 15:10:07.

The scoped cleanup ran after them, at 15:10:48. The final verification followed at 15:10:59.
The unverified receipt was evaluated at 15:11:25, corroboration ran at 15:11:30 and the
corroborated receipt was accepted at 15:11:37.

## What was evaluated

| Item | Value |
|---|---|
| Receipt | [receipt.json](receipt.json): the coding agent's declaration, corroborated as above |
| Acceptance program | `/mnt/F/Nexees_LCL_Implementation_Pack_v0.5.3/accept_task.lcl.txt` |
| Pack content identity | `bd5e8fce26f542c44a891e4340da3b2a84f79ab368b5cb868beb48126c2db3e1` |
| Engine | `lcl 0.9.1`, canonical Core 0.3.0 (identity `7c8d46931933fa28…1aff`) |
| Worktree snapshot | `d976dbb5ae89678e18cb568d8e843d8d36987d27f3f5068104fffbd4a9fe4151`, over 545 files |
| Snapshot listing | [snapshot_files.sha256](snapshot_files.sha256) |
| Predecessors | TASK-001 to TASK-007 accepted, with their engine records intact (evidence record, section 3) |
| Renderer decision | `electron_permitted`, approved: the owner's answer to DESKTOP-RENDERER-01, recorded by TASK-003 |
| Remote observations | All eight `unverified`: they describe the shipped product and are required only for tasks 70 and 73 to 75. TASK-008's remote requirements and scenarios are closed for their storage part only (evidence record, section 8). |
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
| `output.receipt` | Published for `TASK-008`, with this snapshot and content identity |
| Engine record | [evaluation.record.json](evaluation.record.json), SHA-256 `a8c8b44db09aee9f0f8f88e029d8bb92ad54167d650878864ed1a6b93f68b2a2` |

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
  domain, 30 protocol and 28 state tests, the 26 tooling tests and the `lcl` check, validate
  and run of the four LCL projects. `nexees-domain`, `nexees-protocol` and `nexees-state`
  build for `aarch64-linux-android` and `x86_64-linux-android`.
- **Predecessor cross-checks:** all four pass: TASK-001's against the preserved 0.5.2
  baseline, then TASK-002's, TASK-003's and TASK-004's.
- **Pack, LCL and SDK:** the pack is intact, with no file newer than its manifest. The LCL
  repository is clean at `fa1592b`. No LCL SDK file was written since TASK-008 began.
- **Home caches:** 52 files were written since then, all by the owner's IDE
  (logs/ide_activity.txt):
  - 37 in `~/.cargo`, from rust-analyzer: its index refresh, the eight crates it downloaded and
    the sources it unpacked;
  - 15 in `~/.gradle`: the Gradle daemons' logs and registry, and the cache clean-up of the
    idle Gradle 8.9 daemon as it stopped itself.

  No test store is left in the temporary folder, and no task process is running.
- **Git:**
  - against HEAD `1c85864`: exactly the 8 expected files are modified, the three implemented
    placeholders are removed, and the five new source files are added;
  - nothing is staged;
  - the only ignored files are the owner's IDE output.
- **Snapshot listing:** it agrees with `sha256sum` over the same 545 files.
- **Layout:** 261 placeholder files still carry their marker line. That is 264 before, less
  the three implemented ones. The PNG files are unchanged, and no temporary file or build
  output is among the files a commit holds.
- **Stability:** the snapshot was the same before and after the verification.

## To reproduce

1. Confirm the pack:

   ```bash
   sha256sum /mnt/F/Nexees_LCL_Implementation_Pack_v0.5.3/MANIFEST.json
   # 8d9d0080ac13a3fb17cc70603b77eb3b11abcd3fb6d2fad515cfac136fcf07e4
   ```

2. Recompute the snapshot. It prints `d976dbb5…4151` for as long as no committable file
   outside this folder has changed:

   ```bash
   cd /mnt/F/Nexees && git ls-files -z --cached --others --exclude-standard -- . ':!docs/evidence/TASK-008/receipt' | LC_ALL=C sort -zu | while IFS= read -r -d '' f; do [ -f "$f" ] && printf './%s\0' "$f"; done | xargs -0 sha256sum | sha256sum
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
   inputs = json.load(open('/mnt/F/Nexees/docs/evidence/TASK-008/receipt/receipt.json'))['inputs']
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
That is expected: this receipt describes the state at which TASK-008 was accepted, which is
the state committed together with it. Committing does not change the snapshot.
