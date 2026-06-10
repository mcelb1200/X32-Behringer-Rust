#!/bin/bash
OUTPUT=$(cargo test -p x32_tcp --test integration_test 2>&1)
echo "$OUTPUT"
