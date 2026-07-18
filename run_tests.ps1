# Main PowerShell script for testing X32 Rust binaries

# --- Configuration ---
$LogFile = "x32_test_log.txt"
$Binaries = @(
    "x32_emulator",
    "x32_automix",
    "x32_commander",
    "x32_copy_fx",
    "x32_custom_layer",
    "x32_desk_restore",
    "x32_desk_save",
    "x32_fade",
    "x32_geq2_cpy",
    "x32_get_scene",
    "x32_get_scene_name",
    "x32_set_scene",
    "x32_tcp",
    "x32_usb",
    "x32_wav_xlive",
    "x32_ssavergw"
)

# --- Logging ---
function Log-Message {
    param (
        [string]$Message
    )
    $Timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $LogEntry = "[$Timestamp] $Message"
    Add-Content -Path $LogFile -Value $LogEntry
    Write-Host $LogEntry
}

# --- Compilation ---
function Compile-Binaries {
    Log-Message "Starting compilation of all binaries..."
    foreach ($binary in $Binaries) {
        Log-Message "Compiling $binary..."
        cargo build --package $binary --release
        if ($LASTEXITCODE -ne 0) {
            Log-Message "ERROR: Compilation of $binary failed."
            return $false
        }
    }
    Log-Message "Compilation complete."
    return $true
}

# --- X32 Connection ---
$Global:X32Connection = $null

function Detect-X32Connection {
    Log-Message "Attempting to detect X32 connection..."

    # 1. Check for USB Connection
    Log-Message "Checking for USB devices..."
    $usbDevice = Get-PnpDevice -PresentOnly | Where-Object { $_.DeviceID -like "USB\VID_1397*" }
    if ($usbDevice) {
        Log-Message "Found Behringer USB device. Assuming X32 is connected via USB."
        $Global:X32Connection = [PSCustomObject]@{
            Type = "USB"
            IPAddress = $null
        }
        return
    }

    # 2. Check for Network Connection (this can be slow)
    Log-Message "Checking for network devices... This may take a few minutes."
    $ipconfig = Get-NetIPConfiguration | Where-Object { $_.IPv4DefaultGateway -ne $null }
    if ($ipconfig) {
        $ipAddress = $ipconfig.IPv4Address.IPAddress
        $subnet = $ipAddress.Split('.')[0..2] -join '.'
        for ($i = 1; $i -lt 255; $i++) {
            $targetIP = "$subnet.$i"
            Write-Host "Scanning $targetIP..." -NoNewline
            $test = Test-NetConnection -ComputerName $targetIP -Port 10023 -WarningAction SilentlyContinue -InformationLevel Quiet
            if ($test.TcpTestSucceeded) {
                Write-Host " Found X32!"
                Log-Message "Found X32 at network address: $targetIP"
                $Global:X32Connection = [PSCustomObject]@{
                    Type = "Network"
                    IPAddress = $targetIP
                }
                return
            } else {
                 Write-Host ""
            }
        }
    }

    # 3. Prompt user if auto-detection fails
    Log-Message "Could not auto-detect X32 connection."
    if ($NonInteractive) {
        $emulatorPath = ".\target\release\x32_emulator.exe"
        if (-not (Test-Path $emulatorPath)) {
            $emulatorPath = ".\target\debug\x32_emulator.exe"
        }

        if (Test-Path $emulatorPath) {
            Log-Message "Non-interactive mode: Starting local background emulator ($emulatorPath)..."
            $Global:EmulatorProcess = Start-Process -FilePath $emulatorPath -ArgumentList "--ip", "127.0.0.1" -PassThru -NoNewWindow
            Start-Sleep -Seconds 1
            $Global:X32Connection = [PSCustomObject]@{
                Type = "Network"
                IPAddress = "127.0.0.1"
            }
            return
        } else {
            Log-Message "Warning: x32_emulator binary not found. Skipping user prompt and defaulting to no connection."
            $Global:X32Connection = [PSCustomObject]@{
                Type = "None"
                IPAddress = $null
            }
            return
        }
    }
    while ($true) {
        Clear-Host
        Write-Host "Could not automatically detect the X32."
        Write-Host "Please select the connection method:"
        Write-Host "1. Network"
        Write-Host "2. USB"
        Write-Host "s. Skip (run tests without a device)"
        $connChoice = Read-Host "Enter your choice"

        switch ($connChoice) {
            "1" {
                $ip = Read-Host "Please enter the X32 IP address"
                if ($ip -match "\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}") {
                    $Global:X32Connection = [PSCustomObject]@{
                        Type = "Network"
                        IPAddress = $ip
                    }
                    return
                } else {
                    Write-Host "Invalid IP address format."
                    Read-Host "Press Enter to continue..."
                }
            }
            "2" {
                $Global:X32Connection = [PSCustomObject]@{
                    Type = "USB"
                    IPAddress = $null
                }
                return
            }
            "s" {
                 $Global:X32Connection = [PSCustomObject]@{
                    Type = "None"
                    IPAddress = $null
                }
                return
            }
            default {
                Write-Host "Invalid selection."
                Read-Host "Press Enter to continue..."
            }
        }
    }
}


