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
| `assets/` | Data both clients read is JSON (`.json`); pictures are PNG. No code (section 14) |
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

  The crates so far are `core/domain` (`nexees-domain`) and the two that build on it,
  `core/protocol` (`nexees-protocol`) and `core/state` (`nexees-state`).
- **Platform and host crates** follow the same pattern outside `core/`. `platform/desktop` is
  the library crate `nexees-platform-desktop`, with its root at `platform/desktop/lib.rs` and
  one module per slot in its subfolders. The Desktop host is the binary `nexees-host` of the
  crate `nexees-desktop-host`, whose manifest `apps/desktop/Cargo.toml` names its slot file
  `apps/desktop/src/application_host.rs` as the binary's root (TASK-009).
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
- **TypeScript** uses strict mode and Theia's style with 4-space indentation. TASK-009 chose no
  further package for it. The lint is the TypeScript compiler of the reviewed Theia tree with
  every strict check on (`apps/desktop/src/tsconfig.json`), which the lint stage runs through
  `scripts/build/build_desktop.py check`. The formatting rules are `.editorconfig`'s, which the
  format stage enforces. A formatter or ESLint may be added later, under the dependency gate.
  npm dependencies follow DS-05 and DS-06 (section 6).
- **The Desktop window is built outside the checkout** by `scripts/build/build_desktop.py`
  (`check`, `build`, `install`), from copies of its sources, with npm offline on the committed
  lockfile. `install` also sets, in the installed copy of the Electron binary, the fuses that
  `packaging/desktop/package_definition.json` names (SI-28). Kotlin has no stage until
  TASK-057 adds its build, in the same change.
- **An editor finds the window's packages through an optional link**, `apps/desktop/node_modules`,
  to `check/node_modules` in the Desktop build folder. Git ignores the link and no build reads
  it. Without it an editor reports the window's imports as unresolved, while the lint stage
  still checks them.

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
  the deps stage fails on a lockfile no gate covers, so the task that introduces Gradle locking
  adds its gate first. The npm gate (TASK-009) checks each `package-lock.json`: exact versions,
  packages only from the npm registry with an sha512 hash, an `allowScripts` decision for every
  install script, and the locked package set against its OSV review recorded in
  `docs/dependencies/npm_review.json`. A changed lockfile needs a new review there. [deps]
- **Before adding a dependency,** record it in `docs/dependencies` with every gate item; then
  add it. A dependency is also code: no package for a trivial function.
- **Repository tools are inventoried as build dependencies:** the Rust toolchain, cargo-deny,
  Python 3, the LCL engine, and Node.js with npm (TOOL-04).

## 7. Generated artifacts and local state

Build outputs, dependency caches, editor state, logs and machine-local configuration are
never committed. `.gitignore` lists them by the stack's real output names.

**An ignore rule is never permission to delete.** Cleanup removes only what a task recorded
(section 11).

Task evidence is never ignored. The last rules of `.gitignore` re-include `docs/evidence/`, except
the folders Gradle and IDEs generate beside a Gradle prototype kept there: `.gradle/`, `.kotlin/` and
`build/`. An IDE writes such a cache when it imports the prototype. The security stage fails on any
other ignored evidence file, and on those folders unless a Gradle build file of the repository sits
beside them. [security]

An IDE may keep its own build output in the checkout, such as rust-analyzer's `target/`. It is
ignored, never committed and outside every task's snapshot (section 10); the task's final
verification lists it. The Desktop window's packages and build live in a folder outside the
checkout too: the check runner uses `desktop/` beside its Cargo target folder.

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
- **End-to-end tests** of an installed application run outside the runner, because they need
  the built application and a display. `tests/e2e/desktop/local_application.test.mjs` takes a
  prefix made by `scripts/build/build_desktop.py install` and runs the application in a private
  nested session of its own; the task that changes the Desktop records its run. It is also the
  test of the Desktop runtime's hardening (SG-05), and it checks what the user sees: an entry
  counts only where it is visible. At each probe it asks the page for one pixel, which makes the
  page draw a frame: while the developer's real screen is locked, the nested compositor locks
  its own screen too and draws nothing, and Theia's start waits for a frame.

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

A receipt's **snapshot** is what a commit of the task holds. It is the SHA-256 of one `sha256sum`
line per file Git would commit, tracked or untracked but not ignored, outside the task's
`receipt/`, sorted by path under `LC_ALL=C`. Ignored local output is not part of it.

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
| `layout` | Placeholder markers; the architecture catalogue of product and test files; every enforcement point of the threat model names a slot, as a placeholder or as source |
| `format` | `.editorconfig` for every text file outside `docs/evidence/`, and rustfmt |
| `lint` | clippy with warnings denied; repository scripts parse and have a module docstring; JavaScript parses; the Desktop TypeScript compiles with every strict check |
| `build` | The workspace, locked |
| `test` | The Rust tests and `tests/tooling/` |
| `docs` | rustdoc with warnings denied; relative Markdown links outside fixtures; `lcl check`, `validate` and `run` of the charter, architecture, dependency and security projects; `FILE_TREE.txt`; the manual sources |
| `security` | Credentials and private keys, and ignored evidence |
| `deps` | Exact pins, gated lockfiles (cargo-deny, and the npm gate with its recorded review) |
| `evidence` | Complete receipts, and closure records from TASK-005 on |

