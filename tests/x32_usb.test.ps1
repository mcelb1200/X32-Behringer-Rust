# Test module for x32_usb
#
# Credits
#
# *   Original concept and work on the C library: Patrick-Gilles Maillot
# *   Additional concepts by: [User]
# *   Rust implementation by: [User]

function Test-X32Usb {
    Log-Message "--- Running x32_usb tests ---"

    $binaryPath = ".\target\release\x32_usb.exe"
    if (-not (Test-Path $binaryPath)) {
        Log-Message "ERROR: x32_usb.exe not found. Please compile first."
        return
    }

    if ($Global:X32Connection.Type -eq "None") {
        Log-Message "Skipping x32_usb tests as no connection is available."
        return
    }

    $ip = $Global:X32Connection.IPAddress

    # --- Test 1: Interactive 'ls' command test ---
    Log-Message "Test 1: Interactive 'ls' command test"
    Write-Host "This test will verify the 'ls' command for listing USB drive contents." -ForegroundColor Yellow
    Write-Host "SETUP: Please insert a USB drive with some files/folders into the X32 console." -ForegroundColor Yellow
    Read-Host "Press Enter when the USB drive is inserted and ready..."

    Log-Message "Running: x32_usb ls"
    $output = & $binaryPath --ip $ip ls 2>&1 | Tee-Object -Variable lsOutput

    Write-Host "--- Output from 'x32_usb ls' ---"
    Write-Host $output
    Write-Host "----------------------------------"

    Write-Host "VERIFY: Does the output above correctly list the contents of your USB drive?"
    $result = Read-Host "(y/n)"
    if ($result -eq 'y') {
        Log-Message "Test 1 PASSED (User reported success)."
    } else {
        Log-Message "Test 1 FAILED (User reported failure)."
        Log-Message "Captured output: $lsOutput"
    }

    Log-Message "--- Finished x32_usb tests ---"
}