# --- Test Modules ---
$TestModules = @{}
Get-ChildItem -Path ".\tests" -Filter "*.test.ps1" | ForEach-Object {
    $moduleName = $_.BaseName.Split('.')[0]
    $TestModules[$moduleName] = $_.FullName
}

# --- Non-Interactive Mode ---
function Run-AllTestsNonInteractive {
    Log-Message "Running in non-interactive mode."
    $Global:NonInteractive = $true

    # Redefine Read-Host to auto-reply in non-interactive mode
    function global:Read-Host {
        param([string]$Prompt)
        Log-Message "[Non-Interactive Auto-Reply]"
        if ($Prompt -like "*y/n*" -or $Prompt -like "*succeed*") {
            return 'y'
        }
        return ''
    }

    if (-not $SkipBuild) {
        Log-Message "Compiling all binaries..."
        if (-not (Compile-Binaries)) {
            Log-Message "ERROR: Compilation failed. Aborting tests."
            exit 1
        }
    } else {
        Log-Message "Skipping compilation step as requested (-SkipBuild)."
    }

    Detect-X32Connection -NonInteractive $true

    try {
        Log-Message "Running all tests..."
        foreach ($module in $TestModules.GetEnumerator()) {
            . $module.Value
            $fileContent = Get-Content $module.Value -Raw
            if ($fileContent -match 'function\s+(Test-[\w_-]+)') {
                $testFunctionName = $Matches[1]
                Invoke-Expression -Command $testFunctionName
            } else {
                Log-Message "Warning: Could not find Test- function in $($module.Name)"
            }
        }
    } finally {
        # Restore Read-Host
        Remove-Item -Path function:Read-Host -ErrorAction SilentlyContinue
        if ($Global:EmulatorProcess) {
            Log-Message "Stopping background x32_emulator..."
            Stop-Process -Id $Global:EmulatorProcess.Id -Force -ErrorAction SilentlyContinue
        }
    }
    Log-Message "All tests finished."
    exit 0
}

# --- Main Menu (TUI) ---
function Show-MainMenu {
    Clear-Host
    Write-Host "X32 Rust Binaries - Test Suite"
    Write-Host "-------------------------------"
	Write-Host "Connection Status: $($Global:X32Connection.Type) $($Global:X32Connection.IPAddress)"
    Write-Host "1. Compile all binaries"
    Write-Host "2. Detect X32 connection"
    Write-Host "3. Run all tests"
    Write-Host "4. Run specific test..."
    Write-Host "q. Quit"
    $selection = Read-Host "Enter your choice"
    return $selection
}

# --- Main Loop ---
while ($true) {
    $choice = Show-MainMenu
    switch ($choice) {
        "1" {
            Compile-Binaries
            Read-Host "Press Enter to continue..."
        }
        "2" {
            Detect-X32Connection
            Read-Host "Press Enter to continue..."
        }
        "3" {
            Log-Message "Running all tests..."
            if ($null -eq $Global:X32Connection) { Detect-X32Connection }
            foreach ($module in $TestModules.GetEnumerator()) {
                . $module.Value
                $testFunctionName = "Test-$($module.Name)"
                Invoke-Expression -Command $testFunctionName
            }
            Read-Host "Press Enter to continue..."
        }
        "4" {
            Log-Message "Running specific test..."
            if ($null -eq $Global:X32Connection) { Detect-X32Connection }

            $i = 1
            $testOptions = @{}
            foreach ($module in $TestModules.GetEnumerator()) {
                Write-Host "$i. $($module.Name)"
                $testOptions[$i] = $module
                $i++
            }
            $testChoice = Read-Host "Select a test to run"
            $selectedTest = $testOptions[$testChoice]

            if ($selectedTest) {
                . $selectedTest.Value
                $testFunctionName = "Test-$($selectedTest.Name)"
                Invoke-Expression -Command $testFunctionName
            } else {
                Write-Host "Invalid selection."
            }
            Read-Host "Press Enter to continue..."
        }
        "q" {
            return
        }
        default {
            Write-Host "Invalid selection. Please try again."
            Read-Host "Press Enter to continue..."
        }
    }
}
