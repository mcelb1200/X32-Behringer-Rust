#!/bin/bash

# Test module for x32_get_scene
#
# Credits
#
# *   Original concept and work on the C library: Patrick-Gilles Maillot
# *   Additional concepts by: [User]
# *   Rust implementation by: [User]

test_x32_get_scene() {
    log_message "--- Running x32_get_scene tests ---"

    local binary_path="./target/release/x32_get_scene"
    if [ ! -f "$binary_path" ]; then
        log_message "ERROR: x32_get_scene not found. Please compile first."
        return
    fi

    if [ "$X32_CONNECTION_TYPE" = "None" ]; then
        log_message "Skipping x32_get_scene tests as no connection is available."
        return
    fi

    local ip="$X32_IP_ADDRESS"
    local input_file="test_scene_input.txt"
    local output_file="test_scene_output.txt"
    local scene_name="TestScene"
    local scene_note="Test Note"

    # --- Setup: Create synthetic input file ---
    echo "/ch/01/config/name" > "$input_file"
    echo "/ch/01/mix/fader" >> "$input_file"
    log_message "Created synthetic input file: $input_file"

    # --- Test 1: File Generation and Content Verification ---
    log_message "Test 1: File Generation and Content Verification"

    # Cleanup from previous runs
    rm -f "$output_file"

    log_message "Running: x32_get_scene with redirected input..."
    "$binary_path" --ip "$ip" --scene-name "$scene_name" --note "$scene_note" < "$input_file" > "$output_file"

    # Verification
    if [ ! -f "$output_file" ]; then
        log_message "Test 1 FAILED: The output file '$output_file' was not created."
        rm "$input_file"
        return
    fi

    local content=$(cat "$output_file")
    local header_line=$(echo "$content" | head -n 1)
    local first_data_line=$(echo "$content" | sed -n '2p')
    local second_data_line=$(echo "$content" | sed -n '3p')

    if [ $(echo "$content" | wc -l) -lt 3 ]; then
        log_message "Test 1 FAILED: The output file has insufficient content."
    elif [[ "$header_line" != "#2.7# \"$scene_name\" \"$scene_note\""* ]]; then
        log_message "Test 1 FAILED: The scene header is incorrect."
        log_message "Expected header starting with: #2.7# \"$scene_name\" \"$scene_note\""
        log_message "Actual header: $header_line"
    elif [[ "$first_data_line" != "/ch/01/config/name"* ]] || [[ "$second_data_line" != "/ch/01/mix/fader"* ]]; then
        log_message "Test 1 FAILED: The output file does not contain the expected OSC responses."
        log_message "Actual content:\n$content"
    else
        log_message "Test 1 PASSED: Output file created with correct header and content structure."
    fi

    # --- Teardown ---
    rm "$input_file" "$output_file"
    log_message "Cleaned up test files."

    log_message "--- Finished x32_get_scene tests ---"
}
