# Test module for x32_copy_fx

function Test-X32CopyFx {
    Log-Message "--- Running x32_copy_fx tests ---"

    $binaryPath = ".\target\release\x32_copy_fx.exe"
    if (-not (Test-Path $binaryPath)) {
        Log-Message "ERROR: x32_copy_fx.exe not found. Please compile first."
        return
    }

    if ($Global:X32Connection.Type -eq "None") {
        Log-Message "Skipping x32_copy_fx tests as no connection is available."
        return
    }

    $ip = $Global:X32Connection.IPAddress

    # --- Test 1: Copy FX slot ---
    Log-Message "Test 1: Copy FX slot"
    Write-Host "This test will verify the 'copy' command." -ForegroundColor Yellow
    Write-Host "SETUP: Please set FX slot 1 to a 'Hall Reverb' and FX slot 2 to a 'Stereo Delay'." -ForegroundColor Yellow
    Read-Host "Press Enter when you have set up the FX slots..."

    Log-Message "Running: x32_copy_fx --from 1 --to 2 copy"
    & $binaryPath --ip $ip --from 1 --to 2 copy

    Write-Host "VERIFY: Please check if FX slot 2 is now also a 'Hall Reverb' with the same settings as slot 1."
    $result1 = Read-Host "Did the copy succeed? (y/n)"
    if ($result1 -eq 'y') {
        Log-Message "Test 1 PASSED (User reported success)."
    } else {
        Log-Message "Test 1 FAILED (User reported failure)."
    }
    Read-Host "Press Enter for the next test..."

    # --- Test 2: Reset FX slot to defaults ---
    Log-Message "Test 2: Reset FX slot"
    Write-Host "This test will verify the 'reset' command." -ForegroundColor Yellow
    Write-Host "SETUP: In FX slot 1 (Hall Reverb), please change the 'Decay' time to a low value (e.g., 0.5s)." -ForegroundColor Yellow
    Read-Host "Press Enter when you have modified the parameter..."

    Log-Message "Running: x32_copy_fx --from 1 reset"
    & $binaryPath --ip $ip --from 1 reset

    Write-Host "VERIFY: Please check if the 'Decay' time in FX slot 1 has returned to its default value (likely 2.0s)."
    $result2 = Read-Host "Did the reset succeed? (y/n)"
    if ($result2 -eq 'y') {
        Log-Message "Test 2 PASSED (User reported success)."
    } else {
        Log-Message "Test 2 FAILED (User reported failure)."
    }
    Read-Host "Press Enter for the next test..."

    # --- Test 3: Reset FX with custom defaults file ---
    Log-Message "Test 3: Reset with custom defaults"
    Write-Host "This test will verify resetting from a custom defaults file." -ForegroundColor Yellow

    # Setup: Create synthetic defaults file
    $defaultsFile = "test_fx_defaults.txt"
    # This string sets the first parameter (Decay) of a Hall Reverb to max (1.0), and the rest to 0.
    $defaultsContent = "HALL`n1.0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0"
    Set-Content -Path $defaultsFile -Value $defaultsContent
    Log-Message "Created synthetic defaults file with max decay for Hall Reverb."

    Read-Host "Press Enter to run the reset command with the custom file..."

    Log-Message "Running: x32_copy_fx --from 1 reset --defaults-file $defaultsFile"
    & $binaryPath --ip $ip --from 1 reset --defaults-file $defaultsFile

    Write-Host "VERIFY: Please check if the 'Decay' time in FX slot 1 is now set to the maximum possible value (10.0s)."
    $result3 = Read-Host "Did the custom reset succeed? (y/n)"
    if ($result3 -eq 'y') {
        Log-Message "Test 3 PASSED (User reported success)."
    } else {
        Log-Message "Test 3 FAILED (User reported failure)."
    }

    # Teardown
    Remove-Item $defaultsFile
    Log-Message "Cleaned up test files."

    Log-Message "--- Finished x32_copy_fx tests ---"
}
