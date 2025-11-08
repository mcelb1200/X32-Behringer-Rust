# Test module for x32_set_scene

function Test-X32SetScene {
    Log-Message "--- Running x32_set_scene tests ---"

    $binaryPath = ".\target\release\x32_set_scene.exe"
    if (-not (Test-Path $binaryPath)) {
        Log-Message "ERROR: x32_set_scene.exe not found. Please compile first."
        return
    }

    if ($Global:X32Connection.Type -eq "None") {
        Log-Message "Skipping x32_set_scene tests as no connection is available."
        return
    }

    $ip = $Global:X32Connection.IPAddress
    $inputFile = "test_set_scene_input.txt"

    # --- Setup: Create synthetic scene file ---
    $sceneContent = @(
        '/ch/01/config/name "SCENE_SET"',
        '/ch/01/mix/fader ,f 1.0'
    )
    Set-Content -Path $inputFile -Value $sceneContent
    Log-Message "Created synthetic scene file: $inputFile"

    # --- Test 1: Interactive Scene Set ---
    Log-Message "Test 1: Interactive Scene Set"
    Write-Host "This test will verify that the tool can set parameters from a file." -ForegroundColor Yellow
    Write-Host "SETUP: Please go to Channel 1 and ensure its name is cleared and its fader is at the minimum position." -ForegroundColor Yellow
    Read-Host "Press Enter to run the test..."

    Log-Message "Running x32_set_scene with redirected input..."
    Get-Content $inputFile | & $binaryPath --ip $ip --delay 10

    Write-Host "VERIFY: Please check the X32 console."
    Write-Host "Is the name of Channel 1 now 'SCENE_SET' and is its fader at the maximum position (+10dB)?"
    $result = Read-Host "(y/n)"
    if ($result -eq 'y') {
        Log-Message "Test 1 PASSED (User reported success)."
    } else {
        Log-Message "Test 1 FAILED (User reported failure)."
    }

    # --- Teardown ---
    # As a courtesy, reset the channel that was just modified.
    $resetContent = @(
        '/ch/01/config/name ""',
        '/ch/01/mix/fader ,f 0.0'
    )
    $resetContent | & $binaryPath --ip $ip --delay 10

    Remove-Item $inputFile
    Log-Message "Cleaned up test file and reset channel 1."

    Log-Message "--- Finished x32_set_scene tests ---"
}