Name one or more stages with `--stage`. The runner fails closed: a missing tool or setting
fails its stage instead of skipping it. It never writes into the checkout, except that
`--write-file-tree` rewrites `FILE_TREE.txt` on request.

It needs:
- Python 3.11 or later and Git;
- rustup for the pinned toolchain;
- cargo-deny on `PATH`;
- the LCL engine `lcl`, with `NEXEES_LCL_CORE_01` and `NEXEES_LCL_CORE_03` naming the canonical
  Core 0.1.0 and 0.3.0 packages;
- Node.js with npm, with `NEXEES_NPM_CACHE` naming an npm cache that holds the Desktop
  window's locked packages and `NEXEES_BUILD_HOME` the home folder npm runs with.

On the development machine:

```bash
source /mnt/F/Nexees-toolchains/env.sh
export PATH="$PATH:$HOME/.cargo/bin"
export NEXEES_LCL_CORE_01=/mnt/F/LCL/canonical/LCL_Core_0.1.0 NEXEES_LCL_CORE_03=/mnt/F/LCL/canonical/LCL_Core_0.3.0
export NEXEES_NPM_CACHE=/mnt/F/Nexees-toolchains/npm-cache NEXEES_BUILD_HOME=/mnt/F/Nexees-toolchains/home
python3 -B scripts/test/run_checks.py
```

`scripts/ci/continuous_integration.sh` is the entry point a CI service runs. It fetches the
locked dependencies, the Rust crates and the Desktop window's npm packages, then runs every
stage, with cargo-deny refreshing its advisory database.
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

## 14. The visual system and the logo

Established by TASK-010, for both clients (R18, R19).

- **One source of the look.** `assets/theme/design_tokens.json` holds the colours, text sizes,
  spacing and control sizes, and `assets/theme/icon_mapping.json` the icon of each concept, by
  its Codicon name. A client's code holds no colour, size or icon name of its own. A group of
  tokens whose name ends in `_percent` holds shares instead of sizes: the part of its space a
  panel takes when it is first shown.
- **Each client has a generated copy** in its own language, committed beside the code that
  uses it. `scripts/build/design_tokens.py` writes it and is the only thing that does; the
  Desktop copy is `apps/desktop/src/shell/design_tokens.ts`. To change the look, edit the JSON,
  run the script and commit both. The script refuses a source it does not understand and any
  text, accent, status or syntax colour whose contrast against a surface is under 4.5 to 1.
  [test: `tests/tooling/test_design_tokens.py` checks that the committed copy is current and
  that the window uses every token and icon]
- **The Desktop theme** is `apps/desktop/src/shell/theme.ts`. It says which token each of
  Theia's colours and size variables takes, and holds no colour value itself. The end-to-end
  test checks that Theia knows and applies every colour it names.
- **The logo has one source** (binding B1): the file the owner confirmed,
  `brand_assets.LOGO_SOURCE`. `scripts/build/brand_assets.py import` reads only that file,
  never writes to it, and offers no way to name another. It writes the byte-identical copy,
  the derived icons and `manifest.json` under `assets/branding/`. Those files are never
  edited, redrawn or replaced by hand. An icon is the source scaled down; a size larger than
  the source is refused. A new size is added to `ICON_SIZES` and derived by a new import on
  the owner's machine. [test: `tests/tooling/test_brand_assets.py` checks that the committed
  files are the recorded source and exactly its icons]
- **A build takes only recorded icons.** `scripts/build/build_desktop.py` refuses an icon
  whose SHA-256 is not the one the manifest records, so a substituted picture cannot reach an
  installation.

## 15. The Desktop layout

Established by TASK-011 (Desktop UI contract, R18).

- **Every region is Theia's own.** The title row is Theia's title bar for a window without the
  system's frame, and the activity bar, the sidebars, the editor, the bottom panel and the
  status bar are its panels. `apps/desktop/src/shell/` only arranges them:
  - `title_bar`: the row, with the logo and name and the place of the sidebar toggles;
  - `panel_controls`: the two toggles, which run Theia's commands and show what the sidebars did;
  - `panel_layout`: where the regions stand, the share each takes, and a new profile's first layout;
  - `right_sidebar`: the right sidebar's row of text tabs, and its areas;
  - `panel_memory`: what the window remembers of its panels (section 16).
