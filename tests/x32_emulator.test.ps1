# Test module for x32_emulator
#
# Credits
#
# *   Original concept and work on the C library: Patrick-Gilles Maillot
# *   Additional concepts by: mcelb1200
# *   Rust implementation by: mcelb1200

function Test-X32Emulator {
    Log-Message "--- Running x32_emulator tests ---"

    $binaryPath = ".\target\release\x32_emulator.exe"
    if (-not (Test-Path $binaryPath)) {
        Log-Message "ERROR: x32_emulator.exe not found. Please compile first."
        return
    }

    # Test 1: Start and stop the emulator
    Log-Message "Test 1: Starting emulator..."
    $proc = Start-Process -FilePath $binaryPath -NoNewWindow -PassThru
    Start-Sleep -Seconds 2 # Give it time to start

    if ($proc) {
        Log-Message "Emulator process found. Stopping..."
        Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
        Log-Message "Test 1 PASSED."
    } else {
        Log-Message "Test 1 FAILED: Could not start emulator process."
    }

    Log-Message "--- Finished x32_emulator tests ---"
}
