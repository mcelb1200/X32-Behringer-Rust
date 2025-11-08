#!/bin/bash

# Test module for x32_automix

test_x32_automix() {
    log_message "--- Running x32_automix tests ---"

    local binary_path="./target/release/x32_automix"
    if [ ! -f "$binary_path" ]; then
        log_message "ERROR: x32_automix not found. Please compile first."
        return
    fi

    if [ "$X32_CONNECTION_TYPE" = "None" ]; then
        log_message "Skipping x32_automix tests as no connection is available."
        return
    fi

    # Test 1: Interactive Automix Smoke Test
    log_message "Test 1: Interactive Automix Smoke Test"
    echo -e "\e[33mThis test will start the automixer. Please have the X32 console or X32-Edit visible.\e[0m"
    echo -e "\e[33mYou will need a microphone connected to Channel 1.\e[0m"
    read -p "Press Enter to begin the test..."

    log_message "Starting x32_automix with default settings..."
    "$binary_path" --ip "$X32_IP_ADDRESS" &
    local pid=$!

    if ! ps -p $pid > /dev/null; then
        log_message "Test 1 FAILED: Could not start x32_automix process."
        return
    fi

    echo "Automixer is running. Please speak into the microphone on Channel 1."
    echo "Observe if the fader for Channel 1 rises."

    echo "Now, please remain silent for about 5-10 seconds."
    echo "Observe if the fader for Channel 1 lowers automatically."

    read -p "Did the fader rise when you spoke and lower when you were silent? (y/n) " result
    if [[ "$result" == "y" ]]; then
        log_message "Test 1 PASSED (User reported success)."
    else
        log_message "Test 1 FAILED (User reported failure)."
    fi

    log_message "Stopping x32_automix..."
    kill $pid
    wait $pid 2>/dev/null

    log_message "--- Finished x32_automix tests ---"
}
