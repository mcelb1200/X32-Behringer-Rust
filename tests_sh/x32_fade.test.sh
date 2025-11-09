#!/bin/bash

# Test module for x32_fade

test_x32_fade() {
    log_message "--- Running x32_fade tests ---"

    local binary_path="$BINARY_PATH/x32_fade"
    if [ ! -f "$binary_path" ]; then
        log_message "ERROR: x32_fade not found. Please compile first."
        return
    fi

    if [ "$X32_CONNECTION_TYPE" = "None" ]; then
        log_message "Skipping x32_fade tests as no connection is available."
        return
    fi

    local ip="$X32_IP_ADDRESS"
    local test_fader="/ch/01/mix/fader"
    local config_file="test_fade_config.json"

    # --- Test 1: Interactive Fade In and Fade Out ---
    log_message "Test 1: Interactive Fade In and Fade Out"
    echo -e "\e[33mThis test will verify the fade-in and fade-out functionality.\e[0m"
    echo -e "\e[33mSETUP: Please move the fader for Channel 1 to its minimum position (-oo).\e[0m"
    read -p "Press Enter to begin the fade-in test..."

    log_message "Running fade-in on $test_fader..."
    "$binary_path" --ip "$ip" --fader "$test_fader" --fade-in 2.0 --steps 20

    echo "VERIFY: Did the fader for Channel 1 smoothly rise to the 0dB mark?"
    read -p "(y/n) " result1
    if [[ "$result1" == "y" ]]; then
        log_message "Fade-in test PASSED (User reported success)."
    else
        log_message "Fade-in test FAILED (User reported failure)."
    fi

    read -p "Press Enter to begin the fade-out test..."

    log_message "Running fade-out on $test_fader..."
    "$binary_path" --ip "$ip" --fader "$test_fader" --fade-out 2.0 --steps 20

    echo "VERIFY: Did the fader for Channel 1 smoothly lower to its minimum position?"
    read -p "(y/n) " result2
    if [[ "$result2" == "y" ]]; then
        log_message "Fade-out test PASSED (User reported success)."
    else
        log_message "Fade-out test FAILED (User reported failure)."
    fi
    read -p "Press Enter for the next test..."


    # --- Test 2: Save and Load Config ---
    log_message "Test 2: Save and Load Config"
    echo -e "\e[33mThis test will verify the --save-config and --load-config functionality.\e[0m"

    # Setup: Create the config file
    rm -f "$config_file"
    log_message "Saving configuration to $config_file..."
    "$binary_path" --ip "$ip" --fader "$test_fader" --fade-in 2.0 --save-config "$config_file"

    if [ ! -f "$config_file" ]; then
        log_message "Test 2 FAILED: The save-config command did not create the output file."
        return
    fi
    log_message "Config file created successfully."

    echo -e "\e[33mSETUP: Please move the fader for Channel 1 to its minimum position (-oo) again.\e[0m"
    read -p "Press Enter to run the test using the loaded config..."

    log_message "Running fade-in using loaded config..."
    # Note: We are NOT providing --ip or --fader, it should come from the file
    "$binary_path" --load-config "$config_file"

    echo "VERIFY: Did the fader for Channel 1 smoothly rise to the 0dB mark (based on the loaded config)?"
    read -p "(y/n) " result3
    if [[ "$result3" == "y" ]]; then
        log_message "Test 2 PASSED (User reported success)."
    else
        log_message "Test 2 FAILED (User reported failure)."
    fi

    # Teardown
    rm "$config_file"
    log_message "Cleaned up test file."

    log_message "--- Finished x32_fade tests ---"
}
