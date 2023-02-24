#!/bin/bash

# Don't run this script if you're going to distribute the binary
# just use 'cargo build --release' 


branch_name=$(git rev-parse --abbrev-ref HEAD)

if [[ "$branch_name" == "nightly" || "$branch_name" == "nightly-dev" ]]; then
    if [[ "$1" == "-d" ]]; then
        cargo clippy --features "quikc-nightly" && cargo build --features "quikc-nightly"
        exit
    fi
    cargo clippy --features "quikc-nightly" && RUSTFLAGS="-C target-cpu=native" cargo build --release --features "quikc-nightly"
else
    if [[ "$1" == "-d" ]]; then
        cargo clippy && cargo build
        exit
    fi
    cargo clippy && RUSTFLAGS="-C target-cpu=native" cargo build --release
fi

