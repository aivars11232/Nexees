# TASK-005 evidence: Create repository scaffold and engineering conventions

| | |
|---|---|
| Task | TASK-005 — Create repository scaffold and engineering conventions |
| Date | 2026-10-04 |
| Performed by | Coding agent (Claude Code), one primary session; no sub-agents, reviewers or background AI jobs |
| Checkout | `/mnt/F/Nexees/`, branch `main`, base `0337be0488dc71bbae6dcaa1fa815991597d7807` |
| Repository | `https://github.com/aivars11232/Nexees` |
| Specification pack | `/mnt/F/Nexees_LCL_Implementation_Pack_v0.5.3/`, archive revision 0.5.3, specification version 0.5.0 |
| Pack content identity | `bd5e8fce26f542c44a891e4340da3b2a84f79ab368b5cb868beb48126c2db3e1` |
| Pack manifest SHA-256 | `8d9d0080ac13a3fb17cc70603b77eb3b11abcd3fb6d2fad515cfac136fcf07e4` |
| Procedure | Continuation profile 0.5.3 (CA-01 to CA-12), as adopted for TASK-002 and continued for this task |
| Predecessors | TASK-001 (`284f97a`), TASK-002 (`d1a673f`), TASK-003 (`2eaa199`, receipt decided in `bccc9b1`) and TASK-004 (`0337be0`), all accepted |
| Commit of this work | Made after acceptance, as CA-06 permits; it is recorded by Git, not in this file |

## Status

The deliverables are complete and every applicable check passed on its final run.
Acceptance is recorded in [receipt/RECEIPT_RESULT.md](receipt/RECEIPT_RESULT.md), which is
written after this file because the receipt contains a snapshot of everything else.

TASK-005 created the repository's engineering foundation:

- **The conventions** in [docs/engineering/CONVENTIONS.md](../../engineering/CONVENTIONS.md):
  - where things live, and how a placeholder becomes source;
  - Rust, Kotlin and TypeScript rules, readable code and comments, and formatting;
  - dependencies, generated artifacts and secrets;
  - tests, task evidence with its closure record, and scoped cleanup;
  - the checks, CI and the manual sources.
- **A Rust workspace** on the pinned toolchain, with lints, a license and advisory policy, and
  its first crate shell, `core/domain`.
- **A check runner** with nine stages, which fails closed and never writes into the checkout.
- **A scoped cleanup tool**, whose deletions are authorized only by a strictly validated
  manifest.
- **Fixture tests** proving that pre-existing untracked user work survives both the cleanup
  and the runner.
- **A CI entry point.**
- **Generated-artifact exclusions** that can never hide evidence.

No hosted CI service was enabled: that is the owner's decision.

All review here is the agent's own. Under CA-05 the routine corroboration was done by this
session's own local tools; no independent human or model review has taken place.

## 1. Authority

The owner's instruction, verbatim:

```
proceed with task 5 if task 4 is done and corrections too
```

TASK-004 and both owner-requested TASK-003 corrections were accepted and pushed before this
task began (`2eaa199..0337be0` on `origin/main`). This continues the procedure the owner
adopted with pack 0.5.3 (CA-01): one named task through evidence, receipt acceptance, commit
and push, then stop. TASK-006 is not covered. No owner question was needed during the task.

## 2. Checkout preflight (binding B26)

| Item | Observed |
|---|---|
| Working directory | `/mnt/F/Nexees/`; the Git worktree root, with no parent repository, superproject or second worktree |
| `origin` fetch and push | `https://github.com/aivars11232/Nexees.git`; no URL rewrite, no separate push URL |
| Branch and HEAD | `main` at `0337be0`, equal to `origin/main` after a fetch of that branch only, without tags |
| Hooks | None besides Git's samples; `core.hooksPath` unset |
| At task start | Clean worktree, no staged file, no stash, no ignored file |
| Result | Passes ([logs/preflight.txt](logs/preflight.txt)) |

## 3. Pack, native verification, dispatch and predecessors

