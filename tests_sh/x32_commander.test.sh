#!/bin/bash

# Test module for x32_commander
#
# Credits
#
# *   Original concept and work on the C library: Patrick-Gilles Maillot
# *   Additional concepts by: [User]
# *   Rust implementation by: [User]

test_x32_commander() {
    log_message "--- Running x32_commander tests ---"

    local binary_path="./target/release/x32_commander"
    if [ ! -f "$binary_path" ]; then
        log_message "ERROR: x32_commander not found. Please compile first."
        return
    fi

    if [ "$X32_CONNECTION_TYPE" = "None" ]; then
        log_message "Skipping x32_commander tests as no connection is available."
        return
    fi

    # --- Setup: Create synthetic command file ---
    local command_file="test_commander.txt"
    echo "O~~~/ch/01/mix/fader|/ch/02/mix/fader ,f 1.0" > "$command_file"
    log_message "Created synthetic command file: $command_file"

    # --- Test 1: Interactive OSC Mapping Test ---
    log_message "Test 1: Interactive OSC Mapping Test"
    echo -e "\e[33mThis test will start the X32Commander to map Channel 1 fader to Channel 2.\e[0m"
    echo -e "\e[33mPlease have the X32 console or X32-Edit visible.\e[0m"
    read -p "Press Enter to begin the test..."

    log_message "Starting x32_commander..."
    "$binary_path" --ip "$X32_IP_ADDRESS" --file "$command_file" &
    local pid=$!

    if ! ps -p $pid > /dev/null; then
        log_message "Test 1 FAILED: Could not start x32_commander process."
        rm "$command_file"
        return
    fi

    echo "X32Commander is running."
    echo "Please move the fader for Channel 1 on the console."
    echo "Observe if the fader for Channel 2 moves to the maximum position."

    read -p "Did the fader for Channel 2 move to max when you moved Channel 1? (y/n) " result
    if [[ "$result" == "y" ]]; then
        log_message "Test 1 PASSED (User reported success)."
    else
        log_message "Test 1 FAILED (User reported failure)."
    fi

    # --- Teardown ---
    log_message "Stopping x32_commander..."
    kill $pid
    wait $pid 2>/dev/null
    rm "$command_file"
    log_message "Cleaned up test files."

    log_message "--- Finished x32_commander tests ---"
}
