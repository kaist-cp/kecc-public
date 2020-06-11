#!/usr/bin/env bash

rm -rf hw4.zip hw5.zip
zip hw4.zip -j src/opt/opt_utils.rs src/opt/mem2reg.rs
zip hw5.zip -j src/opt/opt_utils.rs src/opt/gvn.rs