**Pack.** All 201 manifest entries match their SHA-256, with no unlisted or missing file.
The content identity recomputes to `bd5e8fce…b3e1`. `tasks/task_005.lcl.txt` (`45db82df…e1b7`)
equals the `after_sha256` of `PROCEDURAL_CHANGES.json`, whose entry confirms that only
procedural fields changed from 0.5.2.

**Native verification.** The pack's runner passed all 51 synthetic language cases with
`lcl 0.9.1` ([logs/native_language_suite.json](logs/native_language_suite.json)). These are
language tests, not evidence for any task.

**Dispatch.** `lcl run … --input input.task_number=5 main.lcl.txt` ran `task.dispatch` and
`task.task_005` only, with `status.succeeded` and no diagnostics. It published the TASK-005
packet (SHA-256 of the packet text `33ee747f…f6a0`), with `procedure_amendment_applies: TRUE`
([logs/dispatch_task_005.record.json](logs/dispatch_task_005.record.json)).

**Predecessors.** All four engine records are intact and `status.succeeded`, and all four
commits are on `origin/main`:

| Task | Engine record |
|---|---|
| TASK-001 | `bbda5d83…07e7` |
| TASK-002 | `c5fadc83…bd13` |
| TASK-003 | `5e4a83c1…56d0` |
| TASK-004 | `2b7406b8…c83f` |

No predecessor deliverable or evidence folder had changed at task start.

## 4. Required reading

The continuation profile and the TASK-005 record were read first. Then the mandatory read
order: the readability and cleanup policy, the no-unnecessary-code policy, the manuals
architecture, the master rules, the global contracts, the bindings and the stop conditions.
Each was read again in the parts the scaffold relies on.

Every file is byte-identical to what TASK-002 to TASK-004 recorded, except the task file,
which is new ([logs/required_reading.txt](logs/required_reading.txt)).

The repository records that assign work to TASK-005 were read in full:

- **docs/architecture:** GAP-01, GAP-02, IF-PLATFORM, PD-CORE-RUNTIME, and the `scripts/ci/`
  and `scripts/test/` slots;
- **docs/dependencies:** DEP-RUST, DEP-ANDROID-BUILD, DEP-THEIA and DS-05 to DS-09;
- **docs/security:** SG-08, TH-41 and TH-52;
- **the charter's marker convention, CONV-MARKER-01.**

## 5. What was decided and built

- **Placeholders become source under the same stem.** The owning task replaces a
  `*.source` placeholder with its real file and language extension, so the architecture's
  stem-based slots keep working. The conventions fix the language per layer within DS-01.
  - Rust: the core, the platform adapters, the Desktop host and the integrations.
  - Kotlin: the Android APIs, lifecycle and UI.
  - TypeScript: the Desktop client inside Theia.
  - Python standard library and POSIX shell: the repository tooling.
- **One Rust workspace, one crate per core area.**
  - `rust-toolchain.toml` pins Rust 1.99.0 with rustfmt, clippy and the three targets.
  - The root `Cargo.toml` sets edition 2024, `publish = false` (no license is selected) and
    the workspace lints, each with its reason.
  - Every public item is documented.
  - No unwrap, expect, `todo!`, `dbg!` or console printing in shipped code. Unsafe code is
    denied, and forbidden in shared-core crates.
  - `clippy.toml` lets tests unwrap and print.
  - The first member is the crate shell `core/domain` (`nexees-domain`): a documented crate
    root and manifest only. TASK-006 adds its types.
  - An empty workspace cannot build at all, so one member is needed for the toolchain, lints
    and gates to run end to end.
- **The license and advisory policy** (`deny.toml`) carries DS-07's license list and DS-08's
  advisory gate; only crates.io is an allowed source.
- **The check runner,** `scripts/test/run_checks.py`, implements the placeholder
  `scripts/test/run_checks.source`. It runs nine stages: layout, format, lint, build, test,
  docs, security, deps and evidence. The things it checks that no earlier task checked:
  - the marker convention, in both directions;
  - every product and test file in the catalogue;
  - `.editorconfig` across the tree;
  - relative links, the four LCL projects, `FILE_TREE.txt` and the one manual source;
  - credentials, private keys and ignored evidence;
  - exact pins, a gate for every lockfile, and cargo-deny;
  - closure records.

  A missing tool fails its stage. The runner refuses a build folder inside the checkout.
  `--write-file-tree` regenerates the inventory, listing an open task's receipt files in
  advance.
