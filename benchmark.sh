#!/bin/bash

if [[ "$branch_name" == "nightly" || "$branch_name" == "nightly-dev" ]]; then
    cargo clippy --features "quikc-nightly" && cargo test quikc_benchmark --features quikc-nightly -- --nocapture
else
    cargo clippy && cargo test quikc_benchmark -- --nocapture
fi