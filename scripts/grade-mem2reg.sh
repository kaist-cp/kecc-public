#!/usr/bin/env bash

# Exit when any command fails.
set -e

# Run lints.
cargo fmt --all -- --check # run `cargo fmt` to auto-correct format.
cargo clippy

# Run tests.
RUST_MIN_STACK=33554432 cargo test --release test_examples_mem2reg -- --nocapture
