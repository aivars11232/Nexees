# Nexees

**Application-only source layout — pre-implementation, structure revision 0.4.**

This repository contains the planned Nexees application folders and named files, the
frozen project charter, the modular architecture, the dependency inventory, the threat
model and the evidence of completed implementation tasks. It does not contain the implementation task pack, numbered task
documents or task-to-path maps.
Keep the implementation pack separate; the charter refers to it by identity.

## Authoritative requirements

The single written source of truth for Nexees is the LCL implementation pack, kept
outside this repository. [docs/charter/](docs/charter/) freezes one exact revision of
it by identity and consolidates the agreed scope, the 75-task execution model and the
requirement-to-task traceability. Where the charter and the pack differ, the pack
governs.

The charter is an LCL Core 0.3.0 project whose entry is `docs/charter/charter.lcl.txt`.
Check, validate and run it with the installed `lcl` and the canonical Core packages. A
successful run shows that the record is structurally consistent, not that any feature
exists.

The charter also records what was open when it was frozen: the Desktop renderer
decision, the Review/Supervisor amendment deferred to TASK-075, and the logo source
path, which did not resolve. TASK-003 obtained the owner's renderer answer, **Electron
permitted**, and TASK-010 the owner's answer on the logo source. Both are recorded in
[docs/dependencies/](docs/dependencies/); the charter itself stays frozen.

[docs/architecture/](docs/architecture/) defines the modular architecture (TASK-002): the
subsystems and the module files each one owns, shared versus client-specific code, the
Desktop and Android local hosts, interfaces, which state is authoritative and who writes
it, data and authority flows, failure domains, and the portability decisions, each with its
status and the resolution the deciding task recorded. It is an LCL Core 0.3.0 project
whose entry is `docs/architecture/architecture.lcl.txt`; it refines the pack, and where
they differ the pack governs. The architecture itself selects no technology, dependency
or renderer.

[docs/dependencies/](docs/dependencies/) is the dependency inventory (TASK-003): every
evaluated component and protocol, classified DEPENDENCY, ADAPTER, REFERENCE or REJECTED,
with its exact version, license and duties, advisory review, maintenance status and
removal path, together with the initial dependency strategy, the owner decisions it rests
on and the open items. Its entry is `docs/dependencies/dependencies.lcl.txt`, and
[LICENSE_MATRIX.md](docs/dependencies/LICENSE_MATRIX.md) is the human-readable matrix.
The feasibility prototypes behind it are evidence under
[docs/evidence/TASK-003/](docs/evidence/TASK-003/), not product code.

[docs/security/](docs/security/) is the threat model (TASK-004). It applies the pack's
eight-level trust hierarchy to Nexees and records the separate authorities, the trust
boundaries, assets and attack surfaces, and 30 immutable security invariants. It lists
threats across Desktop, Android, agent tools, providers, LCL, extensions, auth, updates and
cross-device sessions. Each threat is traced to the later tasks that own it and to the
architecture subsystems and module slots that will enforce it, with planned threat tests.
It also holds TASK-004's security contracts for its remote-control requirements. Its entry
is `docs/security/security.lcl.txt`. It is a design: nothing in it is implemented or tested
yet.

[docs/engineering/CONVENTIONS.md](docs/engineering/CONVENTIONS.md) holds the engineering
conventions (TASK-005). It covers:
- where things live, and how a placeholder becomes source;
- Rust, Kotlin and TypeScript rules;
- readable code and comments;
- dependencies, generated artifacts and secrets;
- tests, task evidence with its closure record, and scoped cleanup;
- the checks, CI and the manual sources.

`scripts/test/run_checks.py` runs every repository check, and `scripts/ci/continuous_integration.sh`
is the entry point for a CI service. No hosted CI is configured.

## Source-file status

Every `*.source` file is an **empty named placeholder**, not executable source or a
completed feature. `*.test.source` files reserve test names; no product test is
implemented or claimed to pass. The filename stems describe the planned modules.

TASK-003 selected the stack, with prototype evidence and the owner's renderer decision: a
shared Rust core used by both hosts (on Android through JNI, with Kotlin for the
platform layer) and Eclipse Theia on Electron for the Desktop client.

TASK-005 created the scaffold:
- the Rust workspace and its pinned toolchain;
- the first crate, `core/domain`;
- the repository checks and their tests.

