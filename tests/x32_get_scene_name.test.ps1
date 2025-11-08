# Test module for x32_get_scene_name

function Test-X32GetSceneName {
    Log-Message "--- Running x32_get_scene_name tests ---"

    $binaryPath = ".\target\release\x32_get_scene_name.exe"
    if (-not (Test-Path $binaryPath)) {
        Log-Message "ERROR: x32_get_scene_name.exe not found. Please compile first."
        return
    }

    if ($Global:X32Connection.Type -eq "None") {
        Log-Message "Skipping x32_get_scene_name tests as no connection is available."
        return
    }

    $ip = $Global:X32Connection.IPAddress

    # --- Test 1: Interactive Scene Change Detection ---
    Log-Message "Test 1: Interactive Scene Change Detection"
    Write-Host "This test will verify that the tool can detect a scene change." -ForegroundColor Yellow

    Log-Message "Starting x32_get_scene_name in the background..."
    $process = Start-Process -FilePath $binaryPath -ArgumentList "--ip $ip --onetime 1 --verbose 0" -PassThru -NoNewWindow -RedirectStandardOutput "scene_name_output.txt"

    Write-Host "The scene name listener is running."
    Write-Host "ACTION: On the X32 console, please load any scene." -ForegroundColor Yellow
    Write-Host "The test will complete automatically once the scene change is detected."

    # Wait for the process to exit (onetime=1)
    $process.WaitForExit()
    Log-Message "Process has exited. Checking output..."

    # Verification
    if (-not (Test-Path "scene_name_output.txt")) {
        Log-Message "Test 1 FAILED: The output file was not created."
        return
    }

    $output = Get-Content "scene_name_output.txt"
    if ($output.Length -eq 0) {
        Log-Message "Test 1 FAILED: The tool did not produce any output."
    } elseif ($output[0] -match "\d{2} - .*") {
        Log-Message "Test 1 PASSED: Scene change detected and output is in the correct format."
        Log-Message "Detected: $($output[0])"
    } else {
        Log-Message "Test 1 FAILED: The output format is incorrect."
        Log-Message "Actual output: $($output[0])"
    }

    # --- Teardown ---
    Remove-Item "scene_name_output.txt"
    Log-Message "Cleaned up test file."

    Log-Message "--- Finished x32_get_scene_name tests ---"
}
