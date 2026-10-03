# Nexees engineering conventions

How Nexees is laid out, written, built, checked and closed, task by task. The LCL
implementation pack's policies govern; this document turns them into repository practice.
Where a rule is checked by a machine, the stage of `scripts/test/run_checks.py` that checks
it is named in brackets.

Established by TASK-005. A task that changes a convention updates this document, and its
check, in the same change.

## 1. Where things live

| Location | Holds |
|---|---|
| `apps/desktop/`, `apps/android/` | The two clients and their local-host composition roots |
| `core/` | The shared core, one Rust crate per area (section 3) |
| `platform/desktop/`, `platform/android/` | Platform adapters that implement the core's ports |
| `integrations/` | Adapters to the IDE foundation, LCL, auth, providers, external agents, Git and the code index |
| `assets/`, `config/`, `packaging/` | Visual resources, configuration slots, package definitions (never signing keys) |
| `scripts/build/`, `scripts/ci/`, `scripts/test/`, `scripts/release/` | Build helpers, the CI entry point, the check runner with the scoped cleanup tool, and release helpers |
| `tests/<layer>/` | Product tests by layer (`unit`, `integration`, `conformance`, `security`, `e2e`, `fixtures`); `tests/tooling/` tests the repository tooling |
| `docs/charter/` | The frozen project charter (TASK-001) |
| `docs/architecture/`, `docs/dependencies/`, `docs/security/` | The architecture, the dependency inventory and the threat model, each an LCL project |
| `docs/engineering/` | These conventions |
| `docs/manuals/` | The one content source of both user manuals (section 13) |
| `docs/evidence/TASK-NNN/` | One task's evidence and receipt (section 10) |

Every file under `apps/`, `core/`, `platform/`, `integrations/`, `assets/`, `config/`,
`packaging/`, `scripts/` and `docs/manuals/` belongs to exactly one slot of
`docs/architecture/subsystems.lcl.txt`, and every test file is attributed to exactly one
subsystem. A task that adds, moves or removes such a file updates that catalogue in the same
change. [layout]

## 2. From placeholder to source

A `*.source` file is a named placeholder. It holds only its marker line, `// start here`.
When the owning task implements it, the task replaces the `.source` file with the real file
under the **same stem** and its language's extension: `core/domain/workspace.source` becomes
`core/domain/workspace.rs`. The stem keeps the architecture slot valid.

The marker convention CONV-MARKER-01 is recorded in the charter, and its rules are checked
[layout]:

- A task that edits a file carrying a marker removes the marker in the same change.
- A task never strips a marker from a file it does not otherwise edit.
- A marked file is therefore an untouched placeholder: as the layout import created it, or,
  when a later task adds one, nothing but its marker line.

| Layer | Language and extension |
|---|---|
| `core/` | Rust (`.rs`) |
| `platform/desktop/` | Rust |
| `platform/android/` | Rust for what the core calls; Kotlin (`.kt`) for Android APIs, lifecycle and UI (DS-01) |
| `apps/desktop/` | TypeScript (`.ts`, `.tsx`) on Theia and Electron; the local host is a Rust binary |
| `apps/android/` | Kotlin, with the core linked through JNI |
| `integrations/` | Rust; the IDE adapters are TypeScript, because they run inside Theia |
| `scripts/` | Python 3.11 or later, standard library only (`.py`); POSIX shell (`.sh`) for thin entry points |
| `tests/` | The language of what the test exercises; `tests/tooling/` is Python |

The task that creates a build for a layer confirms this split in its own evidence. It may
refine the split only within DS-01.

## 3. Rust

- **Toolchain.** `rust-toolchain.toml` pins Rust 1.99.0 with rustfmt, clippy and the Desktop
  and Android targets. All crates use edition 2024 and `rust-version = "1.99"`.
- **One crate per area.** Each directory under `core/` becomes one library crate,
  `nexees-<area>`, when its owning task first implements it:
  - its manifest is `core/<area>/Cargo.toml` and its root is `core/<area>/lib.rs`, with
    `[lib] path = "lib.rs"` and `[lints] workspace = true`;
  - its modules are the slot stems;
  - the task adds the crate to `members` in the root `Cargo.toml`.

  The first crate is `core/domain` (`nexees-domain`).
- **Dependencies between crates** follow the architecture: a crate depends only on crates of
  subsystems that its own subsystem's `depends_on` lists. That keeps the graph acyclic (AD-12).
- **Platform ports.** The core defines a platform port as a trait in the crate of the subsystem
  that needs it, and each platform crate implements it (IF-PLATFORM). A port is added together
  with its first implementation, never ahead of it.
- **Lints.** The workspace lints in `Cargo.toml` apply to every crate, and the checks deny
  warnings:
  - every public item is documented;
  - shipped code never unwraps, expects or panics on purpose; it returns typed errors, so
    failures stay closed;
  - no `todo!`, `dbg!` or console printing; output goes through logging and its secret
    redaction (SI-08);
  - unsafe code is denied. Shared-core crates also `#![forbid(unsafe_code)]`. A platform
    adapter that needs unsafe code allows it locally, next to a `// SAFETY:` comment.

  Tests may unwrap and print (`clippy.toml`). [lint]
