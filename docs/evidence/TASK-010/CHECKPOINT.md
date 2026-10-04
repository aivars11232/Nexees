# TASK-010 — checkpoint, not a closure

**Status: in progress. TASK-010 is not accepted and has no receipt.** This file marks a save
point that the owner asked for on 2026-10-04, before a pause of the development session. The
task's evidence record replaces it when the task is closed.

## What this commit holds

- **The logo (B1).** The owner resolved the open item BIND-B1-LOGO on 2026-10-04. Asked whether
  the file `Nexees Logo⁄Icon.png` in `/home/aivars/Pictures` (its name holds the character
  U+2044 where the binding writes a folder separator) is the logo source, the owner answered:
  "/home/aivars/pictures that's where Nexees logo is", and then sent the logo image itself, which
  shows the emblem of that file. The file was read in place; nothing in that folder was changed.
  `scripts/build/brand_assets.py import` copied it byte for byte to
  `assets/branding/source/nexees-logo.png` (SHA-256
  `334506456a00f0d3eabe7a2f01cca9c90548e9c52caee3f40e00cc2751b59bdd`) and derived seven icon sizes
  into `assets/branding/derived/`; `assets/branding/manifest.json` records them.
- **The shared design tokens.** `assets/theme/design_tokens.json` and
  `assets/theme/icon_mapping.json`, checked and copied for the Desktop by
  `scripts/build/design_tokens.py`.
- **The Desktop look.** The Nexees dark theme and compact sizes
  (`apps/desktop/src/shell/theme.ts`), the About dialog with the logo
  (`apps/desktop/src/shell/about_dialog.ts`), the application icons and the desktop entry's icon
  (`scripts/build/build_desktop.py`).
- **Tests.** `tests/tooling/test_brand_assets.py` and `tests/tooling/test_design_tokens.py`.

At this commit `scripts/test/run_checks.py` passes all nine stages, and the application builds,
installs and shows the theme and the About dialog in a private test session.

## What is still to do before TASK-010 can close

- The end-to-end test of the installed application does not yet check the theme, the About
  dialog or the installed icons, and has not been run against this build.
- A test of the build's refusal of an icon the manifest does not record.
- Manual, conventions, README and the dependency and security records.
- Negative tests, regression run, cleanup, final verification, the evidence record and the
  receipt.
