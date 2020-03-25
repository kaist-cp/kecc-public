#!/usr/bin/env bash

! cargo run --manifest-path $PROJECT_DIR/Cargo.toml --release --bin fuzz -- $FUZZ_ARG test_reduced.c
