#!/bin/bash

# Don't run this script if you're going to distribute the binary
# just use 'cargo build --release' 

RUSTFLAGS="-C target-cpu=native" cargo build --release
