#!/bin/bash

# Test module for x32_copy_fx

test_x32_copy_fx() {
    log_message "--- Running x32_copy_fx tests ---"

    local binary_path="$BINARY_PATH/x32_copy_fx"
    if [ ! -f "$binary_path" ]; then
        log_message "ERROR: x32_copy_fx not found. Please compile first."
        return
    fi

    if [ "$X32_CONNECTION_TYPE" = "None" ]; then
        log_message "Skipping x32_copy_fx tests as no connection is available."
        return
    fi

    local ip="$X32_IP_ADDRESS"

    # --- Test 1: Copy FX slot ---
    log_message "Test 1: Copy FX slot"
    echo -e "\e[33mThis test will verify the 'copy' command.\e[0m"
    echo -e "\e[33mSETUP: Please set FX slot 1 to a 'Hall Reverb' and FX slot 2 to a 'Stereo Delay'.\e[0m"
    read -p "Press Enter when you have set up the FX slots..."

    log_message "Running: x32_copy_fx --from 1 --to 2 copy"
    "$binary_path" --ip "$ip" --from 1 --to 2 copy

    echo "VERIFY: Please check if FX slot 2 is now also a 'Hall Reverb' with the same settings as slot 1."
    read -p "Did the copy succeed? (y/n) " result1
    if [[ "$result1" == "y" ]]; then
        log_message "Test 1 PASSED (User reported success)."
    else
        log_message "Test 1 FAILED (User reported failure)."
    fi
    read -p "Press Enter for the next test..."

    # --- Test 2: Reset FX slot to defaults ---
    log_message "Test 2: Reset FX slot"
    echo -e "\e[33mThis test will verify the 'reset' command.\e[0m"
    echo -e "\e[33mSETUP: In FX slot 1 (Hall Reverb), please change the 'Decay' time to a low value (e.g., 0.5s).\e[0m"
    read -p "Press Enter when you have modified the parameter..."

    log_message "Running: x32_copy_fx --from 1 reset"
    "$binary_path" --ip "$ip" --from 1 reset

    echo "VERIFY: Please check if the 'Decay' time in FX slot 1 has returned to its default value (likely 2.0s)."
    read -p "Did the reset succeed? (y/n) " result2
    if [[ "$result2" == "y" ]]; then
        log_message "Test 2 PASSED (User reported success)."
    else
        log_message "Test 2 FAILED (User reported failure)."
    fi
    read -p "Press Enter for the next test..."

    # --- Test 3: Reset FX with custom defaults file ---
    log_message "Test 3: Reset with custom defaults"
    echo -e "\e[33mThis test will verify resetting from a custom defaults file.\e[0m"

    # Setup: Create synthetic defaults file
    local defaults_file="test_fx_defaults.txt"
    echo -e "HALL\n1.0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0" > "$defaults_file"
    log_message "Created synthetic defaults file with max decay for Hall Reverb."

    read -p "Press Enter to run the reset command with the custom file..."

    log_message "Running: x32_copy_fx --from 1 reset --defaults-file $defaults_file"
    "$binary_path" --ip "$ip" --from 1 reset --defaults-file "$defaults_file"

    echo "VERIFY: Please check if the 'Decay' time in FX slot 1 is now set to the maximum possible value (10.0s)."
    read -p "Did the custom reset succeed? (y/n) " result3
    if [[ "$result3" == "y" ]]; then
        log_message "Test 3 PASSED (User reported success)."
    else
        log_message "Test 3 FAILED (User reported failure)."
    fi

    # Teardown
    rm "$defaults_file"
    log_message "Cleaned up test files."

    log_message "--- Finished x32_copy_fx tests ---"
}
