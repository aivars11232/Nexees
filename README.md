# Nexees

**Application-only source layout — pre-implementation, structure revision 0.4.**

This repository contains the planned Nexees application folders and named files, the
frozen project charter and the evidence of completed implementation tasks. It does not
contain the implementation task pack, numbered task documents or task-to-path maps.
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

The charter also records what is still open: the Desktop renderer decision, the
Review/Supervisor amendment deferred to TASK-075, and the logo source path, which does
not currently resolve.

## Source-file status

Every `*.source` file is an **empty named placeholder**, not executable source or a
completed feature. `*.test.source` files reserve test names; no tests are implemented
or claimed to pass. The filename stems describe the planned modules.

The Desktop renderer, implementation languages and build system are still unconfirmed.
The `.source` suffix is deliberately temporary: choose the approved stack first, then
use its actual language extensions and build files. No Electron, WebView, native GUI,
Kotlin, TypeScript, Rust or other stack has been silently selected by this archive.
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
| `assets/` | Existing approved visual references, future verified icon imports and shared visual resources. |
| `config/` | Configuration slots and the existing non-secret local-path example. |
| `docs/charter/` | Frozen project charter as an LCL project: scope, traceability, open decisions and structural checks. |
| `docs/evidence/` | Evidence of completed implementation tasks, one folder per task. |
| `docs/manuals/` | Nexees and integrated-LCL manual drafts, retained from the previous pack. |
| `tests/` | Named unit, integration, conformance, security and end-to-end test slots. |
| `scripts/` | Build, check, manual-bundling, CI and release helper slots. |
| `packaging/` | Desktop, Android and update packaging slots; no signing secrets. |

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

Existing approved logo and Desktop-layout references are unchanged. The development
machine logo binding remains exactly `/home/aivars/Pictures/Nexees Logo/Icon.png`.
That local file was not accessed when this layout was created, and TASK-001 found that
the path does not resolve on the development machine (charter item BIND-B1-LOGO). The
chat reference is not a silent replacement for the owner-bound build source. Source/derived icon folders remain empty until the
real source is verified and the required assets are produced.

## Inventory

[FILE_TREE.txt](FILE_TREE.txt) lists every actual file and directory in this archive.
No application build, dependency lockfile, compiled artifact, fake signing key or
user workspace data is included. Source files and necessary upstream build metadata
will be implemented after the unresolved platform choices are settled.
