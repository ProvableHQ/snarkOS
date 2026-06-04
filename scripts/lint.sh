#!/usr/bin/env bash
#
# Runs the same lints as CI: clippy, rustfmt, and shellcheck.
# Invoked by the rusty-hook pre-commit hook (see .rusty-hook.toml).

set -e

echo "Running clippy..."
cargo clippy --workspace --all-targets --all-features -- -D warnings

echo "Checking rustfmt..."
cargo +nightly fmt --all -- --check

if command -v shellcheck >/dev/null; then
  echo "Running shellcheck..."
  shellcheck -x .ci/*.sh scripts/*.sh
else
  echo "warning: shellcheck not installed, skipping script lint (CI will still run it)"
fi
