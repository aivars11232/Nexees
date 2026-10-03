#!/bin/sh
# Continuous-integration entry point for Nexees: what a CI service runs for every change.
#
# It fetches the locked Rust dependencies, then runs every stage of scripts/test/run_checks.py
# in order. The job fails when any stage fails, including a stage whose tool is missing. No
# hosted CI service is configured in this repository: choosing one, and any cost it brings, is
# the owner's decision (docs/engineering/CONVENTIONS.md, "Continuous integration").
#
# The CI machine provides:
#   - the full Git history, because the layout stage compares placeholders with the commit
#     that imported them;
#   - rustup, which installs the toolchain that rust-toolchain.toml pins;
#   - git, python3 3.11 or later, and cargo-deny 0.20.2;
#   - the LCL engine `lcl` 0.9.1, with the canonical Core packages named by the variables
#     NEXEES_LCL_CORE_01 and NEXEES_LCL_CORE_03.
#
# Extra arguments go to run_checks.py, for example `--stage docs` to run one stage.
set -eu
cd "$(dirname "$0")/../.."

# Download the locked dependencies once; the checks themselves then run offline.
cargo fetch --locked
# --online lets cargo-deny refresh its advisory database before the deps stage uses it.
exec python3 -B scripts/test/run_checks.py --online "$@"
