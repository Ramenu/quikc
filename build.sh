#!/bin/bash

# Don't run this script if you're going to distribute the binary
# just use 'cargo build --release' 


branch_name=$(git rev-parse --abbrev-ref HEAD)

if [[ "$branch_name" == "nightly" || "$branch_name" == "nightly-dev" ]]; then
    cargo clippy --features "quikc-nightly" && RUSTFLAGS="-C target-cpu=native" cargo build --release --features "quikc-nightly"
else
    cargo clippy && RUSTFLAGS="-C target-cpu=native" cargo build --release
fi

