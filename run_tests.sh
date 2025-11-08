#!/bin/bash

# --- Configuration ---
LOG_FILE="x32_test_log_bash.txt"
BINARIES=(
    "x32_emulator"
    "x32_automix"
    "x32_commander"
    "x32_copy_fx"
    "x32_custom_layer"
    "x32_desk_restore"
    "x32_desk_save"
    "x32_fade"
    "x32_geq2_cpy"
    "x32_get_scene"
    "x32_get_scene_name"
    "x32_set_scene"
    "x32_tcp"
    "x32_usb"
    "x32_wav_xlive"
)
X32_CONNECTION_TYPE="None"
X32_IP_ADDRESS=""

# --- Logging ---
log_message() {
    local message="$1"
    local timestamp=$(date +"%Y-%m-%d %H:%M:%S")
    echo "[$timestamp] $message" | tee -a "$LOG_FILE"
}

# --- Compilation ---
compile_binaries() {
    log_message "Starting compilation of all binaries..."
    for binary in "${BINARIES[@]}"; do
        log_message "Compiling $binary..."
        cargo build --package "$binary" --release
        if [ $? -ne 0 ]; then
            log_message "ERROR: Compilation of $binary failed."
            return 1
        fi
    done
    log_message "Compilation complete."
    return 0
}

# --- X32 Connection Detection (Linux) ---
detect_x32_connection() {
    log_message "Attempting to detect X32 connection..."

    # 1. Check for USB Connection
    log_message "Checking for USB devices..."
    if lsusb | grep -q "1397:"; then
        log_message "Found Behringer USB device. Assuming X32 is connected via USB."
        X32_CONNECTION_TYPE="USB"
        X32_IP_ADDRESS=""
        return
    fi

    # 2. Check for Network Connection
    log_message "Checking for network devices... This may take a few minutes."
    if command -v nmap &> /dev/null; then
        local subnet=$(ip -o -f inet addr show | awk '/scope global/ {print $4}' | head -n 1)
        if [ -n "$subnet" ]; then
            log_message "Scanning subnet $subnet for X32 on port 10023..."
            local found_ip=$(nmap -p 10023 --open -n "$subnet" | awk '/Nmap scan report for/{ip=$NF} /10023\/open/{print ip}' | head -n 1)
            if [ -n "$found_ip" ]; then
                log_message "Found X32 at network address: $found_ip"
                X32_CONNECTION_TYPE="Network"
                X32_IP_ADDRESS="$found_ip"
                return
            fi
        fi
    else
        log_message "Warning: 'nmap' is not installed. Skipping network scan. Please install it (e.g., 'sudo apt-get install nmap')."
    fi

    # 3. Prompt user if auto-detection fails
    log_message "Could not auto-detect X32 connection."
    echo "Could not automatically detect the X32."
    echo "Please select the connection method:"
    echo "1. Network"
    echo "2. USB"
    echo "s. Skip (run tests without a device)"
    read -p "Enter your choice: " conn_choice

    case "$conn_choice" in
        1)
            read -p "Please enter the X32 IP address: " ip
            if [[ "$ip" =~ ^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
                X32_CONNECTION_TYPE="Network"
                X32_IP_ADDRESS="$ip"
            else
                echo "Invalid IP address format."
            fi
            ;;
        2)
            X32_CONNECTION_TYPE="USB"
            X32_IP_ADDRESS=""
            ;;
        s)
            X32_CONNECTION_TYPE="None"
            X32_IP_ADDRESS=""
            ;;
        *)
            echo "Invalid selection."
            ;;
    esac
}


# --- Main Menu (TUI) ---
show_main_menu() {
    clear
    echo "X32 Rust Binaries - Test Suite (Linux/Bash)"
    echo "-------------------------------------------"
    echo "Connection Status: $X32_CONNECTION_TYPE $X32_IP_ADDRESS"
    echo "1. Compile all binaries"
    echo "2. Detect X32 connection"
    echo "3. Run all tests"
    echo "4. Run specific test..."
    echo "q. Quit"
    read -p "Enter your choice: "
}

# --- Main Loop ---
while true; do
    show_main_menu
    case "$REPLY" in
        1)
            compile_binaries
            read -p "Press Enter to continue..."
            ;;
        2)
            detect_x32_connection
            read -p "Press Enter to continue..."
            ;;
        3)
            log_message "Running all tests..."
            if [ "$X32_CONNECTION_TYPE" = "None" ]; then
                detect_x32_connection
            fi

            for test_file in tests_sh/*.test.sh; do
                if [ -f "$test_file" ]; then
                    source "$test_file"
                    # The function name is derived from the file name
                    local test_function_name=$(basename "$test_file" .test.sh | tr '-' '_')
                    "test_$test_function_name"
                fi
            done
            read -p "All tests finished. Press Enter to continue..."
            ;;
        4)
            log_message "Running specific test..."
            if [ "$X32_CONNECTION_TYPE" = "None" ]; then
                detect_x32_connection
            fi

            echo "Select a test to run:"
            local i=1
            local test_files=()
            for test_file in tests_sh/*.test.sh; do
                if [ -f "$test_file" ]; then
                    echo "$i. $(basename "$test_file" .test.sh)"
                    test_files+=("$test_file")
                    ((i++))
                fi
            done

            read -p "Enter your choice: " test_choice
            local selected_file=${test_files[$((test_choice-1))]}

            if [ -n "$selected_file" ]; then
                source "$selected_file"
                local test_function_name=$(basename "$selected_file" .test.sh | tr '-' '_')
                "test_$test_function_name"
            else
                echo "Invalid selection."
            fi
            read -p "Test finished. Press Enter to continue..."
            ;;
        q)
            break
            ;;
        *)
            echo "Invalid option. Please try again."
            read -p "Press Enter to continue..."
            ;;
    esac
done
