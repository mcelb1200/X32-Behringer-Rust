#!/bin/bash

# Test module for x32_wav_xlive

# Helper function to generate a minimal, valid, 32-bit mono WAV file using Python.
# Using Python for this is simpler and more reliable than trying to construct binary data in pure Bash.
generate_mono_wav_file() {
    local path="$1"
    local sample_rate=${2:-48000}
    local duration_seconds=${3:-1}

    python3 -c "
import wave
import struct

path = '$path'
sample_rate = $sample_rate
duration = $duration_seconds
num_channels = 1
bits_per_sample = 32
num_samples = sample_rate * duration
block_align = num_channels * (bits_per_sample // 8)
byte_rate = sample_rate * block_align

with wave.open(path, 'wb') as wf:
    wf.setnchannels(num_channels)
    wf.setsampwidth(bits_per_sample // 8)
    wf.setframerate(sample_rate)

    # Write silent audio data (32-bit samples are 4 bytes)
    sample = struct.pack('<i', 0) # Little-endian 32-bit signed integer
    for _ in range(num_samples):
        wf.writeframesraw(sample)
"
}

test_x32_wav_xlive() {
    log_message "--- Running x32_wav_xlive tests ---"

    local binary_path="./target/release/x32_wav_xlive"
    if [ ! -f "$binary_path" ]; then
        log_message "ERROR: x32_wav_xlive not found. Please compile first."
        return
    fi

    if ! command -v python3 &> /dev/null; then
        log_message "WARNING: 'python3' is not installed. Skipping x32_wav_xlive test."
        return
    fi

    if ! command -v hexdump &> /dev/null; then
        log_message "WARNING: 'hexdump' is not installed. Skipping x32_wav_xlive test."
        return
    fi

    if ! command -v sox &> /dev/null; then
        log_message "WARNING: 'sox' is not installed. Some tests may be skipped."
    fi

    local test_dir="temp_wav_source"

    # --- Setup ---
    rm -rf "$test_dir"
    mkdir -p "$test_dir"
    log_message "Created temporary directory: $test_dir"

    # Generate two mono WAV files
    generate_mono_wav_file "$test_dir/ch_1.wav"
    generate_mono_wav_file "$test_dir/ch_2.wav"
    log_message "Generated synthetic mono WAV files."

    # --- Test 1: Session Creation and WAV file verification ---
    log_message "Test 1: Session Creation and WAV file verification"

    log_message "Running: x32_wav_xlive $test_dir"
    "$binary_path" "$test_dir" --silent

    # Verification
    local session_dir=$(find "$test_dir" -mindepth 1 -maxdepth 1 -type d)
    if [ -z "$session_dir" ]; then
        log_message "Test 1 FAILED: No session directory was created."
        rm -rf "$test_dir"
        return
    fi
    log_message "Session directory found: $(basename "$session_dir")"

    if [ ! -f "$session_dir/SE_LOG.BIN" ]; then
        log_message "Test 1 FAILED: SE_LOG.BIN was not found in the session directory."
        rm -rf "$test_dir"
        return
    fi
    log_message "SE_LOG.BIN found."

    local wav_file=$(find "$session_dir" -name "*.wav")
    if [ -z "$wav_file" ]; then
        log_message "Test 1 FAILED: No .wav file was found in the session directory."
        rm -rf "$test_dir"
        return
    fi
    log_message "Multichannel WAV file found: $(basename "$wav_file")"

    # Read the header of the output WAV to verify its properties using 'hexdump'
    local header=$(hexdump -s 22 -n 2 -e '1/2 "%d"' "$wav_file") # num channels
    local num_channels=$(echo "$header" | awk '{print $1}')

    header=$(hexdump -s 34 -n 2 -e '1/2 "%d"' "$wav_file") # bits per sample
    local bits_per_sample=$(echo "$header" | awk '{print $1}')

    if [[ "$num_channels" -eq 2 && "$bits_per_sample" -eq 32 ]]; then
        log_message "Test 1 PASSED: Output WAV file has the correct properties (2 channels, 32-bit)."
    else
        log_message "Test 1 FAILED: Output WAV file has incorrect properties."
        log_message "Expected: 2 Channels, 32 Bits per Sample"
        log_message "Actual:   $num_channels Channels, $bits_per_sample Bits per Sample"
    fi

    # --- Teardown ---
    rm -rf "$test_dir"
    log_message "Cleaned up temporary directory."

    log_message "--- Finished x32_wav_xlive tests ---"
}