- **One thin row.** Whatever a later task adds to the title row goes into that row, left of the
  sidebar toggles, which stay immediately before the window controls. Nothing gets a row of its
  own above or below it, and the row keeps the height of its token.
- **Sizes and shares are design tokens** (section 14): pixels in `desktop`, and in
  `desktop_percent` the share each panel takes when it is first shown. `panel_layout` says of
  which space: the left sidebar's is of the window's width, the right one's of the width beside
  the left one, the bottom panel's of the height between the title row and the status bar.
- **A right-sidebar area is an ordinary view** of Theia's right side panel, which the widget
  factory `AREA_VIEWS` makes under the area's ID. The task that fills an area gives that ID its
  own view there and keeps the ID, so that a stored layout still finds the area. Until then the
  area says that it is not available and holds no control.
- **The end-to-end test checks the layout as the user sees it:** the places and order of the
  regions, their shares, the toggles and the areas. A task that changes the layout changes those
  checks in the same change.

## 16. What a client remembers

Established by TASK-012 (ST-VIEW, R13, C5).

- **View state lives with the host.** What a window remembers of its presentation is a typed,
  versioned record of `core/domain/client`, which the host keeps in the device's state store.
  The window gives it to the host and asks for it on its own channel; it never writes the store
  itself. The first such record is the panel layout: which of the left sidebar, the right
  sidebar and the bottom panel are shown, their sizes and their selected views.
- **Presentation only.** A view-state record names no target and carries no intent. The host
  checks its shape and bounds and reads nothing else into it, and nothing takes it as an input
  to authorization, binding or targeting (AD-06).
- **One layout per client for now.** Every window of a user is the client `desktop-window`.
  View state per workspace comes with TASK-016. That task also decides whether Theia's own
  stored layout, which holds the place of each view and the open editors, moves into the store.
- **Changes reach the host as they settle** (`apps/desktop/src/shell/panel_memory`). A change the
  host has not taken is kept in the window's profile and outranks the host's copy at the next
  start. A task that adds view state extends the module's sample and the record together, and
  the end-to-end checks with them.
- **A new record is a new step, and a new message a new version.** Adding a record to the state
  store, or changing one's version, is a step of the migration registry with its test
  (`core/state/migrations/`). A new message between a window and its host is a new protocol
  version (`core/protocol/version_negotiation`); two ends that agreed on an older version do
  without it.

## 17. Workspaces

Established by TASK-013 and TASK-014 (C1 to C4, B2, B5, B6, B18, R12, ST-WORKSPACE).

- **One registry per device.** A workspace is its record, which every participating device
  knows, and this device's replica with its root (`core/domain/workspace`). The registry
  (`core/workspaces/workspace_registry`) keeps both in the device's state store and writes them
  in one transaction. No other code creates, opens or closes a workspace.
- **IDs are given once.** A new workspace's ID comes from its creator, on the Desktop sixteen
  random bytes from the host; it is never derived from a name or a path and never reused.
- **A root is one workspace's.** A root is an existing folder, kept in its canonical form and
  at most 1024 bytes long. On its device it never is, contains or lies inside another
  workspace's root, compared by the directories' identity, so no spelling of a path, link or
  mount makes one workspace's files another's. A root that has since been moved or replaced
  by a link is not opened.
- **Open and close are the foreground.** Opening a workspace makes it the foreground workspace
  of the client that asked, and closing it leaves that client none. Neither binds an agent or
  changes another record (C2, R11): an agent names its workspace itself. A window's request
  names no client; the host keeps the foreground for the client on that channel.
- **Each window is a client.** Every Desktop window attaches over a channel of its own
  (`apps/desktop/src/main`), and the host gives each attached window the lowest number free:
  number 1 is the client `desktop-window`, number n the client `desktop-window-n`
  (`apps/desktop/src/application_host`). A window's foreground is its own, and when the window
  detaches its client shows none. The panel layout stays one for all of a user's windows, under
  `desktop-window`, until TASK-016.
- **The foreground decides the Explorer.** The Explorer of a Desktop window shows its client's
  foreground workspace and nothing else (`apps/desktop/src/workspaces/workspace_switcher`): a
  folder the window opens that is no workspace yet becomes a CODE workspace; a folder that
  cannot be one is closed, and the window says why; a window has one folder, and the commands
  that would add another are removed, so that two workspaces' trees are never one Explorer.
  Theia's own workspace shows the folder (reuse first). The title row names the workspace the
  host confirmed, never one the window only assumes.
- **A new workspace operation** goes into the registry with its tests, into the protocol as a
  new version, and into the switcher, with end-to-end checks of what the window then shows.

