#!/bin/bash

# Test module for x32_get_scene_name
#
# Credits
#
# *   Original concept and work on the C library: Patrick-Gilles Maillot
# *   Additional concepts by: [User]
# *   Rust implementation by: [User]

test_x32_get_scene_name() {
    log_message "--- Running x32_get_scene_name tests ---"

    local binary_path="./target/release/x32_get_scene_name"
    if [ ! -f "$binary_path" ]; then
        log_message "ERROR: x32_get_scene_name not found. Please compile first."
        return
    fi

    if [ "$X32_CONNECTION_TYPE" = "None" ]; then
        log_message "Skipping x32_get_scene_name tests as no connection is available."
        return
    fi

    local ip="$X32_IP_ADDRESS"
    local output_file="scene_name_output.txt"

    # --- Test 1: Interactive Scene Change Detection ---
    log_message "Test 1: Interactive Scene Change Detection"
    echo -e "\e[33mThis test will verify that the tool can detect a scene change.\e[0m"

    log_message "Starting x32_get_scene_name in the background..."
    "$binary_path" --ip "$ip" --onetime 1 --verbose 0 > "$output_file" &
    local pid=$!

    echo "The scene name listener is running."
    echo -e "\e[33mACTION: On the X32 console, please load any scene.\e[0m"
    echo "The test will complete automatically once the scene change is detected."

    # Wait for the process to exit (onetime=1)
    wait $pid
    log_message "Process has exited. Checking output..."

    # Verification
    if [ ! -f "$output_file" ]; then
        log_message "Test 1 FAILED: The output file was not created."
        return
    fi

    local output=$(cat "$output_file")
    if [ -z "$output" ]; then
        log_message "Test 1 FAILED: The tool did not produce any output."
    elif [[ "$output" =~ ^[0-9]{2}[[:space:]]-[[:space:]].* ]]; then
        log_message "Test 1 PASSED: Scene change detected and output is in the correct format."
        log_message "Detected: $output"
    else
        log_message "Test 1 FAILED: The output format is incorrect."
        log_message "Actual output: $output"
    fi

    # --- Teardown ---
    rm "$output_file"
    log_message "Cleaned up test file."

    log_message "--- Finished x32_get_scene_name tests ---"
}
