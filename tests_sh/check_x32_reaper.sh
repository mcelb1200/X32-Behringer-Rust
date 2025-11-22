#!/bin/bash
set -e

cargo build -p x32_reaper
./target/debug/x32_reaper --help

echo "x32_reaper check passed."
