#!/bin/bash
set -e

cargo build -p x32_replay
./target/debug/x32_replay --help

echo "x32_replay check passed."
