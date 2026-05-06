# Agent Guidelines

## Formatting and Lints

Always run the following after any code modification:

```bash
cargo +nightly fmt
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

## Modifying BFT Code

When making changes to BFT-related code (anything under `node/bft/`), run the following checks in order:

### 1. Unit tests

```bash
cargo test -p snarkos-node-bft --lib
```

### 2. Build with test features

The CI scripts invoke `snarkos` by name, so the binary must be on PATH. Build it with the
`test_network` feature and prepend the output directory to PATH:

```bash
cargo build --features test_network
export PATH="$PWD/target/debug:$PATH"
```

### 3. Devnet test

```bash
.ci/test_devnet.sh
```

### 4. Additional CI tests

Ask the user whether they want to run the following tests before merging. These cannot run
concurrently and take significant time in total, so they are not always run on every change.

```bash
.ci/test_partial_upgrade.sh
.ci/test_full_upgrade.sh
.ci/test_restart_majority.sh
.ci/test_reset_minority.sh
.ci/test_restart_all.sh
```
