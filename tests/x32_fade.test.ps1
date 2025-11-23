# Test module for x32_fade
#
# Credits
#
# *   Original concept and work on the C library: Patrick-Gilles Maillot
# *   Additional concepts by: [User]
# *   Rust implementation by: [User]

function Test-X32Fade {
    Log-Message "--- Running x32_fade tests ---"

    $binaryPath = ".\target\release\x32_fade.exe"
    if (-not (Test-Path $binaryPath)) {
        Log-Message "ERROR: x32_fade.exe not found. Please compile first."
        return
    }

    if ($Global:X32Connection.Type -eq "None") {
        Log-Message "Skipping x32_fade tests as no connection is available."
        return
    }

    $ip = $Global:X32Connection.IPAddress
    $testFader = "/ch/01/mix/fader"
    $configFile = "test_fade_config.json"

    # --- Test 1: Interactive Fade In and Fade Out ---
    Log-Message "Test 1: Interactive Fade In and Fade Out"
    Write-Host "This test will verify the fade-in and fade-out functionality." -ForegroundColor Yellow
    Write-Host "SETUP: Please move the fader for Channel 1 to its minimum position (-oo)." -ForegroundColor Yellow
    Read-Host "Press Enter to begin the fade-in test..."

    Log-Message "Running fade-in on $testFader..."
    & $binaryPath --ip $ip --fader $testFader --fade-in 2.0 --steps 20

    Write-Host "VERIFY: Did the fader for Channel 1 smoothly rise to the 0dB mark?"
    $result1 = Read-Host "(y/n)"
    if ($result1 -eq 'y') {
        Log-Message "Fade-in test PASSED (User reported success)."
    } else {
        Log-Message "Fade-in test FAILED (User reported failure)."
    }

    Read-Host "Press Enter to begin the fade-out test..."

    Log-Message "Running fade-out on $testFader..."
    & $binaryPath --ip $ip --fader $testFader --fade-out 2.0 --steps 20

    Write-Host "VERIFY: Did the fader for Channel 1 smoothly lower to its minimum position?"
    $result2 = Read-Host "(y/n)"
    if ($result2 -eq 'y') {
        Log-Message "Fade-out test PASSED (User reported success)."
    } else {
        Log-Message "Fade-out test FAILED (User reported failure)."
    }
    Read-Host "Press Enter for the next test..."


    # --- Test 2: Save and Load Config ---
    Log-Message "Test 2: Save and Load Config"
    Write-Host "This test will verify the --save-config and --load-config functionality." -ForegroundColor Yellow

    # Setup: Create the config file
    if (Test-Path $configFile) { Remove-Item $configFile }
    Log-Message "Saving configuration to $configFile..."
    & $binaryPath --ip $ip --fader $testFader --fade-in 2.0 --save-config $configFile

    if (-not (Test-Path $configFile)) {
        Log-Message "Test 2 FAILED: The save-config command did not create the output file."
        return
    }
    Log-Message "Config file created successfully."

    Write-Host "SETUP: Please move the fader for Channel 1 to its minimum position (-oo) again." -ForegroundColor Yellow
    Read-Host "Press Enter to run the test using the loaded config..."

    Log-Message "Running fade-in using loaded config..."
    # Note: We are NOT providing --ip or --fader, it should come from the file
    & $binaryPath --load-config $configFile

    Write-Host "VERIFY: Did the fader for Channel 1 smoothly rise to the 0dB mark (based on the loaded config)?"
    $result3 = Read-Host "(y/n)"
    if ($result3 -eq 'y') {
        Log-Message "Test 2 PASSED (User reported success)."
    } else {
        Log-Message "Test 2 FAILED (User reported failure)."
    }

    # Teardown
    Remove-Item $configFile
    Log-Message "Cleaned up test file."

    Log-Message "--- Finished x32_fade tests ---"
}
