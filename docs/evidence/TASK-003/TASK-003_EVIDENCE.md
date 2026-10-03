# TASK-003 evidence: Complete reuse, license, maintenance, and dependency feasibility audit

| | |
|---|---|
| Task | TASK-003 — Complete reuse, license, maintenance, and dependency feasibility audit |
| Date | 2026-10-03 |
| Performed by | Coding agent (Claude Code), one primary session; no sub-agents, reviewers or background AI jobs |
| Checkout | `/mnt/F/Nexees/`, branch `main`, base `d1a673f62d1a9a31839290e8041eb3c7eb91ba97` |
| Repository | `https://github.com/aivars11232/Nexees` |
| Specification pack | `/mnt/F/Nexees_LCL_Implementation_Pack_v0.5.3/`, archive revision 0.5.3, specification version 0.5.0 |
| Pack content identity | `bd5e8fce26f542c44a891e4340da3b2a84f79ab368b5cb868beb48126c2db3e1` |
| Pack manifest SHA-256 | `8d9d0080ac13a3fb17cc70603b77eb3b11abcd3fb6d2fad515cfac136fcf07e4` |
| Procedure | Continuation profile 0.5.3 (CA-01 to CA-12), as adopted for TASK-002 and continued for this task |
| Predecessors | TASK-001 (commit `284f97a`) and TASK-002 (commit `d1a673f`), both accepted |
| Commit of this work | Made after acceptance, as CA-06 permits; it is recorded by Git, not in this file |

## Status

The deliverables are complete and every applicable check passed on its final run.
Acceptance is recorded in [receipt/RECEIPT_RESULT.md](receipt/RECEIPT_RESULT.md), which is
written after this file because the receipt contains a snapshot of everything else.

TASK-003 classified 49 components and protocols, froze the initial dependency strategy,
and proved by prototype that one Rust core carrying the existing LCL engine runs on Desktop
and inside an Android 16 app: LCL validation, durable storage, ZIP import, an on-device
JavaScript test backend, passive offline help, and a mutually authenticated PC-to-phone
and phone-to-PC path. It also showed that Eclipse Theia on Electron builds and renders.
The owner answered DESKTOP-RENDERER-01: **Electron permitted**.

Every Android result comes from a disposable emulator, the owner's choice; **nothing here
is real-phone evidence**. Under CA-05 the routine corroboration was done by this session's
own local tools. All review here is the agent's own; no independent human or model review
has taken place.

## 1. Authority

The owner's instruction, verbatim:

```
Okay, proceed with task 3 now
```

This continues the procedure the owner adopted with pack 0.5.3 for TASK-002 (CA-01): one
named task through evidence, receipt acceptance, commit and push, then stop. TASK-004 is
not covered.

The owner answered four questions during the task. The questions and the chosen answers,
verbatim:

| Question | Answer |
|---|---|
| DESKTOP-RENDERER-01: may the installed Nexees Desktop app render its UI with Electron/Chromium, or must it be native only? | Electron permitted |
| For TASK-003's Android proofs (shared core and LCL running on Android, an on-device test backend, lifecycle and PC-to-phone checks), which device may I use? | Emulator only: "A disposable Android 16 (API 36) x86_64 emulator from the system image already in /mnt/F/.lcl-android/sdk, in its own AVD folder. Your phone is not touched; OEM-specific lifecycle behaviour stays unproven until a later real-device task." |
| The Android prototypes need tools that aren't installed: the Android NDK, a Rust toolchain with Android targets (rustup is absent) and Gradle/Maven build dependencies. Where may I install them? | Separate Nexees folder: "Download into a new /mnt/F/Nexees-toolchains/ outside the repo (roughly 5-8 GB): an isolated rustup toolchain and an NDK, used only through environment variables. System packages, ~/.cargo, ~/.gradle and the LCL SDK stay unchanged; the NDK reuses the Android SDK licence acceptance already recorded in your LCL SDK." |
| Two assigned scenarios touch your real desktop session: RC-T17 opens the installed HopToDesk (Flatpak) through an allowlisted launch, and RC-T04 tests start-at-login. How should I run them? | HopToDesk yes, login isolated: "Launch HopToDesk once, check its separate permission/setup state, then close it. Test the start-at-login registration only in an isolated profile; real login and logout tests stay with TASK-009." |

## 2. Checkout preflight (binding B26)

