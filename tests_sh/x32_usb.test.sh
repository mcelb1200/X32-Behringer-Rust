#!/bin/bash

# Test module for x32_usb

test_x32_usb() {
    log_message "--- Running x32_usb tests ---"

    local binary_path="./target/release/x32_usb"
    if [ ! -f "$binary_path" ]; then
        log_message "ERROR: x32_usb not found. Please compile first."
        return
    fi

    if [ "$X32_CONNECTION_TYPE" = "None" ]; then
        log_message "Skipping x32_usb tests as no connection is available."
        return
    fi

    local ip="$X32_IP_ADDRESS"

    # --- Test 1: Interactive 'ls' command test ---
    log_message "Test 1: Interactive 'ls' command test"
    echo -e "\e[33mThis test will verify the 'ls' command for listing USB drive contents.\e[0m"
    echo -e "\e[33mSETUP: Please insert a USB drive with some files/folders into the X32 console.\e[0m"
    read -p "Press Enter when the USB drive is inserted and ready..."

    log_message "Running: x32_usb ls"
    local output
    output=$("$binary_path" --ip "$ip" ls 2>&1)

    echo "--- Output from 'x32_usb ls' ---"
    echo "$output"
    echo "----------------------------------"

    echo "VERIFY: Does the output above correctly list the contents of your USB drive?"
    read -p "(y/n) " result
    if [[ "$result" == "y" ]]; then
        log_message "Test 1 PASSED (User reported success)."
    else
        log_message "Test 1 FAILED (User reported failure)."
        log_message "Captured output: $output"
    fi

    log_message "--- Finished x32_usb tests ---"
}
