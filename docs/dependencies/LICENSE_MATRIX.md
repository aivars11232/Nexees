# Nexees dependency and license matrix

Initial dependency strategy frozen by TASK-003 on 2026-10-03. This is the human-readable
companion of the machine-readable inventory in [`dependencies.lcl.txt`](dependencies.lcl.txt),
which holds one record per component with every field the pack's license and dependency
gate requires. When the two disagree, the inventory is the authority and this page is the
defect. Evidence: [`docs/evidence/TASK-003/`](../evidence/TASK-003/TASK-003_EVIDENCE.md).

**Classifications.** DEPENDENCY: code Nexees ships or links. ADAPTER: a program, service,
protocol or platform API used through a Nexees adapter. REFERENCE: studied, not shipped.
REJECTED: not used, for the reason given.

**Proven** means the TASK-003 prototype actually ran the component: on Desktop (x86_64
Linux), on Android 16 (a disposable x86_64 emulator), or both. Nothing here is proven on a
real phone.

## Owner decisions

| Decision | Answer |
|---|---|
| DESKTOP-RENDERER-01 | **electron_permitted**: the Desktop app may render with Electron/Chromium |
| Android test device | Emulator only; the owner's phone was not touched |
| Toolchains | Isolated in `/mnt/F/Nexees-toolchains/`, outside the repository |
| Desktop session tests | HopToDesk launched once; start-at-login tested in an isolated profile only |

## Strategy in brief

1. **One shared Rust core** for Desktop and Android, with thin platform adapters (Kotlin
   for Android lifecycle and UI, Linux adapters for IPC, launch and startup).
2. **The existing LCL engine**, linked at one pinned commit, with the Core packages
   shipped and verified on every device; never a second parser.
3. **Desktop foundation: Eclipse Theia 1.76.0 on Electron**, kept inside the IDE
   subsystem so it stays replaceable; Code-OSS is the fallback.
4. **Exact pins** (Cargo `=` versions plus `Cargo.lock`; npm exact versions plus
   `package-lock.json` and security overrides). npm install scripts run only when approved.
5. **License gate.** Permissive licenses pass; EPL-2.0, LGPL-2.1+, MPL-2.0, CC-BY-4.0 and
   OFL-1.1 pass only with their recorded duties; no license blocks shipping.
6. **Advisory gate.** Every change runs cargo-deny (RustSec) and an OSV lookup. A runtime
   high or critical advisory blocks the change unless it is fixed by a pin or its path is
   disabled with a recorded disposition.
7. **Capabilities, not promises.** A component unproven on a host is a reported gap there.

## Matrix

