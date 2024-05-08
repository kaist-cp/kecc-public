#!/usr/bin/env bash

# Exit when any command fails.
set -e

# Run lints.
cargo fmt --all -- --check # run `cargo fmt` to auto-correct the code. 
cargo clippy               # run `cargo clippy --fix` to auto-correct the code.

# Run tests.

# write_c
echo "Run write_c"
RUST_MIN_STACK=33554432 cargo test --release test_examples_write_c -- --nocapture
RUST_MIN_STACK=33554432 python3 tests/fuzz.py --print -n80 --seed 22

# irgen
echo "Run irgen"
RUST_MIN_STACK=33554432 cargo test --release test_examples_irgen -- --nocapture
RUST_MIN_STACK=33554432 python3 tests/fuzz.py --irgen -n80 --seed 22

# simplify_cfg
echo "Run simplify_cfg"
RUST_MIN_STACK=33554432 cargo test --release test_examples_simplify_cfg -- --nocapture

# mem2reg
echo "Run mem2reg"
RUST_MIN_STACK=33554432 cargo test --release test_examples_mem2reg -- --nocapture

# gvn
echo "Run gvn"
RUST_MIN_STACK=33554432 cargo test --release test_examples_gvn -- --nocapture

# deadcode
echo "Run deadcode"
RUST_MIN_STACK=33554432 cargo test --release test_examples_deadcode -- --nocapture

# asmgen
echo "Run asmgen"
RUST_MIN_STACK=33554432 cargo test --release test_examples_asmgen -- --nocapture

echo "All tests successful"
