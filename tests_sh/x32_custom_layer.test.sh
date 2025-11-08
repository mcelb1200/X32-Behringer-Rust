#!/bin/bash

# Test module for x32_custom_layer

test_x32_custom_layer() {
    log_message "--- Running x32_custom_layer tests ---"

    local binary_path="./target/release/x32_custom_layer"
    if [ ! -f "$binary_path" ]; then
        log_message "ERROR: x32_custom_layer not found. Please compile first."
        return
    fi

    if [ "$X32_CONNECTION_TYPE" = "None" ]; then
        log_message "Skipping x32_custom_layer tests as no connection is available."
        return
    fi

    local ip="$X32_IP_ADDRESS"
    local test_file="test_layer.chn"

    # --- Test 1: Save and Restore Layer ---
    log_message "Test 1: Save and Restore Layer"
    echo -e "\e[33mThis test will verify the 'save' and 'restore' commands.\e[0m"
    echo -e "\e[33mSETUP: Please go to Channel 1 and change its name to 'SAVED_STATE'.\e[0m"
    read -p "Press Enter when you have set the channel name..."

    log_message "Running: x32_custom_layer save --file $test_file"
    "$binary_path" --ip "$ip" save --file "$test_file"

    if [ ! -f "$test_file" ]; then
        log_message "Test 1 FAILED: The save command did not create the output file."
        return
    fi
    log_message "Save file created successfully."

    echo -e "\e[33mSETUP 2: Now, please change the name of Channel 1 to 'MODIFIED'.\e[0m"
    read -p "Press Enter when you have changed the name again..."

    log_message "Running: x32_custom_layer restore --file $test_file"
    "$binary_path" --ip "$ip" restore --file "$test_file"

    echo "VERIFY: Please check if the name of Channel 1 has been restored to 'SAVED_STATE'."
    read -p "Did the restore succeed? (y/n) " result1
    if [[ "$result1" == "y" ]]; then
        log_message "Test 1 PASSED (User reported success)."
    else
        log_message "Test 1 FAILED (User reported failure)."
    fi
    rm "$test_file"
    read -p "Press Enter for the next test..."


    # --- Test 2: Reset Channel ---
    log_message "Test 2: Reset Channel"
    echo -e "\e[33mThis test will verify the 'reset' command.\e[0m"
    echo -e "\e[33mSETUP: Please go to Channel 2 and change its name to 'TO_BE_RESET'.\e[0m"
    read -p "Press Enter when you have set the channel name..."

    log_message "Running: x32_custom_layer reset --channels 2"
    "$binary_path" --ip "$ip" reset --channels "2"

    echo "VERIFY: Please check if Channel 2 has been reset to its default state (name is cleared or default)."
    read -p "Did the reset succeed? (y/n) " result2
    if [[ "$result2" == "y" ]]; then
        log_message "Test 2 PASSED (User reported success)."
    else
        log_message "Test 2 FAILED (User reported failure)."
    fi

    log_message "--- Finished x32_custom_layer tests ---"
}