- **The closure record.** From TASK-005 on, every receipt carries a `closure` object (schema
  `nexees-task-closure` version 1). It records the readability review, the cleanup, the manual
  impact and the post-cleanup verification (Q5 RC-05). The evidence stage rejects a missing,
  incomplete or unknown-field record.
- **Scoped cleanup,** `scripts/test/task_cleanup.py`, removes only the artifacts a versioned,
  strictly validated manifest lists, and only while they are unchanged.
  - It refuses tracked files, changed files, anything reached through a link or outside the
    checkout, and non-empty folders.
  - It opens every path component without following links and re-checks a file's identity
    just before unlinking it.
  - Its record lists every untracked file it left in place, with a hash.
- **The CI entry point,** `scripts/ci/continuous_integration.sh`, implements the placeholder
  `scripts/ci/continuous_integration.source`. It fetches the locked dependencies and runs
  every stage. It documents what a CI machine must provide; no hosted CI was configured.
- **Generated-artifact exclusions.** `.gitignore` names the stack's real outputs, and its last
  rule re-includes `docs/evidence/`. A log under evidence can never again be left out of a
  commit, as `pc_receiver.log` was in TASK-003. `.editorconfig` and `.gitattributes` gained the
  language sections their placeholders deferred until the stack was chosen.
- **The scaffold items earlier tasks assigned to TASK-005:**
  - the local-IPC slot `platform/desktop/transport/local_ipc`, which resolves GAP-01;
  - the CI slot for the dependency gate, which resolves SG-08;
  - a repository secret scan in CI (TH-52) and the supply-chain gate (TH-41).

  IF-PLATFORM's port pattern is a convention, not code: a port is added together with its
  first implementation, since an unused interface would be unnecessary code. GAP-02 (a startup
  test slot, "TASK-005 or TASK-009") is left to TASK-009, which builds the startup it tests.

## 6. Deliverables

**New files:**

| Path | Purpose |
|---|---|
| `docs/engineering/CONVENTIONS.md` | The conventions (section 5) |
| `rust-toolchain.toml`, `Cargo.toml`, `Cargo.lock`, `clippy.toml`, `deny.toml` | The Rust workspace and its policies |
| `core/domain/Cargo.toml`, `core/domain/lib.rs` | The first crate shell |
| `scripts/test/run_checks.py` | The check runner |
| `scripts/test/task_cleanup.py` | The scoped cleanup tool |
| `scripts/ci/continuous_integration.sh` | The CI entry point |
| `tests/tooling/test_task_cleanup.py`, `tests/tooling/test_run_checks.py` | The tooling tests and fixtures |
| `platform/desktop/transport/local_ipc.source` | The GAP-01 placeholder, marker only |
| `docs/evidence/TASK-005/` | This record, `logs/` and, written last, `receipt/` |

**Removed files:** the two placeholders, now implemented under the same stems:
`scripts/test/run_checks.source` and `scripts/ci/continuous_integration.source`.

**Changed files:**

| Path | Change |
|---|---|
| `.gitignore`, `.editorconfig`, `.gitattributes` | Stack-specific rules; their marker lines are removed, since they are now edited |
| `SECURITY.md` | A sentence on the runner's secret scan; its marker line is removed (section 11) |
| `README.md` | The scaffold, the conventions, the checks, the layout table and the inventory note |
| `FILE_TREE.txt` | Regenerated with the runner |
| `docs/architecture/subsystems.lcl.txt` | The slots of the new files, TASK-005 as a co-owner of SS-DOMAIN-PROTOCOL, the tooling test slots, the local-IPC slot |
| `docs/architecture/decisions.lcl.txt` | GAP-01 resolved; PD-CORE-RUNTIME notes the scaffold |
| `docs/dependencies/components.lcl.txt` | DEP-RUST adds rustfmt and the pin |
| `docs/dependencies/strategy.lcl.txt`, `checks.lcl.txt`, `record.lcl.txt`, `LICENSE_MATRIX.md` | The repository tools TOOL-01 to TOOL-03: cargo-deny, Python 3 and the `lcl` engine |
| `docs/security/tests.lcl.txt` | SG-08 resolved |