- **Formatting.** rustfmt's defaults, 100 columns. [format]
- **Builds stay outside the checkout** (DS-09). Set `CARGO_TARGET_DIR` to a folder outside it.
  `run_checks.py` refuses a target folder inside the checkout.

## 4. Kotlin and TypeScript

- **Kotlin** follows the official Kotlin coding conventions with 4-space indentation. TASK-057
  chooses its formatter and lint (Android Lint at least) under the dependency gate and adds
  them to the runner.
- **TypeScript** uses strict mode and Theia's style with 4-space indentation. TASK-009 chooses
  its lint and formatter under the dependency gate and adds them to the runner. npm
  dependencies follow DS-05 and DS-06 (section 6).
- Until those builds exist, the runner has no Kotlin or TypeScript stage. The task that adds a
  build adds its stage in the same change.

## 5. Readable code and comments

These rules apply `policies/code_readability_and_cleanup.lcl.txt` (Q1) and master rule R25 to
every change.

- **Module documentation.** Every crate, module and script opens with what it is for and where
  its boundaries are. [lint: module docstrings of scripts; `missing_docs` for Rust]
- **What must be documented.** Public interfaces state their inputs, outputs, side effects,
  failure modes and invariants where the interface does not already show them. Always
  document:
  - workspace and device bindings, and state transitions;
  - synchronization conflicts and permission decisions;
  - read-only handoff enforcement and archive path checks;
  - retry and idempotency rules, and platform restrictions.
- **How comments are written.** Comments say what important code does and why a non-obvious
  choice exists. They do not narrate trivial lines and are not decorative. They are updated in
  the same change as the code, and commented-out code is deleted. A comment may cite the
  requirement it serves (SI-08, RC-09). It does not tell the project's history, which belongs
  in Git and the evidence.
- **Tests** say in their names and docstrings which behaviour or edge case they prove.
- **Generated and vendored code** keeps its own conventions and notices, and is not
  hand-edited.

## 6. Dependencies

- **Exact pins (DS-05).** Every Rust dependency requirement is `=x.y.z` or a workspace or path
  reference, and `Cargo.lock` is committed. Shared versions live in `[workspace.dependencies]`.
  [deps]
- **Install scripts (DS-06).** npm packages run install scripts only when `allowScripts` lists
  them.
- **Licenses (DS-07).** `deny.toml` allows only the licenses that need no review. A license
  with duties is allowed per crate, after its duties are recorded in `docs/dependencies`.
  [deps: cargo-deny]
- **Advisories (DS-08).** cargo-deny checks RustSec. Each kind of lockfile needs its own gate:
  the deps stage fails on a lockfile no gate covers, so the task that introduces npm or Gradle
  locking adds its gate first. [deps]
- **Before adding a dependency,** record it in `docs/dependencies` with every gate item; then
  add it. A dependency is also code: no package for a trivial function.
- **Repository tools are inventoried as build dependencies:** the Rust toolchain, cargo-deny,
  Python 3 and the LCL engine.

## 7. Generated artifacts and local state

Build outputs, dependency caches, editor state, logs and machine-local configuration are
never committed. `.gitignore` lists them by the stack's real output names.

**An ignore rule is never permission to delete.** Cleanup removes only what a task recorded
(section 11).

Task evidence is never ignored: the last rule of `.gitignore` re-includes `docs/evidence/`,
and the security stage fails if any evidence file is ignored. [security]

## 8. Secrets

Secrets, recovery codes, private keys, real conversations and private workspace data never
enter the repository, logs, prompts or test fixtures. The security stage scans every file a
commit would hold for credential formats and private-key files. [security] A test that needs
a credential-shaped string assembles it at runtime, so the source itself holds none.

## 9. Tests

- **Layers.** The layers are those of the pack's test strategy. Product tests go in the slots
  under `tests/<layer>/` that the architecture names. Unit tests of a Rust crate sit beside
  its code in `#[cfg(test)]` modules.
- **Behaviour.** Tests are deterministic and use no network. Fixtures are generated and
  bounded: no real resource-exhaustion payloads, and no real personal data.
- **Regressions.** A bug fixed after its task closed gets a regression test, or a written
  reason why that is impossible.
- **Repository tooling** has its tests in `tests/tooling/`, run by the test stage. They include
  the fixtures that protect pre-existing untracked user work. [test]

## 10. Task evidence and closure

Each task keeps one folder, `docs/evidence/TASK-NNN/`:

- `TASK-NNN_EVIDENCE.md`, the record;
- a cross-check script when the engine cannot see what must be checked;
- `logs/`;
- `receipt/`, written last and excluded from the snapshot, with six files:
  `RECEIPT_RESULT.md`, `corroboration.txt`, `evaluation.record.json`,
  `final_verification.txt`, `receipt.json` and `snapshot_files.sha256`.

`FILE_TREE.txt` lists the receipt files before they exist. `run_checks.py --write-file-tree`
regenerates it. [docs]