TASK-006 implemented the shared domain model in `core/domain`. It defines the versioned,
strictly validated types both hosts share:
- workspaces and their device-local replicas;
- devices and their trust;
- agent sessions and their bindings;
- tasks and evidence;
- LCL revisions, imports and model handoffs;
- grants, approvals and the remote request envelope.

The crate holds no behaviour, storage or transport. Those arrive with the tasks that own them.

TASK-007 defined the protocol in `core/protocol`: what travels between a window and its host,
and between two paired hosts.
- Every channel opens by agreeing on a protocol version, and refuses a mismatch rather than
  downgrading.
- Messages are bounded and strictly decoded.
- Requests name their operation from a closed registry, and their targets explicitly.
- Remote control and content synchronization are separate message families.
- A request is never run twice: duplicates get the recorded outcome, and after a lost
  acknowledgment the sender reconciles by request ID.

It defines the contracts and their rules; transport, storage and execution come with later tasks.

TASK-008 built the state store in `core/state`: each host's one transactional store, SQLite
bundled through rusqlite.
- Writes are durable before they are reported saved.
- Every record is versioned and decoded strictly when it is read.
- Migrations apply all at once or not at all, and may not change a record they do not declare.
- The operation journal turns an interrupted effect into an unknown outcome instead of replaying
  it.
- The outbox keeps unsynced content changes, and nothing else, until the peer acknowledges them.

Checkpoints, the project-state entities and crash-safe resume come with their own tasks.

TASK-009 built the first Desktop shell: an installed application with its own window, on
Eclipse Theia and Electron.
- The window runs no executor. It attaches to the Nexees host (`nexees-host`), a separate
  process that runs once per user and holds the device's state.
- Window and host talk over a Unix socket private to the user, with the peer's user checked
  by the kernel and every message bounded.
- The host stops after its last window closes, unless the user enables start at login, which
  keeps it running.
- Extensions stay disabled until their confinement is decided.
- The window's backend serves only the window that started it, its page runs in a sandboxed
  renderer without Node, and the window connects to nothing beyond this machine.

`scripts/build/build_desktop.py` checks, builds and installs it outside the checkout;
`tests/e2e/desktop/local_application.test.mjs` tests the installed application.

TASK-010 gave Nexees its look.
- [assets/theme/](assets/theme/) holds the shared design tokens and the icon mapping: the one
  source of the colours, sizes and icons of both clients, taken from the approved compact
  layout. `scripts/build/design_tokens.py` checks them, including the contrast that text must
  keep, and writes each client's copy.
- The Desktop window opens in the Nexees dark theme with compact, VS Code-like sizes; Theia's
  other themes stay available. Its About dialog shows the logo.
- The logo reaches the product only as the bound source and icons scaled down from it
  (`scripts/build/brand_assets.py`); an installation carries the icon in seven sizes.

TASK-011 gave the window the approved layout.
- One thin title row replaces the system's title bar. It holds the logo and name, the menu, the
  window's title, the two sidebar toggles and, immediately after them, the window controls.
- Under it stand the activity bar and the left sidebar, the editor and the right sidebar over the
  bottom panel, and the status bar. A new profile shows them in the shares measured on the
  approved layout, which are design tokens too.
- The right sidebar shows its five areas as a row of text tabs: AI Agent, LCL, Tasks, Chat and
  Logs. No task has filled an area yet, and each says so.
- Every region is Theia's own panel, arranged in [apps/desktop/src/shell/](apps/desktop/src/shell/).

TASK-012 made the window remember its panels.
- Which of the left sidebar, the right sidebar and the bottom panel are shown, their sizes and
  their selected views are the window's view state. The window gives it to the Nexees host,
  which keeps it in the device's state store, and gets it back when it starts
  ([apps/desktop/src/shell/panel_memory.ts](apps/desktop/src/shell/panel_memory.ts)).
- So the panels come back as the user left them: after the window was closed, after it was
  stopped unexpectedly, and on a profile in which Theia has stored nothing.
- The layout is presentation only. It is a typed record of the shared domain model
  (`core/domain/client.rs`), carried by two new messages of the window's channel, which made
  protocol version 2 and schema version 2 of the state store.
- It is one layout for the window. What a client remembers per workspace comes with the
  workspaces (TASK-013 to TASK-016).

