#!/usr/bin/env bash

# Clears the previous submissions.
rm -rf irgen.zip simplify_cfg.zip mem2reg.zip gvn.zip deadcode.zip asmgen.zip final.zip

# Creates new submissions.
zip irgen.zip -j src/c/write_c.rs src/irgen/mod.rs
zip simplify_cfg.zip -j src/opt/opt_utils.rs src/opt/simplify_cfg.rs
zip mem2reg.zip -j src/opt/opt_utils.rs src/opt/mem2reg.rs
zip gvn.zip -j src/opt/opt_utils.rs src/opt/gvn.rs
zip deadcode.zip -j src/opt/opt_utils.rs src/opt/deadcode.rs
zip asmgen.zip -r src/c/write_c.rs src/irgen/mod.rs src/opt/opt_utils.rs src/asmgen/*.rs
zip final.zip -r src/