From TASK-005 on, `receipt.json` carries a **closure record**, the object `closure`. It
records the four things every closure must record (Q5 RC-05). The evidence stage validates it
strictly: an unknown or missing field fails, and so does a closed task without one. [evidence]

| Field | Content |
|---|---|
| `schema`, `version` | `"nexees-task-closure"`, `1` |
| `task` | `"TASK-NNN"`, equal to the folder |
| `readability` | `reviewer` (who reviewed comments and names against the final code) and `summary` |
| `cleanup` | `summary`, `record` (the path of the cleanup record within the task folder, or `null` for a no-op whose summary gives the reason), `removed` (paths) and `preserved_untracked` (count) |
| `manual_impact` | `status` (`"updated"` or `"not_applicable"`) and `detail` |
| `final_verification` | `log` (`"receipt/final_verification.txt"`), `snapshot` (equal to the receipt's snapshot) and `result` (`"passed"`) |

## 11. Scoped cleanup

Cleanup at task close follows Q3 and never uses `git clean`, `git reset --hard`, a stash or
wildcard deletion. The steps:

1. **Record each temporary artifact** a task creates inside the checkout as it creates it:
   `scripts/test/task_cleanup.py record MANIFEST TASK-NNN PATH...` (a folder as `PATH/`).
   Keep the manifest outside the checkout. Artifacts outside the checkout, such as scratch
   folders, are removed by the task itself and named in the cleanup summary.
2. **At close, read the plan:** `task_cleanup.py clean MANIFEST`.
3. **Apply it, with the record in the evidence:**
   `task_cleanup.py clean MANIFEST --apply --record docs/evidence/TASK-NNN/logs/cleanup_record.json`.
   The tool refuses, and keeps:
   - a tracked file;
   - a file changed since it was recorded;
   - anything behind a symbolic link or outside the checkout;
   - a folder that is not empty.

   The record lists every untracked file left in place, with its hash: the user work the
   cleanup preserved.
4. **Run the checks again** after cleanup, then summarise the record in the closure.

A manifest is JSON with exactly the fields `schema` (`"nexees-task-artifacts"`), `version`
(`1`), `task` and `artifacts`. Each artifact has either `path` and `sha256`, or a `path`
ending in `/` and `"directory": true`.

**A manifest with anything else is rejected before anything is deleted**, because it
authorizes deletion. [test: `tests/tooling/test_task_cleanup.py`]

## 12. Checks and continuous integration

`scripts/test/run_checks.py` runs the stages in this order:

| Stage | Checks |
|---|---|
| `layout` | Placeholder markers, and the architecture catalogue of product and test files |
| `format` | `.editorconfig` for every text file outside `docs/evidence/`, and rustfmt |
| `lint` | clippy with warnings denied; repository scripts parse and have a module docstring |
| `build` | The workspace, locked |
| `test` | The Rust tests and `tests/tooling/` |
| `docs` | rustdoc with warnings denied; relative Markdown links outside fixtures; `lcl check`, `validate` and `run` of the charter, architecture, dependency and security projects; `FILE_TREE.txt`; the manual sources |
| `security` | Credentials and private keys, and ignored evidence |
| `deps` | Exact pins, gated lockfiles and cargo-deny |
| `evidence` | Complete receipts, and closure records from TASK-005 on |

Name one or more stages with `--stage`. The runner fails closed: a missing tool or setting
fails its stage instead of skipping it. It never writes into the checkout, except that
`--write-file-tree` rewrites `FILE_TREE.txt` on request.

It needs:
- Python 3.11 or later and Git;
- rustup for the pinned toolchain;
- cargo-deny on `PATH`;
- the LCL engine `lcl`, with `NEXEES_LCL_CORE_01` and `NEXEES_LCL_CORE_03` naming the canonical
  Core 0.1.0 and 0.3.0 packages.

On the development machine:

```bash
source /mnt/F/Nexees-toolchains/env.sh
export PATH="$PATH:$HOME/.cargo/bin"
export NEXEES_LCL_CORE_01=/mnt/F/LCL/canonical/LCL_Core_0.1.0 NEXEES_LCL_CORE_03=/mnt/F/LCL/canonical/LCL_Core_0.3.0
python3 -B scripts/test/run_checks.py
```

`scripts/ci/continuous_integration.sh` is the entry point a CI service runs. It fetches the
locked dependencies, then runs every stage, with cargo-deny refreshing its advisory database.
The CI machine needs the full Git history, because the layout stage compares placeholders
with the commit that imported them.

**No hosted CI service is configured.** Choosing one, and any cost it brings, is the owner's
decision.

## 13. Manual sources

`docs/manuals/NEXEES_USER_MANUAL.md` and `docs/manuals/LCL_IN_NEXEES_USER_MANUAL.md` are the
one content source of the two manuals, for both clients. Each states its status in a
`**Status:**` line. No second copy exists elsewhere in the repository. [docs]

The bundling helper `scripts/build/bundle_manuals` (TASK-071 and TASK-072) builds the in-app
manuals from them. A feature task updates them in the same change as the behaviour they
describe (R26), or records why its change has no manual impact.
