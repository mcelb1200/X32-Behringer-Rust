# Test module for x32_emulator

function Test-X32Emulator {
    Log-Message "--- Running x32_emulator tests ---"
    $passed = $true

    # Test 1: Start and stop the emulator
    Log-Message "Test 1: Starting emulator..."
    $emulatorProcess = Start-ManagedProcess -BinaryName "x32_emulator"
    if ($null -eq $emulatorProcess) {
        Log-Message "Test 1 FAILED: Could not start emulator process."
        $passed = $false
    } else {
        Start-Sleep -Seconds 3
        if ($emulatorProcess.Process.HasExited) {
            Log-Message "Test 1 FAILED: Emulator process exited prematurely."
            $passed = $false
        } else {
            Log-Message "Emulator process is running. Test 1 PASSED."
        }
        Stop-ManagedProcess -ManagedProcess $emulatorProcess
    }

    Log-Message "--- Finished x32_emulator tests ---"
    return @{ Passed = $passed }
}
