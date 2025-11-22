#!/bin/bash
set -e

# Compile
cargo build -p x32_set_preset

# Check Help
./target/debug/x32_set_preset --help

# Create dummy channel preset
echo "#2.7#" > test.chn
echo "/config/name \"Kick\"" >> test.chn
echo "/headamp/gain 2.5" >> test.chn

# Run with valid arguments (expect connection error, but not panic)
# We capture output and check for "Connecting to X32"
output=$(./target/debug/x32_set_preset test.chn --target ch01 --ip 127.0.0.1 --verbose 2>&1 || true)

if [[ "$output" == *"Connecting to X32"* ]]; then
    echo "Argument parsing verified."
else
    echo "Failed to reach connection step."
    echo "$output"
    exit 1
fi

rm test.chn
echo "x32_set_preset check passed."
