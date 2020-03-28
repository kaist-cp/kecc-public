#!/usr/bin/env bash

cargo run --manifest-path $PROJECT_DIR/Cargo.toml --release -- --parse test_reduced.c >/dev/null 2>&1 &&\
! cargo run --manifest-path $PROJECT_DIR/Cargo.toml --release --bin fuzz -- $FUZZ_ARG test_reduced.c