| Item | Observed |
|---|---|
| Working directory | `/mnt/F/Nexees/`; the Git worktree root, with no parent repository, superproject or second worktree |
| `origin` fetch and push | `https://github.com/aivars11232/Nexees.git`, an approved equivalent; no URL rewrite, no separate push URL |
| Branch and HEAD | `main` at `d1a673f`, equal to the local `origin/main`; upstream `origin/main` |
| Hooks | None besides Git's samples; `core.hooksPath` unset |
| At task start | Clean worktree right after the TASK-002 commit and push |
| Recheck at 22:20 | Unchanged identity; only this task's files modified or added; nothing staged, no stash |
| Result | Passes ([logs/preflight.txt](logs/preflight.txt)) |

## 3. Pack, native verification, dispatch and predecessors

**Pack.** All 201 manifest entries match their SHA-256 with no unlisted or missing file;
the content identity recomputes to `bd5e8fce…b3e1`; `tasks/task_003.lcl.txt`
(`d8a8fe49…5a12`) equals the `after_sha256` of `PROCEDURAL_CHANGES.json`, whose entry
confirms that only procedural fields changed from 0.5.2.

**Native verification.** The pack's runner, `tools/verify_with_lcl.py`, passed all 51
synthetic language cases with `lcl 0.9.1`, including `positive_task_003_language_test`
and `reject_renderer_unresolved`
([logs/native_language_suite.json](logs/native_language_suite.json)). These are language
tests, not evidence for any task.

**Dispatch.** `lcl run … --input input.task_number=3 main.lcl.txt` ran `task.dispatch` and
`task.task_003` only and published the TASK-003 packet with
`procedure_amendment_applies: TRUE` (SHA-256 of the packet text `e3b928ca…19aa`),
[logs/dispatch_task_003.record.json](logs/dispatch_task_003.record.json).

**Predecessors.** TASK-001's receipt and engine record (`bbda5d83…07e7`,
`status.succeeded`) and TASK-002's (`c5fadc83…bd13`, `status.succeeded`) are intact, both
commits are on `origin/main`, and `docs/charter/`, `docs/evidence/TASK-001/` and
`docs/evidence/TASK-002/` are unchanged. TASK-003 extends four `docs/architecture/` files
by design (section 6).

## 4. Required reading

The continuation profile and the TASK-003 record were read first; then the reuse
documents, the Android operating model, the remote-control rules and the acceptance
gates TASK-003 relies on. The rest of the mandatory read order was read in full for
TASK-002 earlier in this session and is byte-identical now, so it was reused, as the
profile allows ("reuse only verified unchanged reads").
[logs/required_reading.txt](logs/required_reading.txt) lists every file with its SHA-256,
and the public sources consulted. Only public package coordinates were sent to any
service.

## 5. What was decided

The machine-readable inventory is [docs/dependencies/](../../dependencies/dependencies.lcl.txt);
[LICENSE_MATRIX.md](../../dependencies/LICENSE_MATRIX.md) is the human-readable matrix. In
short:

- **One shared Rust core** for both hosts (rlib on Desktop, cdylib through JNI on
  Android), carrying the **existing LCL engine** at commit `fa1592b`, with SQLite
  (rusqlite, bundled), zip, Boa, pulldown-cmark, rustls with ring, rcgen and serde.
  Platform code is limited to adapters.
- **Desktop: Eclipse Theia 1.76.0 on Electron**, with five npm overrides: four move
  Electron, DOMPurify, uuid and `@tootallnate/once` to fixed versions, and one removes an
  unlicensed build package. Code-OSS is the fallback.
- **49 components**: 19 DEPENDENCY, 10 ADAPTER, 10 REFERENCE, 10 REJECTED, each with
  version, license and duties, advisory review, maintenance, transitive summary, platforms,
  need, alternatives, reason, removal path, verification and owner tasks.
- **Strategy DS-01 to DS-11**: exact pins, approved install scripts only, a license gate,
  an advisory gate, isolated builds, protocols before forks, capabilities before promises.
- **Ten open items** with their owner tasks, among them the owner's shipping terms for
  the LCL engine and manual, the auth hosting decision and the off-LAN route.

The architecture's open decisions now carry a status and resolution
(`docs/architecture/decisions.lcl.txt`): 6 resolved, 9 partly resolved with the
remaining part assigned, and PD-PROVIDER-ANDROID left open as not TASK-003's.

