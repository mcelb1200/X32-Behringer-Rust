#!/bin/bash

# Test module for x32_emulator

test_x32_emulator() {
    log_message "--- Running x32_emulator tests ---"

    local binary_path="$BINARY_PATH/x32_emulator"
    if [ ! -f "$binary_path" ]; then
        log_message "ERROR: x32_emulator not found. Please compile first."
        return
    fi

    # Test 1: Start and stop the emulator
    log_message "Test 1: Starting emulator..."
    "$binary_path" &
    local pid=$!
    sleep 5 # Give it time to start

    if ps -p $pid > /dev/null; then
        log_message "Emulator process found. Stopping..."
        kill $pid
        log_message "Test 1 PASSED."
    else
        log_message "Test 1 FAILED: Could not find emulator process."
    fi

    log_message "--- Finished x32_emulator tests ---"
}
