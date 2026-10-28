#!/bin/sh
set -eu
cargo fmt --all --check
cargo clippy -q --all-targets
cargo test -q
cargo lint
cargo llvm-cov -q --workspace --fail-under-lines 95 --summary-only
cargo mutants -j 4 || [ $? -eq 3 ]