## 6. Deliverables

| File | Change | Why it is needed |
|---|---|---|
| `docs/dependencies/dependencies.lcl.txt` | New | Entry of the inventory project |
| `docs/dependencies/baseline.lcl.txt` | New | Audit identity, the owner's renderer answer, vocabularies and the component type (every gate item) |
| `docs/dependencies/components.lcl.txt` | New | The 49 component records |
| `docs/dependencies/strategy.lcl.txt` | New | Owner decisions, strategy DS-01 to DS-11, npm overrides, advisory dispositions, open items |
| `docs/dependencies/checks.lcl.txt` | New | 3 `VALIDATE` and 5 `VERIFY` checks |
| `docs/dependencies/record.lcl.txt` | New | Returns the inventory record |
| `docs/dependencies/LICENSE_MATRIX.md` | New | Human-readable license matrix (the gate asks for both forms) |
| `docs/architecture/baseline.lcl.txt` | Changed | `status` and `resolution` fields on each open decision |
| `docs/architecture/decisions.lcl.txt` | Changed | The 16 resolutions; SA-08 and RO-01 to RO-04 extended with TASK-003's findings |
| `docs/architecture/checks.lcl.txt` | Changed | `verify.arch_renderer_left_open` becomes `verify.arch_renderer_resolved`; new `verify.arch_open_decision_status` |
| `docs/architecture/record.lcl.txt` | Changed | Runs the new checks; its limits sentence now names where the answers come from |
| `docs/manuals/NEXEES_USER_MANUAL.md` | Changed | One sentence: the renderer choice is no longer open |
| `README.md` | Changed | The owner's answer, the inventory and the selected stack |
| `FILE_TREE.txt` | Changed | Inventory of the new files |
| `docs/evidence/TASK-003/` | New | This record, `check_dependencies.py`, the prototypes, logs and receipt |

The prototypes under `docs/evidence/TASK-003/prototypes/` are **evidence, not product
code**: they are kept so the logs can be reproduced. They are outside every product root
of the architecture, and no `*.source` placeholder was changed.

| Prototype | What it is |
|---|---|
| `feasibility-core/` | Rust crate `nexees-feasibility`: `lcl_bridge`, `store`, `import`, `js_backend`, `help`, `remote`, `probe`, `android` (JNI); binaries `nexees-probe` and `nexees-host` (Desktop receiver, peer-checked IPC, allowlisted launch, start-at-login); `Cargo.lock`, `deny.toml`, fixtures |
| `android-app/` | Kotlin app (no AndroidX): activity with explicit user controls, connectedDevice receiver service, launcher, notifications, opt-in boot receiver; `build.sh` |
| `harness/` | `device.py` (adb and uiautomator), `scenarios.py` (the RC scenarios), `advisories.py` (inventory and OSV lookup) |
| `theia/` | The evaluated Theia `package.json` and `package-lock.json`, `run_nested.sh`, `check_workbench.mjs` |

## 7. Results

### 7.1 Shared core: one probe, three runs

`nexees-probe` (Desktop and an Android executable) and the app's "Run probe" button (JNI)
execute the same 35 checks ([probe_desktop.json](logs/probe_desktop.json),
[probe_android_shell.json](logs/probe_android_shell.json),
[probe_android_app.json](logs/probe_android_app.json)):

