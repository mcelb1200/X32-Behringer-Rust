#!/bin/bash

# Test module for x32_set_scene

test_x32_set_scene() {
    log_message "--- Running x32_set_scene tests ---"

    local binary_path="./target/release/x32_set_scene"
    if [ ! -f "$binary_path" ]; then
        log_message "ERROR: x32_set_scene not found. Please compile first."
        return
    fi

    if [ "$X32_CONNECTION_TYPE" = "None" ]; then
        log_message "Skipping x32_set_scene tests as no connection is available."
        return
    fi

    local ip="$X32_IP_ADDRESS"
    local input_file="test_set_scene_input.txt"

    # --- Setup: Create synthetic scene file ---
    echo '/ch/01/config/name "SCENE_SET"' > "$input_file"
    echo '/ch/01/mix/fader ,f 1.0' >> "$input_file"
    log_message "Created synthetic scene file: $input_file"

    # --- Test 1: Interactive Scene Set ---
    log_message "Test 1: Interactive Scene Set"
    echo -e "\e[33mThis test will verify that the tool can set parameters from a file.\e[0m"
    echo -e "\e[33mSETUP: Please go to Channel 1 and ensure its name is cleared and its fader is at the minimum position.\e[0m"
    read -p "Press Enter to run the test..."

    log_message "Running x32_set_scene with redirected input..."
    "$binary_path" --ip "$ip" --delay 10 < "$input_file"

    echo "VERIFY: Please check the X32 console."
    echo "Is the name of Channel 1 now 'SCENE_SET' and is its fader at the maximum position (+10dB)?"
    read -p "(y/n) " result
    if [[ "$result" == "y" ]]; then
        log_message "Test 1 PASSED (User reported success)."
    else
        log_message "Test 1 FAILED (User reported failure)."
    fi

    # --- Teardown ---
    # As a courtesy, reset the channel that was just modified.
    {
        echo '/ch/01/config/name ""'
        echo '/ch/01/mix/fader ,f 0.0'
    } | "$binary_path" --ip "$ip" --delay 10

    rm "$input_file"
    log_message "Cleaned up test file and reset channel 1."

    log_message "--- Finished x32_set_scene tests ---"
}
