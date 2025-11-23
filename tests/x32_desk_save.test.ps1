# Test module for x32_desk_save
#
# Credits
#
# *   Original concept and work on the C library: Patrick-Gilles Maillot
# *   Additional concepts by: [User]
# *   Rust implementation by: [User]

function Test-X32DeskSave {
    Log-Message "--- Running x32_desk_save tests ---"

    $binaryPath = ".\target\release\x32_desk_save.exe"
    if (-not (Test-Path $binaryPath)) {
        Log-Message "ERROR: x32_desk_save.exe not found. Please compile first."
        return
    }

    if ($Global:X32Connection.Type -eq "None") {
        Log-Message "Skipping x32_desk_save tests as no connection is available."
        return
    }

    $ip = $Global:X32Connection.IPAddress
    $testFile = "test_desk_save_output.txt"

    # --- Test 1: File Creation and Sanity Check ---
    Log-Message "Test 1: File Creation and Sanity Check"

    # Cleanup from previous runs
    if (Test-Path $testFile) { Remove-Item $testFile }

    Log-Message "Running: x32_desk_save --desk-save $testFile"
    & $binaryPath --ip $ip --desk-save $testFile

    # Verification
    if (-not (Test-Path $testFile)) {
        Log-Message "Test 1 FAILED: The output file '$testFile' was not created."
        return
    }

    $content = Get-Content $testFile
    if ($content.Length -eq 0) {
        Log-Message "Test 1 FAILED: The output file is empty."
    } elseif ($content[0].StartsWith("/")) {
        Log-Message "Test 1 PASSED: Output file created and first line appears to be a valid OSC message."
    } else {
        Log-Message "Test 1 FAILED: The output file does not contain expected OSC data."
        Log-Message "First line was: $($content[0])"
    }

    # --- Teardown ---
    Remove-Item $testFile
    Log-Message "Cleaned up test file."

    Log-Message "--- Finished x32_desk_save tests ---"
}