| Group | Checks | Desktop x86_64 | Android executable | Android app, airplane mode |
|---|---|---|---|---|
| LCL engine: Core packages open against their trust anchors; valid task; invalid task (`error.reference.unresolved at 28:17`); four-file Core 0.3.0 project | 4 | pass | pass | pass |
| SQLite 3.53.2: v1 to v2 migration keeps data, WAL, integrity; rollback; abort mid-transaction | 3 | pass | pass | pass (crash check runs only as an executable; the app's kill and reboot tests cover it) |
| ZIP import: good project; traversal, absolute, backslash, symlink, duplicate, ratio bomb and LCL-invalid archives rejected; nothing left behind | 9 | pass | pass | pass |
| JavaScript backend: fail, repair, pass; endless loop stopped; no ambient API | 3 | pass | pass | pass |
| Help: hostile Markdown passive; all 24 LCL manual files render passive | 2 | pass | pass | pass |
| Transport and dispatcher over TLS: grant absent, granted write with hash and validation, replay, stale revision, expiry, wrong device, path escape, unpaired certificate, launch state, capabilities, revocation, oversized frame, receiver stop, no receiver | 14 | pass | pass | pass |

The file hashes and the manual's rendered size (377638 bytes) are identical on Desktop and
Android. The app's run had airplane mode on, with no route besides Android's inert
`dummy0`.

### 7.2 Android 16 emulator scenarios

Device: Android 16, API 36, `sdk_phone64_x86_64` userdebug build `BE2A.250530.026.D1`
(security patch 2025-07-05), emulator 37.1.11.0, in `/mnt/F/Nexees-toolchains/avd` with
its own adb server. [app_logcat.txt](logs/app_logcat.txt) holds the app's and Android's
activity log after the RC-T06 reboots, which cleared the earlier log; the earlier lines each
scenario relies on are kept in its record. UI actions are real taps found through
uiautomator, including the phone user's "Allow" in Android's own notification dialog;
nothing is driven behind the app's UI. The PC reaches the phone through the emulator's network redirect (bound to the
PC's 127.0.0.1), a LAN stand-in that airplane mode does cut. The pairing was a test
stand-in: the PC certificate was copied into the app's private folder with `run-as`
(TASK-060 owns real pairing). Final runs, [scenarios.jsonl](logs/scenarios.jsonl):

| Section | Result | Shows |
|---|---|---|
| setup | 5/5 | Fresh install; notifications allowed by the user; the in-app probe and a manual chapter work in airplane mode; receiver, PC grant and start-at-boot are off by default |
| t05 (RC-T05) | 11/11 | Grant absent: denied. After the phone user allows PC requests: create then edit an isolated LCL fixture, inspected on the phone (file hash and database revisions 1 and 2); stale edit conflict; an invalid edit saved as revision 3 with LCL's rejection reported; wrong device denied; unpaired PC rejected at TLS (UnknownCA); expired refused; after revocation denied, with saved revisions kept |
| capabilities (RC-14) | 2/2 | Per-action availability: write available, Settings and alarms need the user, maps and screen input unsupported; a maps launch returns unsupported |
| t07 (RC-T07) | 9/9 | Foreground launch completes. In the background Android blocked the start (`Background activity launch blocked! … callingUidProcState: FOREGROUND_SERVICE … BAL_BLOCK`, kept in the record's `os_block_log`); the request became needs_user_action with a notification; the notification opens a review screen only; Continue completes; expiry, revocation and Decline rechecked at Continue; locked: Android asks for the PIN first and the request stays pending until unlock |
| t06 (RC-T06) | 12/12 | Served in foreground, background (connectedDevice service type `0x10` with its notification), screen locked and forced deep Doze; network loss: unavailable, then served again; process killed: Android restarted the sticky service (new PID) and service resumed; force-stop: unavailable until the user starts it; reboot: unavailable by default, revisions and grant kept; opt-in start at boot: Android 16 allowed the connectedDevice start and the receiver served |

### 7.3 Desktop receiver, IPC, launch and startup

| Section | Result | Shows |
|---|---|---|
| pc, final run 21:18 | 10/10 | IPC folder 0700 and socket 0600; own user served, a mismatching uid denied (`peer uid 1000 … is not uid 4242`), messages over 4096 bytes refused, the PC cannot set the phone's grant; phone-to-PC denied while PC-to-phone works (directional grants); outside a graphical session the launch is unsupported; after the PC user revokes, denied; with no receiver, unavailable; no HopToDesk instance appeared |
| pc, run 21:13:54: the launch (now the once-only `pc_launch` section) | the launch happened; reply outcome_unknown | The one approved HopToDesk launch, from the phone's button through TLS, the shared dispatcher, the allowlist and `gio launch`: a new instance and window appeared. The launcher's first version held the receiver on a captured pipe, so the phone honestly reported outcome_unknown. Fixed and covered by a regression test (red before, green after); not relaunched, as the owner approved one launch. Only the new instance was closed. HopToDesk's own state: Flatpak X11 and PulseAudio sockets, no Wayland socket, no screencast or remote-desktop grant ([hoptodesk_launch.txt](logs/hoptodesk_launch.txt)) |
| startup (RC-T04), 21:18 | 6/6 | Isolated profile only: off by default; a valid XDG autostart entry; systemd's own generator turns it into a unit `PartOf` and `After` `graphical-session.target`; disable removes it; relative or shell commands refused; the real autostart folder, user units and linger unchanged |

Two earlier `pc` runs in the log are not counted: at 21:13:10 the harness took a stale
socket file for a ready receiver, so nothing was granted and nothing launched; the 21:13:54
run is the launch above, with the receiver defect that also made its revocation check fail.
Linger was already enabled on this account before TASK-003 and was never changed.

### 7.4 Desktop foundation

Theia 1.76.0 with Electron 42.8.1 installed, built (0 errors) and rendered the workbench:
Explorer, Search, Source Control, Debug, Extensions, Testing, Problems, Outline and a live
bash terminal. It did the same again with the overrides (Electron 42.11.10) and with the
final `unzipper` override, then quit cleanly
([theia_build.txt](logs/theia_build.txt), [theia_workbench.json](logs/theia_workbench.json)).
Chromium's headless Ozone platform never started a renderer with this Electron, so the app
ran in a nested KWin with a virtual framebuffer on a private D-Bus session and an isolated
HOME: nothing reached the owner's screen, session bus or keyring. npm 12 ran no install
script until approved; eight were approved and puppeteer denied.

### 7.5 Security, licenses and maintenance

- **Rust (216 crates):** cargo-deny 0.20.2 against RustSec advisory-db `ef6173cb`:
  advisories, bans, licenses and sources all OK; OSV: no advisory
  ([cargo_deny.txt](logs/cargo_deny.txt)).
- **npm as published (900 packages):** 34 advisories in 10 packages, including 4 high in
  Electron 42.8.1 and 2 critical in decompress 4.2.1.
- **npm evaluated (888 packages):** 9 advisories in 5 packages. Only decompress is
  runtime (Theia's .vsix unpacker), with no fix: extension installation stays disabled
  until TASK-021 (OI-04). The other four are build or test tooling. Dispositions:
  `data.dep_advisory_dispositions`.
- **Licenses:** every package in both trees has an identified license except the owner's
  LCL components, whose shipping terms the owner must record before any release (OI-01).
  `buffers` 0.1.1, unlicensed, was removed by the `unzipper` override.
- **Maintenance flags:** keytar is archived; Theia's tree has 13 packages its
  maintainers deprecated, 8 outside build tooling (the `xterm` 5.3.0 package and addons,
  two old `glob` versions, `inflight`, `nano`, `prebuild-install`), and a prerelease
  node-pty; Aider's activity has slowed; Electron 42 leaves upstream support when 45 ships
  (OI-05, OI-06).

## 8. Checks run

| Command | Result | Log |
|---|---|---|
| `lcl check`, `validate`, `run` of `docs/dependencies/dependencies.lcl.txt` | Exit 0 each; `status.succeeded` with all 5 `VERIFY` TRUE | [dependencies_lcl.txt](logs/dependencies_lcl.txt) |
| `python3 -B docs/evidence/TASK-003/check_dependencies.py` | 24 of 24 pass, exit 0 | [dependencies_lcl.txt](logs/dependencies_lcl.txt) |
| Negative tests: 11 seeded defects in scratch copies, plus 3 unmutated controls | All detected by the intended check; controls pass | [negative_tests.txt](logs/negative_tests.txt) |
| Rust: builds for x86_64 Linux, x86_64 and arm64 Android; clippy; 6 unit tests; cargo-deny | All exit 0; no clippy warning | [rust_checks.txt](logs/rust_checks.txt) |
| Probes (section 7.1) | 35/35 on each of three runs | the three probe logs |
| Android app build (Gradle) and package facts | Successful, no warnings; the APK's native library equals the final build byte for byte | [android_build.txt](logs/android_build.txt) |
| Scenarios (sections 7.2 and 7.3) | All final runs pass | [scenarios.jsonl](logs/scenarios.jsonl) |
| Theia build and workbench | Builds and renders | [theia_build.txt](logs/theia_build.txt) |
| Regression: architecture and charter `lcl` checks, TASK-002's and TASK-001's cross-checks | All pass | [regression.txt](logs/regression.txt) |
| Native language suite | 51/51 | [native_language_suite.json](logs/native_language_suite.json) |
| Toolchain provenance and isolation | Recorded, with one deviation (section 12) | [toolchains.txt](logs/toolchains.txt) |
| Secret and privacy scan of every added or changed file | No finding | [security_scan.txt](logs/security_scan.txt) |

## 9. Check accounting

### Verification

| Check | How it was met |
|---|---|
| T003.VERIFY.01 pinned-version and license evidence; standalone phone import/validation; passive offline help | Exact versions and licenses for all 49 components, the 216 crates and 888 npm packages; ZIP import with LCL validation and passive help ran inside the app in airplane mode (sections 7.1, 7.2). |
| T003.VERIFY.02 build the affected targets | The Rust crate for three targets, the APK, the Theia app, and both LCL projects (`lcl check` and `validate`). |
| T003.VERIFY.03 task-specific tests | Probes, scenarios, the workbench check, 6 unit tests, `check_dependencies.py` and the negative tests (section 8). |
| T003.VERIFY.04 regressions of the changed subsystem | The architecture's own checks and TASK-002's cross-check after the decision changes; the charter's checks and TASK-001's cross-check unchanged; the native suite. No product test suite exists yet. |
| T003.VERIFY.05 final diff inspected | Section 6 lists every changed and new file; no product placeholder changed; the receipt's final verification repeats the inspection. |
| T003.VERIFY.06 mobile feasibility build and run evidence for the shared runtime, LCL and a runnable fixture; no PC-only backend; no assumed LCL package | The LCL engine's source was read, built for Android and run on the device with its real Core packages; the fixture failed, was repaired and passed on the device with the network off. |

### Completion gate

| Check | How it was met |
|---|---|
| T003.CLOSE.01 dependencies closed | TASK-002 accepted and its lineage verified (section 3). |
| T003.CLOSE.02 objective without unrelated scope | Candidates evaluated and classified, the strategy frozen, rejected alternatives recorded; the prototypes only answer feasibility questions; nothing of TASK-004 or later was done. |
| T003.CLOSE.03 build passes | As T003.VERIFY.02. |
| T003.CLOSE.04 required tests pass | As T003.VERIFY.03; the failed intermediate runs are explained in sections 7.3 and 12. |
| T003.CLOSE.05 security checks pass | cargo-deny and OSV ran; every remaining advisory has a recorded disposition, and the one runtime advisory keeps its path disabled (OI-04); the secret scan is clean; no private key is in the repository; SA-08 is updated; the prototypes' security behaviour was tested (section 7). |
| T003.CLOSE.06 no unnecessary code or dependency | No product code or dependency was added. The prototypes are the smallest code that runs each candidate, and the inventory prefers smaller options (DEP-AMMONIA, DEP-AWS-LC-RS, DEP-UNIFFI and DEP-CARGO-NDK rejected). One script, because the engine cannot read lockfiles or registry data. |
| T003.CLOSE.07 no binding invariant violated | Bound checkout, branch and remote; the pack, the LCL repository and the canonical packages unchanged; one primary agent; the owner's receiver, startup, phone, firewall and router untouched; HopToDesk launched once as approved. One deviation: the LCL SDK's `.knownPackages` cache was rewritten (section 12). |
| T003.CLOSE.08 evidence recorded | This folder. |
| T003.CLOSE.09 Git diff understood | As T003.VERIFY.05. |
| T003.CLOSE.10 assigned standalone, remote and sync requirements; later scenarios traceable | A9's TASK-003 row: reusable Android runtime, LCL bridge, on-device code and test backend, and the Desktop toolkit after the owner's answer, all proven. RC and RC-T results below. Real-device repeats (TASK-066, TASK-074), real login tests (TASK-009) and the off-LAN route (TASK-060, TASK-070) are traced as open items, not waived. |
| T003.CLOSE.11 readability and comments | Agent's own review, section 10. |
| T003.CLOSE.12 scoped cleanup | Section 10. |
| T003.CLOSE.13 post-cleanup verification on the final revision | `receipt/final_verification.txt`. |
| T003.CLOSE.14 manual impact | Section 10. |

### Remote-control requirements and scenarios

| Check | How it was met | Limits and owners |
|---|---|---|
| T003.RC.05 startup and login distinct | User-level XDG autostart, off by default, lifetime of the graphical session (systemd's generator); no linger, system service or automatic login; outside a session the launch is unsupported | Real login, logout and lock tests: TASK-009. Linger was already on and left so |
| T003.RC.06 no receiver means unavailable | Unavailable with no PC receiver, after force-stop or reboot on the phone, during network loss, and in the probe | Nothing wakes or bypasses a device |
| T003.RC.11 app launching | Allowlist key to desktop-entry ID to `gio launch` with argv; unknown app denied (unit test); Android opens only listed public intents; nothing installs or extends the allowlist | — |
| T003.RC.12 session and launch truth | Graphical, active and unlocked session required; accepted, running, completed, failed, denied, unsupported, needs_user_action, expired and outcome_unknown are separate; the real launch reported outcome_unknown, never a window or screen control | The fixed launcher's full reply with HopToDesk: TASK-009 |
| T003.RC.14 other Android apps | Public intents with `<queries>`; per-action capabilities; unsupported and needs_user_action returned | Real phones: TASK-066 |
| T003.RC.15 Android lifecycle honesty | connectedDevice service with visible notification and Stop; RC-T06 results reported as observed | Real devices and OEM policies: TASK-057, TASK-066, TASK-074 |
| T003.RC.16 Android user action | Notification, then the phone user's explicit Continue or Decline in the app; a pending action is never success; the app taps nothing and uses no accessibility service | — |
| T003.RC.19 transport and private networking | rustls TLS 1.3, mutual authentication with pinned certificates, revocable grants; the PC receiver listened only on 127.0.0.1; no router or firewall change and no relay | The off-LAN route needs the owner's decision (OI-03); real pairing and revocation UX: TASK-060, TASK-070 |
| T003.RC.21 screen-control boundary | HopToDesk launched as an app only; its permissions read and left separate; no screen-control claim | — |
| T003.RC.23 secure service integration | SO_PEERCRED on the IPC socket, 0700/0600 modes, user privilege, 4 KiB IPC messages and 64 KiB frames, remote writes confined to workspace files, one dispatcher and grant check | The protocol-version refusal exists but no test sends a wrong version |
| T003.RC.25 one product, reused services | One `remote::dispatch` in the shared core serves both receivers, with shared request types, store and transport; only launch and capabilities are platform adapters | — |
| T003.RC.26 capabilities before promises | The Desktop receiver and launch integration and a PC-to-phone local action (LCL fixture edit) were proven; prerequisites and limits are recorded in sections 7.2 to 7.4 and the open items | Emulator only |
| T003.RCT.04 | Within the owner's limit: enable and disable in an isolated profile, login-start semantics shown by systemd's generator, no-GUI tested (unsupported), locked handled (unit test of the session judgement), no privileged setup, no fabricated window result | Login, restart and logout on the real session: TASK-009 (owner's decision) |
| T003.RCT.05 | Section 7.2, t05: 11/11 | Emulator |
| T003.RCT.06 | Section 7.2, t06: 12/12 | Emulator |
| T003.RCT.07 | Section 7.2, t07: 9/9 | Emulator |
| T003.RCT.17 | Section 7.3 and [hoptodesk_launch.txt](logs/hoptodesk_launch.txt) | One launch, as approved |

The receipt's remote-observation inputs stay `unverified`: the pack requires them only for
tasks 70 and 73 to 75, and they describe the product, which does not exist yet.

## 10. Review, cleanup and manuals

**Readability (agent's own review).** Each Rust module and Kotlin class starts with a
comment stating its job and its security property; comments explain why, not what. The
inventory's parts state their purpose in their `SPECIFICATION` descriptions. Review and
testing corrected, before the final logs: identical certificate subjects for every device
(now random), a launcher that held the receiver on a captured pipe (regression test), a
help check that would have counted escaped text as a tag, test edits that were not valid
LCL, deprecated Gradle APIs, buttons hidden under the status bar by Android 16's
edge-to-edge layout, and harness waits that trusted a stale socket file. No commented-out
code remains.

**Cleanup.** Inside the checkout, Gradle, Kotlin and cargo outputs were redirected to
`/mnt/F/Nexees-toolchains/`; the in-repo outputs of the first builds (`build/`,
`app/build/`, `.kotlin/`) were deleted; every Python script ran with `-B`. Outside it,
task-owned processes were stopped by exact match: the emulator and its adb server, Theia's
helpers, the stuck receiver and the HopToDesk instance this task started. Before the final
verification the disposable AVD (it held the app's test keys) and the PC test identities
in the session scratch folder were removed; the rest of that folder is removed after the
receipt. The receipt's final verification records the result. The toolchain folder stays,
because the owner approved it and later tasks need it; `rm -rf /mnt/F/Nexees-toolchains`
removes all of it. No pre-existing or user-owned file was deleted, reset or stashed.

**Manuals.** The Nexees manual draft said the renderer choice was still open; that
sentence now states the decision. Its statements about Android background limits,
directional grants and HopToDesk not establishing a screen session already match what was
observed. The LCL-in-Nexees draft needs no change: phone-local import and validation
without a PC are what the prototype showed.

## 11. Policy review and disclosures

All 135 rules of the nine policy documents were reviewed for this task.

| Policy document | Rules | Relation to TASK-003 |
|---|---|---|
| `policies/master_rules.lcl.txt` | 32 | Conduct rules followed; reuse-first and evidence-before-claims shaped the audit; no product feature claimed |
| `policies/global_contracts.lcl.txt` | 31 | C20 resolved by the owner's answer, never inferred; the security and portability contracts shaped the selection |
| `policies/acceptance_criteria.lcl.txt` | 11 | Product acceptance, none due now; feasibility evidence feeds the owning tasks |
| `policies/code_readability_and_cleanup.lcl.txt` | 6 | Applied to the prototypes and scripts; section 10 |
| `policies/no_unnecessary_code.lcl.txt` | 7 | No product code or dependency; minimal prototypes; smaller options preferred |
| `policies/reuse_policy.lcl.txt` | 5 | Applied: every candidate inspected and classified before use; LCL reused, not reimplemented |
| `policies/security_baseline.lcl.txt` | 9 | Explicit assumptions; fail-closed transport and grants; no secret handled or committed; advisory gate |
| `policies/usage_and_agents.lcl.txt` | 8 | One agent; one heavy process at a time (emulator stopped before the Theia build); Git writes only as CA-06 permits |
| `architecture/remote_device_control.lcl.txt` | 26 | The twelve assigned requirements and five scenarios tested within the owner's limits; nothing of the owner's enabled |

Disclosures:

- **Network.** Public registries, GitHub metadata, OSV (package@version only), the RustSec
  database, Gradle and Maven repositories, Electron and Node header downloads. No project
  code or personal data was sent; `npm audit` was not used because it uploads the
  dependency tree (CA-07).
- **Session hook.** A hook asks for a dynamic web-application security scan after code
  changes. There is no running Nexees web application to scan and `HAWK_API_KEY` is unset,
  so none was run; the prototypes' security behaviour was tested directly.
- **Concurrent workload.** Another tool's nested Plasma session (`archdock-plasma-lifecycle`)
  ran on this machine during the task; it was not touched.
- **Owner-only state observed.** HopToDesk was already running, and linger was already
  enabled; both were left as found.

## 12. Deviations and findings

- **LCL SDK cache rewritten.** `avdmanager`, run with the LCL SDK as its root to create the
  AVD, rewrote `/mnt/F/.lcl-android/sdk/.knownPackages` (16 bytes, a package-list cache,
  first created 2026-09-25). No package changed, but its earlier bytes were not kept, so
  identity cannot be shown. A later AVD should be written without running SDK tools against
  the LCL SDK.
- **Receiver defect found by the real launch** (section 7.3), fixed with a regression test.
- **Harness defects** (stale socket, emulator lock-screen state, edge-to-edge layout) caused
  failed intermediate runs; each final run is complete and passing.
- **Electron headless mode** does not start a renderer here; the nested-session method is
  recorded for later tasks.
- **Canonical Core 0.1.0** has eight directories with 1901 timestamps that Gradle refuses;
  the APK build copies the packages (verified identical) and leaves the canonical folder
  untouched.

## 13. Open items and limits

The ten open items OI-01 to OI-10 are in `docs/dependencies/strategy.lcl.txt` with their
owner tasks. Beyond them:

- **Emulator only.** No result here is real-phone evidence, and the arm64 build was compiled
  but not run.
- **Prototypes are not product code.** TASK-005 creates the real scaffold; nothing in
  `apps/`, `core/` or `platform/` was implemented.
- **The off-LAN route, the auth backend and the LCL shipping terms** wait for the owner.
- BIND-B1-LOGO and AMEND-075-01 remain as the charter records them.

## 14. Next task

TASK-004 (threat model). It was not started.
