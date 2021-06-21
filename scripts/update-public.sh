#!/usr/bin/env bash

files="src/asmgen/mod.rs src/c/write_c.rs src/irgen/mod.rs src/opt/deadcode.rs src/opt/gvn.rs src/opt/mem2reg.rs src/opt/opt_utils.rs src/opt/simplify_cfg.rs"

for file in $files; do
    mv $file $file.public
done

# deleted:    src/asmgen/mod.rs.public
# deleted:    src/c/write_c.rs.public
# deleted:    src/irgen/mod.rs.public
# deleted:    src/opt/deadcode.rs.public
# deleted:    src/opt/gvn.rs.public
# deleted:    src/opt/mem2reg.rs.public
# deleted:    src/opt/opt_utils.rs.public
# deleted:    src/opt/simplify_cfg.rs.public
