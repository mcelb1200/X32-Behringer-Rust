#!/bin/bash
set -e

cargo build -p x32_get_lib
./target/debug/x32_get_lib --help

cargo build -p x32_set_lib
./target/debug/x32_set_lib --help

echo "x32_get_lib and x32_set_lib check passed."