**New dependencies:** none that a product build links or ships. Two additions to tooling:

- **The rustfmt component** of the already selected Rust 1.99.0, installed into the approved
  isolated toolchain folder ([logs/toolchain.txt](logs/toolchain.txt)). rustup verifies each
  download's SHA-256.
- **The repository tools are now inventoried as TOOL-01 to TOOL-03.** cargo-deny is the
  owner's existing install, executed only, as in TASK-003. Python 3 uses the standard library
  only. The LCL engine is the owner's tool, executed only.

TASK-003's frozen cross-check counts exactly 49 components, so these went into their own list
instead of new component records.

## 7. Checks run

| Command | Result | Log |
|---|---|---|
| `python3 -B scripts/test/run_checks.py`, all nine stages on the final tree | All pass, exit 0 | [run_checks.txt](logs/run_checks.txt) |
| `python3 -B -m unittest discover -s tests/tooling` (inside the test stage) | 22 of 22 pass | [run_checks.txt](logs/run_checks.txt) |
| Negative tests: 22 seeded defects in scratch copies of the checkout, plus 2 controls | All 22 caught by the intended stage; the controls pass | [negative_tests.txt](logs/negative_tests.txt) |
| Regression: the charter, architecture, dependency and security projects, and the TASK-001 to TASK-004 cross-checks | 20 commands, all exit 0 | [regression.txt](logs/regression.txt) |
| Native language suite | 51/51 | [native_language_suite.json](logs/native_language_suite.json) |
| Toolchain: rustfmt installed, versions recorded | Recorded | [toolchain.txt](logs/toolchain.txt) |
| Secret and privacy scan | No finding | [security_scan.txt](logs/security_scan.txt) |
| Scoped cleanup with the new tool | Nothing to remove; untracked files recorded as kept | [cleanup_record.json](logs/cleanup_record.json) |

The negative tests seed one defect per case, and the runner catches each:

- **Placeholders and catalogue:** an edited placeholder that kept its marker, a marker stripped
  without an edit, a product file without a slot, an unattributed test file.
- **Formatting and lint:** trailing whitespace, unformatted Rust, an unwrap, an undocumented
  public item, a script without a docstring.
- **Documentation:** a broken link, a stale `FILE_TREE.txt`, a manual without its status, a
  second manual copy.
- **Security and dependencies:** a credential, ignored evidence, a range instead of a pin, an
  ungated lockfile.
- **Evidence:** a missing receipt file, an incomplete closure record.
- **The cleanup tool itself,** where the fixture tests catch it:
  - when it stops refusing tracked files;
  - when it starts following links;
  - when it accepts unknown manifest fields.

## 8. Check accounting

### Verification

| Check | How it was met |
|---|---|
| T005.VERIFY.01 formatting and documentation checks, and a fixture protecting pre-existing untracked user work | The format and docs stages pass on the final tree. Two fixtures exercise that protection: `tests/tooling/test_task_cleanup.py` (8 tests: user files byte-identical; tracked, changed, linked, outside and non-empty targets refused; invalid manifests rejected before any deletion) and `test_run_checks.ReadOnlyRunTest` (a run over a checkout holding untracked user work changes no file). |
| T005.VERIFY.02 build the affected targets | The workspace builds, lints with warnings denied, tests and documents on Rust 1.99.0 (build, lint, test, docs stages). The repository scripts parse (lint stage). No Kotlin or TypeScript build exists yet (section 12). |
| T005.VERIFY.03 task-specific tests | The 22 tooling tests and the 22 negative tests (section 7). |
| T005.VERIFY.04 regressions of the changed subsystem | The changed predecessor records pass their own LCL projects and the TASK-002, TASK-003 and TASK-004 cross-checks. The charter and TASK-001's cross-check are unchanged and pass. The native suite passes. |
| T005.VERIFY.05 final diff inspected | Section 6 lists every new, changed and removed file; the receipt's final verification repeats the inspection. |

