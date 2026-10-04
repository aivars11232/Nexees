# TASK-009 receipt result

**TASK-009 is accepted.** The pack's acceptance program accepts the corroborated
receipt: `status.succeeded`, exit code 0.

This folder was written after everything else in the task, because the receipt has to
contain a snapshot of the rest of the worktree. The snapshot leaves this folder out, so
writing it did not change the state that was verified.

## This is the second close

TASK-009 was first closed at 17:12:46 on 2026-10-04: the acceptance program accepted a
receipt for the snapshot `04d9a60b76693c71449434d19231cdd24e3c0ea470b033e4f16e6eb597c42b0b`
over 575 files, with the engine record `82886e9d…c4c9` and 46 corroborated observations.
That close was reported to the owner and was not committed, because the owner's order puts a
recheck between the report and the commit.

The recheck found the task unfinished: the status bar that shows the host was never visible,
the security records' assignments to TASK-009 had not been read, and the window fetched a file
from Google's servers at each start (evidence record, section 11). The first receipt described
a state that was not what it claimed, so its folder was removed, the work was completed, and
the task was closed again. Nothing of the first close reached Git. The one record of it that
is kept is the cleanup record of that close, [logs/cleanup_first_close.json](../logs/cleanup_first_close.json).

Everything below describes the second close.

## Corroboration

Under CA-05 of the continuation profile, routine corroboration is done by the primary
session's own local tools. [corroboration.txt](corroboration.txt) records 48
observations, all confirmed:

- the pack's manifest and content identity;
- the checkout, branch and remote, with nothing staged. HEAD is `d00bb83`, the owner-directed
  correction made after the TASK-008 commit, which changed `core/domain/task.rs` only and was
  unchanged during the task;
- that the dispatch ran task 9 only;
- the accepted TASK-001 to TASK-008 predecessors, each with its evidence unchanged. The charter
  is unchanged since TASK-001. In `docs/security` the task changed the threats, the test gaps,
  the risks and decisions and their counts, and no invariant. The user manual is the one manual
  source it changed;
- that the files the task added, changed and removed are exactly those the evidence record
  lists: 20 new, 22 changed and 9 removed placeholders;
- the presence and hashes of the 19 evidence files then listed (`corroboration.txt`, which
  that step wrote, was added afterwards, making 20);
- that the only ignored files are the owner's IDE output and the editor's link, so the commit
  carries every evidence file;
- the results recorded in every log:
  - the check runner, with the Android builds of the three core crates;
  - the negative tests: 23 seeded defects caught, the 14 test-stage cases by named failing
    tests, and the two controls;
  - the five defects seeded against the correction `d00bb83`, each caught by its named test;
  - the regression: 24 commands exit 0;
  - the Desktop build and installation, with two fuses set and the rebuilt native modules
    loading in the installed Electron, and the 30 checks of the installed application;
  - the four hardening trials, each as the records state it;
  - the OSV query, which covers exactly the package set the npm review records;
  - the native suite, the security scan, the IDE record, the preflight and the reading
    record;
  - the final verification, with every exit 0 and the installed application's 30 checks;
- that the receipt's 30 check IDs equal the task's, each accounted for in the evidence
  record;
- the 135 policy IDs;
- the renderer decision against the owner's recorded answer;
- that each of the seven assigned remote requirements and five scenarios has its row in the
  evidence record, with what TASK-009 closed and its runtime owners;
- the closure record (next section);
- the snapshot, recomputed.

On that record, and on nothing else, `evidence_verified_by_host` was set to true and the
blocker `UNVERIFIED_RECEIPT` was cleared in [receipt.json](receipt.json), and
`corroboration.txt` was added to its evidence files. Every other value is unchanged.

This is the agent's self-corroboration with deterministic tools. **It is not an
independent human or model review, and the owner has not reviewed TASK-009.** The first
close passed the same kind of corroboration: it confirms that the logs say what the record
says, not that the record asks the right questions.

## Closure record

[receipt.json](receipt.json) carries the closure record `closure`, schema
`nexees-task-closure` version 1. The runner's own validator accepts it, and so does the
evidence stage of `run_checks.py`.