A placeholder becomes source when its owning task implements it, under the same stem with
its language's extension.
This is a complete inventory of this planned layout, not a prediction of every file
that upstream frameworks or implementation will eventually generate.

## Product layout

| Folder | Product responsibility |
|---|---|
| `apps/desktop/` | Installed Desktop application, window, compact panels, workspace views and settings. |
| `apps/android/` | Matching responsive Android application, independently usable without a PC. |
| `core/` | Shared workspace, agent, tools, security, task execution, state, providers, LCL, sync, remote control and help. |
| `platform/` | Device-specific execution, lifecycle, scoped storage, secrets and transport adapters. |
| `integrations/` | Reusable IDE, LCL, auth, provider, external-agent, Git and syntax-index adapters. |
| `assets/` | The approved visual references, the logo source with its derived icons, and the design tokens and icon mapping both clients share. |
| `config/` | Configuration slots and the existing non-secret local-path example. |
| `docs/charter/` | Frozen project charter as an LCL project: scope, traceability, open decisions and structural checks. |
| `docs/architecture/` | Modular architecture as an LCL project: subsystems and module ownership, hosts, interfaces, authoritative state, flows, failure domains and decisions. |
| `docs/dependencies/` | Dependency inventory as an LCL project, the license matrix and the initial dependency strategy. |
| `docs/security/` | Threat model as an LCL project: trust hierarchy, boundaries, attack surfaces, security invariants, threats, threat tests and remote security contracts. |
| `docs/engineering/` | Engineering conventions: layout, languages, readability, formatting, dependencies, tests, evidence, cleanup, checks and CI. |
| `docs/evidence/` | Evidence of completed implementation tasks, one folder per task. |
| `docs/manuals/` | Nexees and integrated-LCL manual drafts, retained from the previous pack. |
| `tests/` | Named unit, integration, conformance, security and end-to-end test slots, and the repository tooling tests in `tests/tooling/`. |
| `scripts/` | The check runner, the scoped cleanup tool and the CI entry point, and the build, manual-bundling and release helper slots. |
| `packaging/` | Desktop, Android and update packaging slots; no signing secrets. |

Every product file, meaning everything under `apps/`, `core/`, `platform/`,
`integrations/`, `assets/`, `config/`, `packaging/`, `scripts/` and `docs/manuals/`,
belongs to exactly one subsystem in `docs/architecture/subsystems.lcl.txt`, and each
planned test file is attributed to the subsystem it proves. A task that adds, moves or
merges a file updates that catalogue in the same change.

`core/tasking/` and the task-panel files are **Nexees runtime functionality**, not tasks
for building Nexees. Similarly, the LCL adapter integrates the existing core rather
than containing another parser/validator implementation. Adapters are boundaries for
reuse, not a requirement to reimplement libraries. Consolidate slots where the chosen
reusable component genuinely supplies the same responsibility.

## Existing content retained

The manual files remain explicitly marked as unverified, pre-implementation drafts.
They are not published instructions for a working release. The model-orientation
prompt is a template; its fields must be bound to the verified session/protocol schema.
The runtime must enforce read-only access independently of what a prompt says.

The approved logo and Desktop-layout references are unchanged. The pack binds the logo to
`/home/aivars/Pictures/Nexees Logo/Icon.png`, a path that does not resolve on the
development machine (charter item BIND-B1-LOGO). In TASK-010 the owner confirmed that the
logo is the file in that Pictures folder whose name is `Nexees Logo⁄Icon.png`, with the
character U+2044 where the binding writes a folder separator. `scripts/build/brand_assets.py`
reads only that file, and never changes it. It copied the file byte for byte to
`assets/branding/source/` and derived the application's icon sizes into
`assets/branding/derived/`; `assets/branding/manifest.json` records them. The chat reference
remains a reference only: nothing is derived from it.

## Inventory

[FILE_TREE.txt](FILE_TREE.txt) lists every actual file and directory in this archive.
No application build, compiled artifact, fake signing key or user workspace data is
included. The Rust workspace's `Cargo.lock` is committed, because DS-05 locks every
dependency. Source files and their build metadata arrive with the tasks that implement
them. Regenerate the inventory with `python3 -B scripts/test/run_checks.py --write-file-tree`.
