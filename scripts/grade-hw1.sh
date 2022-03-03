#!/usr/bin/env bash

# Exit when any command fails.
set -e

# Run lints.
cargo fmt --all -- --check # run `cargo fmt` to auto-correct format.
cargo clippy

# Run tests.
RUST_MIN_STACK=33554432 cargo test test_examples_write_c --release
python3 tests/fuzz.py --print -n30
