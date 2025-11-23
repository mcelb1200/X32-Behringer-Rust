# Test module for x32_emulator
#
# Credits
#
# *   Original concept and work on the C library: Patrick-Gilles Maillot
# *   Additional concepts by: [User]
# *   Rust implementation by: [User]

function Test-X32Emulator {
    Log-Message "--- Running x32_emulator tests ---"

    $binaryPath = ".\target\release\x32_emulator.exe"
    if (-not (Test-Path $binaryPath)) {
        Log-Message "ERROR: x32_emulator.exe not found. Please compile first."
        return
    }

    # Test 1: Start and stop the emulator
    Log-Message "Test 1: Starting emulator..."
    Start-Process -FilePath $binaryPath -NoNewWindow
    Start-Sleep -Seconds 5 # Give it time to start

    $proc = Get-Process | Where-Object { $_.Name -eq "x32_emulator" }
    if ($proc) {
        Log-Message "Emulator process found. Stopping..."
        Stop-Process -Name "x32_emulator"
        Log-Message "Test 1 PASSED."
    } else {
        Log-Message "Test 1 FAILED: Could not find emulator process."
    }

    Log-Message "--- Finished x32_emulator tests ---"
}
