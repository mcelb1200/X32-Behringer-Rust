# Test module for x32_desk_restore

function Test-X32DeskRestore {
    Log-Message "--- Running x32_desk_restore tests ---"

    $restoreBinary = Join-Path $Global:BinaryPath "x32_desk_restore.exe"
    $saveBinary = Join-Path $Global:BinaryPath "x32_desk_save.exe"

    if (-not (Test-Path $restoreBinary)) {
        Log-Message "ERROR: x32_desk_restore.exe not found. Please compile first."
        return
    }
    if (-not (Test-Path $saveBinary)) {
        Log-Message "ERROR: x32_desk_save.exe not found. This test requires it to generate a restore file."
        return
    }

    if ($Global:X32Connection.Type -eq "None") {
        Log-Message "Skipping x32_desk_restore tests as no connection is available."
        return
    }

    $ip = $Global:X32Connection.IPAddress
    $testFile = "test_desk_state.txt"

    # --- Test 1: Full Save and Restore Cycle ---
    Log-Message "Test 1: Full Save and Restore Cycle"
    Write-Host "This test will verify the 'x32_desk_save' and 'x32_desk_restore' workflow." -ForegroundColor Yellow
    Write-Host "SETUP: Please go to Channel 1 and change its name to 'DESK_SAVE_TEST'." -ForegroundColor Yellow
    Read-Host "Press Enter when you have set the channel name..."

    Log-Message "Running x32_desk_save to capture the current state..."
    & $saveBinary --ip $ip $testFile

    if (-not (Test-Path $testFile)) {
        Log-Message "Test 1 FAILED: The save command did not create the output file."
        return
    }
    Log-Message "Save file created successfully."

    Write-Host "SETUP 2: Now, please change the name of Channel 1 to 'MODIFIED'." -ForegroundColor Yellow
    Read-Host "Press Enter when you have changed the name..."

    Log-Message "Running x32_desk_restore to restore the previous state..."
    & $restoreBinary --ip $ip $testFile

    Write-Host "VERIFY: Please check if the name of Channel 1 has been restored to 'DESK_SAVE_TEST'."
    $result = Read-Host "Did the restore succeed? (y/n)"
    if ($result -eq 'y') {
        Log-Message "Test 1 PASSED (User reported success)."
    } else {
        Log-Message "Test 1 FAILED (User reported failure)."
    }

    # --- Teardown ---
    Remove-Item $testFile
    Log-Message "Cleaned up test files."

    Log-Message "--- Finished x32_desk_restore tests ---"
}