### Completion gate

| Check | How it was met |
|---|---|
| T005.CLOSE.01 dependencies closed | TASK-004 accepted and its lineage verified (section 3). |
| T005.CLOSE.02 objective without unrelated scope | The objective's items are each built: minimal structure, formatting, lint and test conventions, the CI skeleton, the documentation locations and the generated-artifact exclusions. So are the scope's readable-code conventions, closure and cleanup schemas and manual source location. No domain type, protocol or client was started. The `SECURITY.md` and catalogue edits serve this task's own deliverables. |
| T005.CLOSE.03 build passes | As T005.VERIFY.02. |
| T005.CLOSE.04 required tests pass | As T005.VERIFY.03. |
| T005.CLOSE.05 security checks pass | The security stage and the scan are clean, cargo-deny passes, and the cleanup tool's refusal of links, tracked and changed files is tested. Every new schema is versioned and strictly validated: the cleanup manifest, which authorizes deletion, and the closure record both reject unknown fields. |
| T005.CLOSE.06 no unnecessary code or dependency | No product logic and no product dependency. Each file maps to the objective (section 6), and the one crate shell is the minimum a buildable workspace needs. An unused helper that review found was removed. No port trait was added ahead of its implementation. |
| T005.CLOSE.07 no workspace/agent/LCL binding invariant violated | Bound checkout, branch and remote. The pack, the LCL repository, the canonical packages and the LCL SDK were not written (final verification). One primary agent. No process outlived its command. |
| T005.CLOSE.08 evidence recorded | This folder. |
| T005.CLOSE.09 Git diff understood | As T005.VERIFY.05. |
| T005.CLOSE.10 readability and comments | Agent's own review, section 9. |
| T005.CLOSE.11 scoped cleanup | Section 9; [logs/cleanup_record.json](logs/cleanup_record.json). |
| T005.CLOSE.12 post-cleanup verification on the final revision | `receipt/final_verification.txt`. |
| T005.CLOSE.13 manual impact | Section 9. |

## 9. Review, cleanup and manuals

