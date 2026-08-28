#! /bin/bash

set -x

BASELINE_REV=5233959ad4 # UPDATE ME ON NECESSARY BREAKING CHANGES

# Ensure that the command is installed.
cargo install cargo-semver-checks@0.43.0 --locked

# Ensure we can find the baseline revision
git fetch --unshallow || true

cargo semver-checks --workspace --default-features --baseline-rev $BASELINE_REV
