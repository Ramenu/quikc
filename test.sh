#!/bin/bash

branch_name=$(git rev-parse --abbrev-ref HEAD)
export rustc_version=$(rustc --version)

if [[ "$branch_name" == "nightly" || "$branch_name" == "nightly-dev" ]]; then
    cargo clippy --features "quikc-nightly" && cargo test test_all --features quikc-nightly
else
    cargo clippy && cargo test test_all
fi