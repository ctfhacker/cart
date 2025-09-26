#!/usr/bin/env bash
set -euo pipefail

# Ensure the code is formatted properly
cargo fmt --all -- --check

# Check clippy lints with strict deny list
cargo clippy --all-features --all-targets -- \
  -D warnings \
  -D clippy::pedantic \
  -D clippy::complexity \
  -D clippy::correctness \
  -D clippy::perf \
  -D clippy::style

# Build all targets
cargo build --verbose --all-targets

# Run the full test suite
cargo test --workspace --all-targets
