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
#     NEXEES_LCL_CORE_01 and NEXEES_LCL_CORE_03;
#   - Node.js 26 with npm 12, for the Desktop window's TypeScript check.
#
# Extra arguments go to run_checks.py, for example `--stage docs` to run one stage.
set -eu
cd "$(dirname "$0")/../.."

# Download the locked dependencies once; the checks themselves then run offline.
cargo fetch --locked
# The Desktop window's locked npm packages go into an npm cache outside the checkout, with no
# install script run; the lint stage's TypeScript check then installs them offline from it.
: "${NEXEES_BUILD_HOME:=${TMPDIR:-/tmp}/nexees-ci-home}"
: "${NEXEES_NPM_CACHE:=$NEXEES_BUILD_HOME/npm-cache}"
export NEXEES_BUILD_HOME NEXEES_NPM_CACHE
mkdir -p "$NEXEES_BUILD_HOME"
fill=$(mktemp -d)
mkdir "$fill/src"
cp apps/desktop/package.json apps/desktop/package-lock.json "$fill/"
cp apps/desktop/src/package.json "$fill/src/"
(cd "$fill" && HOME="$NEXEES_BUILD_HOME" \
    npm ci --ignore-scripts --no-audit --no-fund --cache "$NEXEES_NPM_CACHE" >/dev/null)
rm -rf "$fill"
# --online lets cargo-deny refresh its advisory database before the deps stage uses it.
exec python3 -B scripts/test/run_checks.py --online "$@"
