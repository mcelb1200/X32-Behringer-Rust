# Test module for x32_commander
#
# Credits
#
# *   Original concept and work on the C library: Patrick-Gilles Maillot
# *   Additional concepts by: [User]
# *   Rust implementation by: [User]

function Test-X32Commander {
    Log-Message "--- Running x32_commander tests ---"

    $binaryPath = ".\target\release\x32_commander.exe"
    if (-not (Test-Path $binaryPath)) {
        Log-Message "ERROR: x32_commander.exe not found. Please compile first."
        return
    }

    if ($Global:X32Connection.Type -eq "None") {
        Log-Message "Skipping x32_commander tests as no connection is available."
        return
    }

    # --- Setup: Create synthetic command file ---
    $commandFile = "test_commander.txt"
    $commandContent = "O~~~/ch/01/mix/fader|/ch/02/mix/fader ,f 1.0"
    Set-Content -Path $commandFile -Value $commandContent
    Log-Message "Created synthetic command file: $commandFile"

    # --- Test 1: Interactive OSC Mapping Test ---
    Log-Message "Test 1: Interactive OSC Mapping Test"
    Write-Host "This test will start the X32Commander to map Channel 1 fader to Channel 2." -ForegroundColor Yellow
    Write-Host "Please have the X32 console or X32-Edit visible." -ForegroundColor Yellow
    Read-Host "Press Enter to begin the test..."

    Log-Message "Starting x32_commander..."
    $process = Start-Process -FilePath $binaryPath -ArgumentList "--ip $($Global:X32Connection.IPAddress) --file $commandFile" -PassThru

    if ($null -eq $process) {
        Log-Message "Test 1 FAILED: Could not start x32_commander process."
        Remove-Item $commandFile
        return
    }

    Write-Host "X32Commander is running."
    Write-Host "Please move the fader for Channel 1 on the console."
    Write-Host "Observe if the fader for Channel 2 moves to the maximum position."

    $result = Read-Host "Did the fader for Channel 2 move to max when you moved Channel 1? (y/n)"
    if ($result -eq 'y') {
        Log-Message "Test 1 PASSED (User reported success)."
    } else {
        Log-Message "Test 1 FAILED (User reported failure)."
    }

    # --- Teardown ---
    Log-Message "Stopping x32_commander..."
    Stop-Process -Id $process.Id -Force
    Remove-Item $commandFile
    Log-Message "Cleaned up test files."

    Log-Message "--- Finished x32_commander tests ---"
}
