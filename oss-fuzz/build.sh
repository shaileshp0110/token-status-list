#!/bin/bash
# build.sh for OSS-Fuzz
# This file should be placed in google/oss-fuzz/projects/vc-status-list/
# Reference: https://google.github.io/oss-fuzz/getting-started/new-project-guide/rust-lang/

set -e

cd $SRC/vc-status-list

# Build fuzz targets in release mode with debug assertions
# The -O flag builds in release mode, --debug-assertions enables additional checks
cargo fuzz build -O --debug-assertions

# Copy all fuzz targets to $OUT
# This automatically picks up all fuzz targets from fuzz/fuzz_targets/
FUZZ_TARGET_OUTPUT_DIR=fuzz/target/x86_64-unknown-linux-gnu/release
for f in fuzz/fuzz_targets/*.rs
do
    FUZZ_TARGET_NAME=$(basename ${f%.*})
    cp $FUZZ_TARGET_OUTPUT_DIR/$FUZZ_TARGET_NAME $OUT/
done

