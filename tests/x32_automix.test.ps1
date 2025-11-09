# Test module for x32_automix

function Test-X32Automix {
    Log-Message "--- Running x32_automix tests ---"

    $binaryPath = Join-Path $Global:BinaryPath "x32_automix.exe"
    if (-not (Test-Path $binaryPath)) {
        Log-Message "ERROR: x32_automix.exe not found. Please compile first."
        return
    }

    if ($Global:X32Connection.Type -eq "None") {
        Log-Message "Skipping x32_automix tests as no connection is available."
        return
    }

    # Test 1: Interactive Automix Smoke Test
    Log-Message "Test 1: Interactive Automix Smoke Test"
    Write-Host "This test will start the automixer. Please have the X32 console or X32-Edit visible." -ForegroundColor Yellow
    Write-Host "You will need a microphone connected to Channel 1." -ForegroundColor Yellow
    Read-Host "Press Enter to begin the test..."

    Log-Message "Starting x32_automix with default settings..."
    $process = Start-Process -FilePath $binaryPath -ArgumentList "--ip $($Global:X32Connection.IPAddress)" -PassThru

    if ($null -eq $process) {
        Log-Message "Test 1 FAILED: Could not start x32_automix process."
        return
    }

    Write-Host "Automixer is running. Please speak into the microphone on Channel 1."
    Write-Host "Observe if the fader for Channel 1 rises."

    Write-Host "Now, please remain silent for about 5-10 seconds."
    Write-Host "Observe if the fader for Channel 1 lowers automatically."

    $result = Read-Host "Did the fader rise when you spoke and lower when you were silent? (y/n)"
    if ($result -eq 'y') {
        Log-Message "Test 1 PASSED (User reported success)."
    } else {
        Log-Message "Test 1 FAILED (User reported failure)."
    }

    Log-Message "Stopping x32_automix..."
    Stop-Process -Id $process.Id -Force

    Log-Message "--- Finished x32_automix tests ---"
}
