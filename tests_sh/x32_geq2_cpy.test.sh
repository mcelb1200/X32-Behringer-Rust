#!/bin/bash

# Test module for x32_geq2_cpy

test_x32_geq2_cpy() {
    log_message "--- Running x32_geq2_cpy tests ---"

    local binary_path="./target/release/x32_geq2_cpy"
    if [ ! -f "$binary_path" ]; then
        log_message "ERROR: x32_geq2_cpy not found. Please compile first."
        return
    fi

    if [ "$X32_CONNECTION_TYPE" = "None" ]; then
        log_message "Skipping x32_geq2_cpy tests as no connection is available."
        return
    fi

    local ip="$X32_IP_ADDRESS"
    local test_slot=1 # Assuming FX slot 1 for the test

    # --- Test 1: Copy GEQ from side A to B ---
    log_message "Test 1: Copy GEQ from side A to B"
    echo -e "\e[33mThis test will verify the 'ato-b' copy direction.\e[0m"
    echo -e "\e[33mSETUP: Please go to FX slot $test_slot and insert a 'Dual Graphic EQ'.\e[0m"
    echo -e "\e[33mOn side A, create a distinctive EQ curve (e.g., boost some bands, cut others).\e[0m"
    echo -e "\e[33mLeave side B completely flat (all bands at 0dB).\e[0m"
    read -p "Press Enter when you have set up the GEQ..."

    log_message "Running: x32_geq2_cpy --from $test_slot --direction ato-b"
    "$binary_path" --ip "$ip" --from "$test_slot" --direction ato-b

    echo "VERIFY: Please check the GEQ in FX slot $test_slot. Has the curve from side A been copied to side B?"
    read -p "(y/n) " result1
    if [[ "$result1" == "y" ]]; then
        log_message "Test 1 PASSED (User reported success)."
    else
        log_message "Test 1 FAILED (User reported failure)."
    fi
    read -p "Press Enter for the next test..."


    # --- Test 2: Reset GEQ ---
    log_message "Test 2: Reset GEQ"
    echo -e "\e[33mThis test will verify the 'reset' direction.\e[0m"
    echo -e "\e[33mSETUP: Please modify the EQ curve on both side A and side B to be non-flat.\e[0m"
    read -p "Press Enter when you have modified the GEQ..."

    log_message "Running: x32_geq2_cpy --from $test_slot --direction reset"
    "$binary_path" --ip "$ip" --from "$test_slot" --direction reset

    echo "VERIFY: Please check the GEQ in FX slot $test_slot. Have both side A and side B been reset to a flat curve?"
    read -p "(y/n) " result2
    if [[ "$result2" == "y" ]]; then
        log_message "Test 2 PASSED (User reported success)."
    else
        log_message "Test 2 FAILED (User reported failure)."
    fi

    log_message "--- Finished x32_geq2_cpy tests ---"
}