**Readability (agent's own review).** Each script, crate root and configuration file opens
with its purpose. Comments state why a rule exists and cite the requirement it serves. Tests
name the behaviour they prove. Review against the final code made these changes:
- the cleanup tool's checks moved onto one no-follow, non-blocking open with an identity
  re-check, and its plan-mode handling of sub-folders was fixed;
- an unused runner parameter and an unused helper were removed;
- the docs stage now checks the file tree and manuals before the LCL engine, so a missing
  engine cannot hide them;
- command failures are printed as findings, which the negative tests showed were missing;
- the first full run showed that the evidence record's link to its own receipt counted as broken
  before the receipt existed, so such a link is now accepted for an open task, as the file tree
  already was, and has its own test;
- one wrong test expectation was corrected.

**Cleanup.** TASK-005 created no temporary artifact inside the checkout:
- Cargo wrote to `/mnt/F/Nexees-toolchains/target/`;
- tests and the negative-test copies used the system temporary folder and the session's
  scratch folder;
- every Python run used `-B`.

The cleanup tool ran on the real checkout with an empty manifest. It removed nothing and
recorded every untracked file it left in place, all of them TASK-005 deliverables and
evidence ([logs/cleanup_record.json](logs/cleanup_record.json)). The session scratch folder
outside the checkout is removed after the receipt. The isolated toolchain folder keeps the
rustfmt component and the build outputs, as the owner approved for TASK-003. No pre-existing
or user-owned file was deleted, reset or stashed.

**Manuals.** Not applicable to the user manuals: TASK-005 changes no user-visible behaviour.
The manual sources now have a fixed location, checked by the docs stage: exactly the two files
in `docs/manuals/`, each stating its status, with no second copy anywhere.

## 10. Policy review and disclosures

All 135 rules of the nine policy documents were reviewed for this task.

| Policy document | Rules | Relation to TASK-005 |
|---|---|---|
| `policies/master_rules.lcl.txt` | 32 | R3, R4, R6, R15, R25 and R26 are what the runner, the cleanup tool and the conventions make routine |
| `policies/global_contracts.lcl.txt` | 31 | C16, C17, C25, C26 and C27 shaped the dependency gate, the file-to-requirement mapping, the cleanup, the manual source rule and the preflight |
| `policies/acceptance_criteria.lcl.txt` | 11 | Product acceptance, none due now; the closure record makes each task's closure checkable |
| `policies/code_readability_and_cleanup.lcl.txt` | 6 | Q1 to Q5 are installed: conventions, lints, the cleanup tool and its fixtures, and the closure record (RC-02, RC-04, RC-05) |
| `policies/no_unnecessary_code.lcl.txt` | 7 | No product logic; minimal crate shell; no unused interface; new files, removals and tools listed (section 6) |
| `policies/reuse_policy.lcl.txt` | 5 | Reuse of the selected toolchain and cargo-deny; small internal checks instead of new packages |
| `policies/security_baseline.lcl.txt` | 9 | Secret scan, fail-closed checks, strict versioned schemas, unsafe code denied, output kept away from consoles |
| `policies/usage_and_agents.lcl.txt` | 8 | One agent; builds one at a time; Git writes only as CA-06 permits |
| `architecture/remote_device_control.lcl.txt` | 26 | No remote requirement is assigned to TASK-005; nothing remote was touched |

Disclosures:

- **Network.** One download: the rustfmt component from the Rust distribution server, into
  the isolated toolchain, under CA-07. The freshness check fetched only `main` from `origin`.
- **Session hook.** A hook asks for a dynamic web-application security scan after code
  changes. No Nexees web application is running and `HAWK_API_KEY` is unset, so none was run.
- **Owner-only state.** cargo-deny was executed from the owner's `~/.cargo/bin` without
  writing there; its advisory database stays in the isolated `CARGO_HOME`. HopToDesk, linger,
  the autostart folder, the phone and the LCL SDK were not touched.

## 11. Deviations and findings

- **A TASK-004 slip, fixed forward.**
  - TASK-004 edited `SECURITY.md` but kept its marker line, contrary to CONV-MARKER-01. A check
    of every marked file against the import commit found no other case.
  - TASK-005 edits `SECURITY.md` for its own reason, documenting the secret scan, and removes
    the marker in that change.
  - The layout stage now enforces the convention, as the regression test this fix needs.
  - TASK-004's accepted receipt is not changed: none of its inputs is false.
- **The negative tests found a reporting gap.** The runner recorded failed commands as findings
  but did not print them. It now prints them, and the suite was run again.
- **A test-design note.** The first negative-test control failed only because `FILE_TREE.txt`
  predated a new log file. Regenerating it fixed that; such a failure is the docs stage working
  as intended.

## 12. Open items and limits

- **Hosted CI** is not configured. Choosing a service, and any cost it brings, is the owner's
  decision. Until then the checks run locally through the same entry point.
- **Kotlin and TypeScript** have conventions but no stages. TASK-057 and TASK-009 add their
  formatters, lints and builds with the first code.
- **Lockfile gates.** The npm and Gradle lockfile gates are added by the tasks that introduce
  those lockfiles; until then the deps stage refuses such a lockfile.
- **CI requirements.** A CI machine needs:
  - the full Git history;
  - the pinned toolchain;
  - cargo-deny 0.20.2;
  - the LCL engine with the canonical Core packages.
- **Deferred to later tasks:** GAP-02 goes to TASK-009. IF-PLATFORM's ports come with their
  first implementations.
- **Also open:** the open items of `docs/dependencies` (OI-01 to OI-10), the open security
  decisions of `docs/security` (OSD-01 to OSD-08), BIND-B1-LOGO and AMEND-075-01, as their
  records state.

## 13. Next task

TASK-006 (shared domain model). It was not started.
