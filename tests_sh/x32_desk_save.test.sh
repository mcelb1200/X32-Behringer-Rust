#!/bin/bash

# Test module for x32_desk_save

test_x32_desk_save() {
    log_message "--- Running x32_desk_save tests ---"

    local binary_path="./target/release/x32_desk_save"
    if [ ! -f "$binary_path" ]; then
        log_message "ERROR: x32_desk_save not found. Please compile first."
        return
    fi

    if [ "$X32_CONNECTION_TYPE" = "None" ]; then
        log_message "Skipping x32_desk_save tests as no connection is available."
        return
    fi

    local ip="$X32_IP_ADDRESS"
    local test_file="test_desk_save_output.txt"

    # --- Test 1: File Creation and Sanity Check ---
    log_message "Test 1: File Creation and Sanity Check"

    # Cleanup from previous runs
    rm -f "$test_file"

    log_message "Running: x32_desk_save --desk-save $test_file"
    "$binary_path" --ip "$ip" --desk-save "$test_file"

    # Verification
    if [ ! -f "$test_file" ]; then
        log_message "Test 1 FAILED: The output file '$test_file' was not created."
        return
    fi

    if [ ! -s "$test_file" ]; then
        log_message "Test 1 FAILED: The output file is empty."
    elif [[ $(head -n 1 "$test_file") == /* ]]; then
        log_message "Test 1 PASSED: Output file created and first line appears to be a valid OSC message."
    else
        log_message "Test 1 FAILED: The output file does not contain expected OSC data."
        log_message "First line was: $(head -n 1 "$test_file")"
    fi

    # --- Teardown ---
    rm "$test_file"
    log_message "Cleaned up test file."

    log_message "--- Finished x32_desk_save tests ---"
}
