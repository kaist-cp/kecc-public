#!/usr/bin/env bash

# Clears the previous submissions.
rm -rf hw2.zip hw3.zip hw4.zip hw5.zip hw6.zip

# Creates new submissions.
zip hw2.zip -j src/c/write_c.rs src/irgen/mod.rs
zip hw3.zip -j src/opt/opt_utils.rs src/opt/simplify_cfg.rs
zip hw4.zip -j src/opt/opt_utils.rs src/opt/mem2reg.rs
zip hw5.zip -j src/opt/opt_utils.rs src/opt/gvn.rs
zip hw6.zip -j src/opt/opt_utils.rs src/opt/deadcode.rs
zip final.zip -r src/