| ID | Component | Version | License | Class | Scope | Status |
|---|---|---|---|---|---|---|
| DEP-THEIA | Eclipse Theia | 1.76.0 + 5 overrides | EPL-2.0 OR GPL-2.0 w/ Classpath exc. | DEPENDENCY | desktop | Proven on Desktop |
| DEP-ELECTRON | Electron | 42.11.10 (Theia pins 42.8.1) | MIT (+ Chromium notices) | DEPENDENCY | desktop | Proven on Desktop |
| DEP-CODE-OSS | Code - OSS | reference | MIT | REFERENCE | none | Fallback foundation |
| DEP-VSCODIUM | VSCodium | reference | MIT | REFERENCE | none | Branding/marketplace patterns |
| DEP-MONACO | Monaco (@theia/monaco-editor-core) | 1.108.201 | MIT | DEPENDENCY | desktop | Inherited; rendered |
| DEP-OPEN-VSX | Open VSX via @theia/vsx-registry | 1.76.0 client | EPL-2.0 | ADAPTER | service | View rendered; no install |
| DEP-VS-MARKETPLACE | VS Code Marketplace | n/a | Proprietary terms | REJECTED | none | Terms limit use to Microsoft products |
| DEP-LSP | Language Server Protocol | 3.17 (protocol lib 3.17.5) | CC-BY-4.0 spec, MIT libs | ADAPTER | desktop | Inherited; no server run |
| DEP-TREE-SITTER | Tree-sitter | 0.27.0 | MIT | DEPENDENCY | shared core | Unproven (TASK-039) |
| DEP-XTERM | xterm.js (legacy `xterm`) | 5.3.0 | MIT | DEPENDENCY | desktop | Proven (bash terminal); deprecated name |
| DEP-NODE-PTY | node-pty | 1.2.0-beta.12 | MIT | DEPENDENCY | desktop | Proven; prerelease |
| DEP-GIT-CLI | Git CLI | user's install | GPL-2.0-only (separate program) | ADAPTER | desktop | Not exercised (TASK-020) |
| DEP-LIBGIT2 | libgit2 / git2 | 0.21.0 | GPL-2.0 w/ linking exc.; MIT/Apache | REJECTED | none | CLI preferred; gix for Android |
| DEP-GIX | gitoxide | 0.88.0 | MIT OR Apache-2.0 | REFERENCE | none | Android Git candidate, unproven |
| DEP-ACP | Agent Client Protocol SDK | 1.7.0 | Apache-2.0 | ADAPTER | desktop | Not prototyped |
| DEP-OPENHANDS | OpenHands | reference | MIT | REFERENCE | none | Python; patterns only |
| DEP-CLINE | Cline | reference | Apache-2.0 | REFERENCE | none | Patterns only |
| DEP-AIDER | Aider | reference | Apache-2.0 | REFERENCE | none | Activity slowing (last push 2026-05-22) |
| DEP-LITELLM | LiteLLM | n/a | MIT core + enterprise terms | REJECTED | none | Python service; not phone-local |
| DEP-SUPABASE-AUTH | Supabase Auth | clients 2.117.2 | MIT | ADAPTER | service | Needs owner hosting/cost decision |
| DEP-KEYRING | keyring (Secret Service) | 4.2.0 | MIT OR Apache-2.0 | ADAPTER | desktop | Unproven (TASK-025) |
| DEP-ANDROID-KEYSTORE | Android Keystore | platform | Platform API | ADAPTER | android | Unproven (TASK-025) |
| DEP-KEYTAR | keytar (inside Theia) | 7.9.0 | MIT | REJECTED | desktop | Archived upstream since 2022-12 |
| DEP-BUBBLEWRAP | bubblewrap | 0.13.0 (installed) | LGPL-2.0-or-later (separate program) | ADAPTER | desktop | Not exercised (TASK-024) |
| DEP-LCL-ENGINE | LCL engine + Core 0.1.0/0.3.0 | commit fa1592b | No license file (owner's) | DEPENDENCY | shared core | Proven on Desktop and Android |
| DEP-LCL-MANUAL | LCL user manual | commit fa1592b | No license file (owner's) | DEPENDENCY | shared core | Rendered on both |
| DEP-LCL-ANDROID-APP | LCL for Android | commit fa1592b | No license file (owner's) | REFERENCE | none | PC-authority client only |
| DEP-LCL-REMOTE | lcl-remote | commit fa1592b | No license file (owner's) | REFERENCE | none | Listens on all interfaces |
| DEP-RUST | Rust toolchain | 1.99.0 | MIT OR Apache-2.0 | DEPENDENCY | build | Proven; arm64 compiled |
| DEP-SQLITE | SQLite via rusqlite (bundled) | 0.40.2 / SQLite 3.53.2 | MIT; SQLite public domain | DEPENDENCY | shared core | Proven on both; pinned in the workspace by TASK-008 for the state store |
| DEP-ZIP | zip (zip2) + flate2/zlib-rs | 8.6.0 | MIT (Zlib, MIT/Apache) | DEPENDENCY | shared core | Proven on both |
| DEP-BOA | Boa JavaScript engine | 0.22.0 | Unlicense OR MIT | DEPENDENCY | shared core | Proven on both (fail → repair → pass) |
| DEP-PULLDOWN-CMARK | pulldown-cmark | 0.13.4 | MIT | DEPENDENCY | shared core | Proven on both |
| DEP-RUSTLS | rustls + ring + webpki | 0.23.45 / 0.17.14 | Apache-2.0 OR ISC OR MIT; ring Apache-2.0 AND ISC | DEPENDENCY | shared core | Proven on both and PC↔emulator |
| DEP-RCGEN | rcgen | 0.14.10 | MIT OR Apache-2.0 | DEPENDENCY | shared core | Proven on both |
| DEP-SERDE | serde / serde_json | 1.0.229 / 1.0.151 | MIT OR Apache-2.0 | DEPENDENCY | shared core | Proven on both; pinned in the workspace by TASK-006; the protocol's wire format since TASK-007 |
| DEP-NIX | nix | 0.31.3 | MIT | DEPENDENCY | desktop | Proven (peer uid checks) |
| DEP-JNI | jni | 0.22.4 | MIT OR Apache-2.0 | DEPENDENCY | android | Proven in the app |
| DEP-UNIFFI | UniFFI | 0.32.2 | MPL-2.0 | REJECTED | none | One JSON entry point suffices |
| DEP-AMMONIA | ammonia | 4.2.1 | MIT OR Apache-2.0 | REJECTED | none | Parse-time filtering suffices |
| DEP-RQUICKJS | rquickjs | 0.14.0 | MIT | REJECTED | none | C engine; Boa chosen |
| DEP-MLUA | mlua | 0.12.2 | MIT | REJECTED | none | No Lua requirement |
| DEP-WASMI | wasmi | 2.0.0 | MIT OR Apache-2.0 | REFERENCE | none | Future WebAssembly test backend |
| DEP-AWS-LC-RS | aws-lc-rs provider | not added | ISC/Apache-2.0 + OpenSSL terms | REJECTED | none | Heavy C build; ring suffices |
| DEP-CARGO-NDK | cargo-ndk | 4.1.2 | Apache-2.0 OR MIT | REJECTED | none | Plain cargo + NDK clang suffices |
| DEP-ANDROID-BUILD | Gradle / AGP / SDK / NDK | 9.8.0 / 9.4.1 / 36 / r30 | Apache-2.0; Android SDK License | DEPENDENCY | build | Proven (APK built) |
| DEP-GIO | GIO `gio launch` | installed GLib | LGPL-2.1-or-later (separate program) | ADAPTER | desktop | Proven (one HopToDesk launch) |
| DEP-XDG-AUTOSTART | XDG autostart + systemd generator | spec / installed systemd | spec; LGPL-2.1-or-later | ADAPTER | desktop | Proven in an isolated profile |
| DEP-OFF-LAN-ROUTE | WireGuard / Tailscale / Headscale | not selected | GPL-2.0, MIT, BSD-3-Clause | REFERENCE | none | Owner decision (TASK-060/070) |

Totals: 49 components; 19 DEPENDENCY, 10 ADAPTER, 10 REFERENCE, 10 REJECTED.

## Licenses that carry duties

| Where | License | Duty |
|---|---|---|
| Theia (@theia/* packages) | EPL-2.0 (or GPL-2.0 with Classpath exception) | Keep notices; offer source for any modified Theia file |
| jschardet 2.3.0 (in Theia) | LGPL-2.1-or-later | Ship its license and source; keep it a separately replaceable module |
| @vscode/codicons 0.0.45 | CC-BY-4.0 (icons), MIT (code) | Attribution |
| font-awesome 4.7.0 | OFL-1.1 (font), MIT (CSS) | Keep the font license; do not sell the font on its own |
| argparse 2.0.1 | Python-2.0 | Keep the notice |
| dompurify 3.4.16 | MPL-2.0 OR Apache-2.0 | Choose Apache-2.0 |
| Electron | MIT + Chromium's bundle | Ship `LICENSE` and `LICENSES.chromium.html` |
| ring | Apache-2.0 AND ISC | Keep both notices |
| LCL engine, Core packages, manual | none recorded | **The owner must record shipping terms before any release** |

The evaluated Theia tree has 888 npm packages: MIT 680, ISC 53, Apache-2.0 50, the EPL-2.0
dual license 46, BSD 33, BlueOak-1.0.0 9, and single packages under the other terms above.
Four packages declare their license only in an old `licenses` list (busboy, fuzzy,
streamsearch, xmlhttprequest-ssl: all MIT). `union` 0.5.0 (build tooling) declares nothing but
ships an MIT license file. As published, Theia's ffmpeg tool also pulled in `buffers` 0.1.1,
which has no license anywhere (its repository is gone); the `unzipper` override removed it.
The Rust tree has 216 crates, all under the allowlist enforced by cargo-deny.

## Security overrides and advisories

npm overrides applied on top of Theia 1.76.0 (the app built and rendered with them):

| Package | From → to | Why |
|---|---|---|
| electron | 42.8.1 → 42.11.10 | 4 high advisories (fixed from 42.9.2) |
| dompurify | 3.2.7 → 3.4.16 | 18 advisories in Monaco's copy |
| uuid | 7.0.3, 8.3.2 → 11.1.1 | GHSA-w5hq-g745-h8pq |
| @tootallnate/once | 1.1.2 → 2.0.1 | GHSA-vpq2-c234-7xj6 |
| unzipper | 0.9.15 → 0.12.5 | Removes the unlicensed `buffers` 0.1.1 |

OSV results on 2026-10-03: as published, 34 advisories in 10 packages; with the overrides,
9 in 5 packages:

| Package | Advisories | Used at | Disposition |
|---|---|---|---|
| decompress 4.2.1 | 2 critical, 2 moderate (path traversal) | **Runtime**: Theia's .vsix unpacker | No fix; the clean fork is ESM-only and would break Theia. Extension installation stays disabled until TASK-021 replaces the extraction |
| braces 3.0.3 | 1 high (DoS) | Build only (@theia/cli) | No fix; own build patterns only; re-check at upgrades |
| http-cache-semantics 4.2.0 | 1 high | Build only (@theia/ffmpeg download) | No fix; no shared cache in a single-user build |
| serialize-javascript 6.0.2 | 1 high, 1 moderate | Test only (Theia's mocha) | Not used by Nexees's tests |
| diff 7.0.0 | 1 low | Test only (mocha) | Same; the runtime copy (5.2.2) is fixed |

The Rust tree had **no** advisories (cargo-deny 0.20.2 against RustSec advisory-db
`ef6173cb`, and OSV).

Deprecated packages: the evaluated lockfile marks 13 packages deprecated by their
maintainers, 8 outside build and test tooling: `xterm` 5.3.0 and its three addons (renamed
to `@xterm/*`), `glob` 7.2.3 and 10.5.0, `inflight` 1.0.6 (which leaks memory), `nano` 10.1.4
and `prebuild-install` 7.1.3. They come with Theia 1.76.0 and are tracked under OI-06.

## Rejected alternatives

- **VS Code Marketplace**: its terms restrict extensions to Microsoft products; Open VSX instead.
- **libgit2 / git2**: a C library with its own transports; the Git CLI on Desktop, gix as the Android candidate.
- **LiteLLM**: a Python proxy that cannot run on the phone and would duplicate the provider registry.
- **keytar**: archived upstream; the keyring crate for Nexees secrets.
- **UniFFI**: unnecessary for one JSON entry point; MPL-2.0 duties without benefit yet.
- **ammonia**: a second HTML parser; parse-time filtering already removes active content.
- **rquickjs, mlua**: C engines (and no Lua need); Boa is pure Rust.
- **aws-lc-rs**: a heavy C and assembly build for Android; ring covers TLS 1.3.
- **cargo-ndk**: plain cargo with the NDK compiler does the same.

## Repository tools

The repository checks run these tools. No product build links or ships them, so they
carry no redistribution duty. The machine-readable list is `data.dep_repository_tools` in
`strategy.lcl.txt` (TASK-005).

| ID | Tool | License | Use |
|---|---|---|---|
| TOOL-01 | cargo-deny 0.20.2 | MIT OR Apache-2.0 | The license, ban, source and advisory gate (DS-07, DS-08), run by the checks' deps stage |
| TOOL-02 | Python 3.11 or later, standard library only | PSF-2.0 | The check runner, the scoped cleanup tool and their tests |
| TOOL-03 | The LCL engine `lcl` 0.9.1 with the canonical Core packages | The owner's terms (OI-01) | Checks the LCL documentation projects |

The Rust toolchain, rustfmt included, is DEP-RUST above.

## Open items

| ID | Item | Owner task |
|---|---|---|
| OI-01 | Owner records shipping terms for the LCL engine, Core packages and manual | TASK-073, TASK-075 |
| OI-02 | Auth backend hosting and cost | TASK-027, TASK-028 |
| OI-03 | Off-LAN route and any account or cost | TASK-060, TASK-070 |
| OI-04 | Replace or fix Theia's .vsix extraction before enabling extension installs | TASK-021 |
| OI-05 | Handle Theia's keytar-based key store | TASK-025 |
| OI-06 | Track Theia and Electron releases; drop overrides; leave Electron 42 before end of support | TASK-073 |
| OI-07 | Prove or gap Git, the syntax index and language servers on Android | TASK-033, TASK-039, TASK-066 |
| OI-08 | Repeat the Android scenarios on real phones | TASK-066, TASK-074 |
| OI-09 | Move the device identity key into the Android Keystore | TASK-025, TASK-060 |
| OI-10 | Real login, logout, lock and reboot tests of the Desktop receiver | TASK-009 |
