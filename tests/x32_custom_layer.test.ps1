# Test module for x32_custom_layer

function Test-X32CustomLayer {
    Log-Message "--- Running x32_custom_layer tests ---"

    $binaryPath = Join-Path $Global:BinaryPath "x32_custom_layer.exe"
    if (-not (Test-Path $binaryPath)) {
        Log-Message "ERROR: x32_custom_layer.exe not found. Please compile first."
        return
    }

    if ($Global:X32Connection.Type -eq "None") {
        Log-Message "Skipping x32_custom_layer tests as no connection is available."
        return
    }

    $ip = $Global:X32Connection.IPAddress
    $testFile = "test_layer.chn"

    # --- Test 1: Save and Restore Layer ---
    Log-Message "Test 1: Save and Restore Layer"
    Write-Host "This test will verify the 'save' and 'restore' commands." -ForegroundColor Yellow
    Write-Host "SETUP: Please go to Channel 1 and change its name to 'SAVED_STATE'." -ForegroundColor Yellow
    Read-Host "Press Enter when you have set the channel name..."

    Log-Message "Running: x32_custom_layer save --file $testFile"
    & $binaryPath --ip $ip save --file $testFile

    if (-not (Test-Path $testFile)) {
        Log-Message "Test 1 FAILED: The save command did not create the output file."
        return
    }
    Log-Message "Save file created successfully."

    Write-Host "SETUP 2: Now, please change the name of Channel 1 to 'MODIFIED'." -ForegroundColor Yellow
    Read-Host "Press Enter when you have changed the name again..."

    Log-Message "Running: x32_custom_layer restore --file $testFile"
    & $binaryPath --ip $ip restore --file $testFile

    Write-Host "VERIFY: Please check if the name of Channel 1 has been restored to 'SAVED_STATE'."
    $result1 = Read-Host "Did the restore succeed? (y/n)"
    if ($result1 -eq 'y') {
        Log-Message "Test 1 PASSED (User reported success)."
    } else {
        Log-Message "Test 1 FAILED (User reported failure)."
    }
    Remove-Item $testFile
    Read-Host "Press Enter for the next test..."


    # --- Test 2: Reset Channel ---
    Log-Message "Test 2: Reset Channel"
    Write-Host "This test will verify the 'reset' command." -ForegroundColor Yellow
    Write-Host "SETUP: Please go to Channel 2 and change its name to 'TO_BE_RESET'." -ForegroundColor Yellow
    Read-Host "Press Enter when you have set the channel name..."

    Log-Message "Running: x32_custom_layer reset --channels 2"
    & $binaryPath --ip $ip reset --channels "2"

    Write-Host "VERIFY: Please check if Channel 2 has been reset to its default state (name is cleared or default)."
    $result2 = Read-Host "Did the reset succeed? (y/n)"
    if ($result2 -eq 'y') {
        Log-Message "Test 2 PASSED (User reported success)."
    } else {
        Log-Message "Test 2 FAILED (User reported failure)."
    }

    Log-Message "--- Finished x32_custom_layer tests ---"
}
