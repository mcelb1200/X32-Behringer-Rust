# Test module for x32_geq2_cpy

function Test-X32Geq2Cpy {
    Log-Message "--- Running x32_geq2_cpy tests ---"

    $binaryPath = ".\target\release\x32_geq2_cpy.exe"
    if (-not (Test-Path $binaryPath)) {
        Log-Message "ERROR: x32_geq2_cpy.exe not found. Please compile first."
        return
    }

    if ($Global:X32Connection.Type -eq "None") {
        Log-Message "Skipping x32_geq2_cpy tests as no connection is available."
        return
    }

    $ip = $Global:X32Connection.IPAddress
    $testSlot = 1 # Assuming FX slot 1 for the test

    # --- Test 1: Copy GEQ from side A to B ---
    Log-Message "Test 1: Copy GEQ from side A to B"
    Write-Host "This test will verify the 'ato-b' copy direction." -ForegroundColor Yellow
    Write-Host "SETUP: Please go to FX slot $testSlot and insert a 'Dual Graphic EQ'." -ForegroundColor Yellow
    Write-Host "On side A, create a distinctive EQ curve (e.g., boost some bands, cut others)." -ForegroundColor Yellow
    Write-Host "Leave side B completely flat (all bands at 0dB)." -ForegroundColor Yellow
    Read-Host "Press Enter when you have set up the GEQ..."

    Log-Message "Running: x32_geq2_cpy --from $testSlot --direction ato-b"
    & $binaryPath --ip $ip --from $testSlot --direction ato-b

    Write-Host "VERIFY: Please check the GEQ in FX slot $testSlot. Has the curve from side A been copied to side B?"
    $result1 = Read-Host "(y/n)"
    if ($result1 -eq 'y') {
        Log-Message "Test 1 PASSED (User reported success)."
    } else {
        Log-Message "Test 1 FAILED (User reported failure)."
    }
    Read-Host "Press Enter for the next test..."


    # --- Test 2: Reset GEQ ---
    Log-Message "Test 2: Reset GEQ"
    Write-Host "This test will verify the 'reset' direction." -ForegroundColor Yellow
    Write-Host "SETUP: Please modify the EQ curve on both side A and side B to be non-flat." -ForegroundColor Yellow
    Read-Host "Press Enter when you have modified the GEQ..."

    Log-Message "Running: x32_geq2_cpy --from $testSlot --direction reset"
    & $binaryPath --ip $ip --from $testSlot --direction reset

    Write-Host "VERIFY: Please check the GEQ in FX slot $testSlot. Have both side A and side B been reset to a flat curve?"
    $result2 = Read-Host "(y/n)"
    if ($result2 -eq 'y') {
        Log-Message "Test 2 PASSED (User reported success)."
    } else {
        Log-Message "Test 2 FAILED (User reported failure)."
    }

    Log-Message "--- Finished x32_geq2_cpy tests ---"
}
