#!/bin/bash
set -e

# Compile the binary
cargo build -p x32_tap

# Run help
./target/debug/x32_tap --help

echo "x32_tap build and help check passed."
