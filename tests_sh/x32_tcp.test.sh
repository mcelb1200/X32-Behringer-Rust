#!/bin/bash

# Test module for x32_tcp
#
# Credits
#
# *   Original concept and work on the C library: Patrick-Gilles Maillot
# *   Additional concepts by: [User]
# *   Rust implementation by: [User]

test_x32_tcp() {
    log_message "--- Running x32_tcp tests ---"

    local binary_path="./target/release/x32_tcp"
    if [ ! -f "$binary_path" ]; then
        log_message "ERROR: x32_tcp not found. Please compile first."
        return
    fi

    if [ "$X32_CONNECTION_TYPE" = "None" ]; then
        log_message "Skipping x32_tcp tests as no connection is available."
        return
    fi

    if ! command -v nc &> /dev/null; then
        log_message "WARNING: 'nc' (netcat) is not installed. Skipping x32_tcp test."
        return
    fi

    local ip="$X32_IP_ADDRESS"
    local server_port=10041 # Default port for x32_tcp

    # --- Test 1: TCP Bridge Functional Test ---
    log_message "Test 1: TCP Bridge Functional Test"

    # 1. Start the server in the background
    log_message "Starting x32_tcp server in the background..."
    "$binary_path" --ip "$ip" --port "$server_port" &
    local pid=$!
    sleep 2 # Give the server time to start up

    if ! ps -p $pid > /dev/null; then
        log_message "Test 1 FAILED: Could not start x32_tcp process."
        return
    fi

    # 2. Use netcat to send a command and get the response
    local response
    response=$(echo "/info" | nc -w 5 localhost "$server_port")

    # 3. Teardown the server
    log_message "Stopping x32_tcp server..."
    kill $pid
    wait $pid 2>/dev/null

    # 4. Verification
    log_message "Client received: $response"
    if [[ "$response" == /info* ]]; then
        log_message "Test 1 PASSED: Received a valid /info response from the bridge."
    else
        log_message "Test 1 FAILED: The response was not in the expected format."
        log_message "Expected a string starting with '/info', but got: $response"
    fi

    log_message "--- Finished x32_tcp tests ---"
}
