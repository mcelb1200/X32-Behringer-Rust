#!/bin/bash

# Test module for x32_desk_restore

test_x32_desk_restore() {
    log_message "--- Running x32_desk_restore tests ---"

    local restore_binary="$BINARY_PATH/x32_desk_restore"
    local save_binary="$BINARY_PATH/x32_desk_save"

    if [ ! -f "$restore_binary" ]; then
        log_message "ERROR: x32_desk_restore not found. Please compile first."
        return
    fi
    if [ ! -f "$save_binary" ]; then
        log_message "ERROR: x32_desk_save not found. This test requires it to generate a restore file."
        return
    fi

    if [ "$X32_CONNECTION_TYPE" = "None" ]; then
        log_message "Skipping x32_desk_restore tests as no connection is available."
        return
    fi

    local ip="$X32_IP_ADDRESS"
    local test_file="test_desk_state.txt"

    # --- Test 1: Full Save and Restore Cycle ---
    log_message "Test 1: Full Save and Restore Cycle"
    echo -e "\e[33mThis test will verify the 'x32_desk_save' and 'x32_desk_restore' workflow.\e[0m"
    echo -e "\e[33mSETUP: Please go to Channel 1 and change its name to 'DESK_SAVE_TEST'.\e[0m"
    read -p "Press Enter when you have set the channel name..."

    log_message "Running x32_desk_save to capture the current state..."
    "$save_binary" --ip "$ip" "$test_file"

    if [ ! -f "$test_file" ]; then
        log_message "Test 1 FAILED: The save command did not create the output file."
        return
    fi
    log_message "Save file created successfully."

    echo -e "\e[33mSETUP 2: Now, please change the name of Channel 1 to 'MODIFIED'.\e[0m"
    read -p "Press Enter when you have changed the name..."

    log_message "Running x32_desk_restore to restore the previous state..."
    "$restore_binary" --ip "$ip" "$test_file"

    echo "VERIFY: Please check if the name of Channel 1 has been restored to 'DESK_SAVE_TEST'."
    read -p "Did the restore succeed? (y/n) " result
    if [[ "$result" == "y" ]]; then
        log_message "Test 1 PASSED (User reported success)."
    else
        log_message "Test 1 FAILED (User reported failure)."
    fi

    # --- Teardown ---
    rm "$test_file"
    log_message "Cleaned up test files."

    log_message "--- Finished x32_desk_restore tests ---"
}
