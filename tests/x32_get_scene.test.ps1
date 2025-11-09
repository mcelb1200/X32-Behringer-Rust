# Test module for x32_get_scene

function Test-X32GetScene {
    Log-Message "--- Running x32_get_scene tests ---"

    $binaryPath = Join-Path $Global:BinaryPath "x32_get_scene.exe"
    if (-not (Test-Path $binaryPath)) {
        Log-Message "ERROR: x32_get_scene.exe not found. Please compile first."
        return
    }

    if ($Global:X32Connection.Type -eq "None") {
        Log-Message "Skipping x32_get_scene tests as no connection is available."
        return
    }

    $ip = $Global:X32Connection.IPAddress
    $inputFile = "test_scene_input.txt"
    $outputFile = "test_scene_output.txt"
    $sceneName = "TestScene"
    $sceneNote = "Test Note"

    # --- Setup: Create synthetic input file ---
    $inputContent = @(
        "/ch/01/config/name",
        "/ch/01/mix/fader"
    )
    Set-Content -Path $inputFile -Value $inputContent
    Log-Message "Created synthetic input file: $inputFile"

    # --- Test 1: File Generation and Content Verification ---
    Log-Message "Test 1: File Generation and Content Verification"

    # Cleanup from previous runs
    if (Test-Path $outputFile) { Remove-Item $outputFile }

    Log-Message "Running: x32_get_scene with redirected input..."
    Get-Content $inputFile | & $binaryPath --ip $ip --scene-name $sceneName --note $sceneNote > $outputFile

    # Verification
    if (-not (Test-Path $outputFile)) {
        Log-Message "Test 1 FAILED: The output file '$outputFile' was not created."
        Remove-Item $inputFile
        return
    }

    $content = Get-Content $outputFile
    if ($content.Length -lt 3) {
        Log-Message "Test 1 FAILED: The output file has insufficient content."
    } elseif ($content[0] -notlike "#2.7# `"$sceneName`" `"$sceneNote`"*") {
        Log-Message "Test 1 FAILED: The scene header is incorrect."
        Log-Message "Expected header starting with: #2.7# `"$sceneName`" `"$sceneNote`""
        Log-Message "Actual header: $($content[0])"
    } elseif (($content[1] -notlike "/ch/01/config/name*") -or ($content[2] -notlike "/ch/01/mix/fader*")) {
        Log-Message "Test 1 FAILED: The output file does not contain the expected OSC responses."
        Log-Message "Actual content: $($content -join "`n")"
    } else {
        Log-Message "Test 1 PASSED: Output file created with correct header and content structure."
    }

    # --- Teardown ---
    Remove-Item $inputFile
    Remove-Item $outputFile
    Log-Message "Cleaned up test files."

    Log-Message "--- Finished x32_get_scene tests ---"
}