| Part | Recorded |
|---|---|
| Readability | The agent's own review against the final code, with the changes it led to, the recheck's among them (evidence record, section 9). No independent review. |
| Cleanup | The task left one artifact in the checkout: the check runner's bytecode, `scripts/test/__pycache__/run_checks.cpython-314.pyc`, and its folder, written by a command that ran without `-B`. The cleanup of the first close removed both ([logs/cleanup_first_close.json](../logs/cleanup_first_close.json)). The cleanup of this close, [logs/cleanup_record.json](../logs/cleanup_record.json), applied with the same manifest, found them gone and removed and refused nothing. 36 untracked files were left in place: the 20 new files, the evidence record and 15 logs, each with a hash that equals the snapshot listing. 572 ignored files were also left in place: the owner's IDE output (`target/` and TASK-003's `.gradle/`) and the one link through which the owner's editor finds the window's packages. |
| Manual impact | `updated`: `docs/manuals/NEXEES_USER_MANUAL.md` section 2 describes the Desktop window and the Nexees host. The LCL manual is unchanged. |
| Final verification | [final_verification.txt](final_verification.txt), `passed`, on this receipt's snapshot. |

## Order of the close

The recheck's changes were made and tried between 17:26 and 18:45. Then, in this order:

- the hardening trials, at 18:46:02. That was their third run. In the first, the second trial
  judged Theia's single-process mode by the end-to-end test's check that the host holds no
  network socket, which an earlier manual run had failed and that run passed; the trial was
  changed to measure what a process the backend starts inherits. The second run counted that
  process's own three standard descriptors among the inherited ones;
- the Desktop build from scratch, at 18:50:56, and the end-to-end test on it, at 18:51:51;
- the evidence record;
- the negative tests, from 18:56:50 to 19:01:42, and the regression, at 19:01:54;
- the IDE record, at 19:03:39. Its first run of this close found two things the owner's VS Code
  had done since the first close: a new Gradle daemon with two builds of TASK-003's prototype,
  and a download by its TypeScript service into its own type cache. Both were attributed from
  their own logs, and the evidence record says so;
- the security scan of every file the task changed, at 19:03:52;
- [logs/run_checks.txt](../logs/run_checks.txt), at 19:04:09.

The scoped cleanup ran after them and wrote its record at 19:04:34. The final verification
followed at 19:04:45 and ended at 19:06:33. The unverified receipt was evaluated and the
corroboration run at 19:06:40, and the corroborated receipt was accepted at 19:06:51.

## What was evaluated

| Item | Value |
|---|---|
| Receipt | [receipt.json](receipt.json): the coding agent's declaration, corroborated as above |
| Acceptance program | `/mnt/F/Nexees_LCL_Implementation_Pack_v0.5.3/accept_task.lcl.txt` |
| Pack content identity | `bd5e8fce26f542c44a891e4340da3b2a84f79ab368b5cb868beb48126c2db3e1` |
| Engine | `lcl 0.9.1`, canonical Core 0.3.0 (identity `7c8d46931933fa28…1aff`) |
| Worktree snapshot | `9876495b3aed4ad9949e7c20a6bc8abb119e95f0346346b79d4b1dccf3e58e4e`, over 579 files |
| Snapshot listing | [snapshot_files.sha256](snapshot_files.sha256) |
| Predecessors | TASK-001 to TASK-008 accepted, with their engine records intact (evidence record, section 3) |
| Renderer decision | `electron_permitted`, approved: the owner's answer to DESKTOP-RENDERER-01, recorded by TASK-003 |
| Remote observations | All eight `unverified`: they describe the shipped product and are required only for tasks 70 and 73 to 75. TASK-009's remote requirements and scenarios are closed for the Desktop shell's part only (evidence record, section 8). |
| Evaluated on | 2026-10-04 |

The snapshot is what a commit of the task holds (CONVENTIONS.md section 10). It is the
SHA-256 of one `sha256sum` line per file Git would commit, tracked or untracked but not
ignored, outside this folder, with paths sorted under `LC_ALL=C`. The owner's IDE output, the
editor's link and the Desktop build, which lives outside the checkout, are not part of it.

It is the same value before and after the final verification and at corroboration.

## Result

| | Value |
|---|---|
| Purpose | `implementation_review` |
| Exit code | 0 |
| Terminal status | `status.succeeded` |
| Diagnostics | None |
| `VERIFY` checks | All six TRUE: `verify.selected_receipt`, `verify.snapshot_shape`, `verify.ordered_predecessors`, `verify.required_checks`, `verify.instruction_delivery`, `verify.receipt_delivery` |
| `output.receipt` | Published for `TASK-009`, with this snapshot and content identity |
| Engine record | [evaluation.record.json](evaluation.record.json), SHA-256 `d65fa84a9fd910b3c394cacb8727701a1d8cbe1e1774e4f5b0154cd8b5e47245` |

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

- **The check runner:** `scripts/test/run_checks.py` passes all nine stages. That covers the
  93 domain, 30 protocol, 29 state, 12 platform and 9 host tests, the 2 doctests and the 35
  tooling tests. It also covers the strict TypeScript compile, the npm lockfile gate, and the
  `lcl` check, validate and run of the four LCL projects. `nexees-domain`, `nexees-protocol`
  and `nexees-state` build for `aarch64-linux-android` and `x86_64-linux-android`.
- **The Desktop application:** built offline from the final sources, installed into a
  disposable prefix with its two fuses set, and tested end to end: all 30 checks of the
  installed application pass.
- **Predecessor cross-checks:** all four pass: TASK-001's against the preserved 0.5.2
  baseline, then TASK-002's, TASK-003's and TASK-004's.
- **Pack, LCL and SDK:** the pack is intact, with no file newer than its manifest. The LCL
  repository is clean at `fa1592b`. No LCL SDK file was written since TASK-009 began.
- **Home caches:** 40 files were written since then, all by the owner's IDE
  (logs/ide_activity.txt):
  - 9 in `~/.cargo`, from rust-analyzer: its index refresh, the libc crate it downloaded and
    the sources it unpacked;
  - 19 in `~/.gradle`, from the Gradle extension's daemons: one stopping itself after 180 idle
    minutes, and a new one that built TASK-003's prototype twice;
  - 6 in `~/.npm` and 6 in `~/.cache/typescript`, from VS Code's TypeScript service fetching
    its registry of type definitions.

  The application itself never ran against the owner's home. No test folder is left in the
  temporary folder, no task process is running, and the trial installation is gone.
- **Git:**
  - against HEAD `d00bb83`: exactly the 22 expected files are modified, the nine implemented
    placeholders are removed, and the 20 new files and this evidence folder are added;
  - nothing is staged;
  - the only ignored files are the owner's IDE output and the editor's link, which points at
    the Desktop build folder.
- **Snapshot listing:** it agrees with `sha256sum` over the same 579 files.
- **Layout:** 252 placeholder files still carry their marker line. That is 261 before, less
  the nine implemented ones. The PNG files are unchanged, and no temporary file or build
  output is among the files a commit holds.
- **Stability:** the snapshot was the same before and after the verification.

## To reproduce

1. Confirm the pack:

   ```bash
   sha256sum /mnt/F/Nexees_LCL_Implementation_Pack_v0.5.3/MANIFEST.json
   # 8d9d0080ac13a3fb17cc70603b77eb3b11abcd3fb6d2fad515cfac136fcf07e4
   ```

2. Recompute the snapshot. It prints `9876495b…8e4e` for as long as no committable file
   outside this folder has changed:

   ```bash
   cd /mnt/F/Nexees && git ls-files -z --cached --others --exclude-standard -- . ':!docs/evidence/TASK-009/receipt' | LC_ALL=C sort -zu | while IFS= read -r -d '' f; do [ -f "$f" ] && printf './%s\0' "$f"; done | xargs -0 sha256sum | sha256sum
   ```

3. Re-run the checks, on this machine's isolated toolchain and caches. The evidence stage
   also validates this receipt's closure record:

   ```bash
   cd /mnt/F/Nexees
   source /mnt/F/Nexees-toolchains/env.sh
   export PATH="$PATH:$HOME/.cargo/bin" NEXEES_LCL_CORE_01=/mnt/F/LCL/canonical/LCL_Core_0.1.0 NEXEES_LCL_CORE_03=/mnt/F/LCL/canonical/LCL_Core_0.3.0
   export NEXEES_NPM_CACHE=/mnt/F/Nexees-toolchains/npm-cache NEXEES_BUILD_HOME=/mnt/F/Nexees-toolchains/home
   python3 -B scripts/test/run_checks.py --target-dir /mnt/F/Nexees-toolchains/target/nexees
   ```

4. Build, install and test the Desktop application. The prefix is disposable:

   ```bash
   export NEXEES_ELECTRON_CACHE=/mnt/F/Nexees-toolchains/electron-cache CARGO_TARGET_DIR=/mnt/F/Nexees-toolchains/target/nexees
   python3 -B scripts/build/build_desktop.py build --build-dir /mnt/F/Nexees-toolchains/target/nexees/desktop --electron-headers 42.8.1
   python3 -B scripts/build/build_desktop.py install --build-dir /mnt/F/Nexees-toolchains/target/nexees/desktop --prefix /mnt/F/Nexees-toolchains/target/nexees/desktop-install
   node tests/e2e/desktop/local_application.test.mjs /mnt/F/Nexees-toolchains/target/nexees/desktop-install
   ```

5. Evaluate the receipt. The expected result is `status.succeeded`, exit code 0:

   ```bash
   python3 - <<'EOF'
   import json, subprocess
   inputs = json.load(open('/mnt/F/Nexees/docs/evidence/TASK-009/receipt/receipt.json'))['inputs']
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
That is expected: this receipt describes the state at which TASK-009 was accepted, which is
the state committed together with it. Committing does not change the snapshot.
