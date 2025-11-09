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
    "x32_wav_xlive"
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
    param (
        [bool]$Force = $false
    )

    if ($Force) {
        Log-Message "Starting force recompile of all binaries..."
    } else {
        Log-Message "Starting smart compile (checking for missing binaries)..."
    }

    foreach ($binary in $Binaries) {
        $binaryFilePath = Join-Path $Global:BinaryPath "$binary.exe"
        if (-not $Force -and (Test-Path $binaryFilePath)) {
            Log-Message "Skipping $binary (already exists)."
            continue
        }

        Log-Message "Compiling $binary..."
        cargo build --package $binary --release
        if ($LASTEXITCODE -ne 0) {
            Log-Message "ERROR: Compilation of $binary failed."
        }
    }
    Log-Message "Compilation process complete."
}

# --- X32 Connection ---
$Global:X32Connection = $null
$Global:BinaryPath = ".\target\release"

function Show-CompilationMenu {
    while ($true) {
        Clear-Host
        Write-Host "Manage Binaries"
        Write-Host "---------------"
        Write-Host "Current binary path: $Global:BinaryPath"
        Write-Host "a. Smart Compile (only missing binaries)"
        Write-Host "b. Force Recompile (all binaries)"
        Write-Host "c. Set Custom Binary Path"
        Write-Host "r. Return to Main Menu"
        $choice = Read-Host "Enter your choice"

        switch ($choice) {
            "a" {
                Compile-Binaries -Force $false
                Read-Host "Press Enter to continue..."
            }
            "b" {
                Compile-Binaries -Force $true
                Read-Host "Press Enter to continue..."
            }
            "c" {
                $newPath = Read-Host "Enter new binary path (e.g., .\target\release)"
                if (Test-Path $newPath) {
                    $Global:BinaryPath = $newPath
                    Log-Message "Binary path set to $newPath"
                } else {
                    Write-Host "Path not found."
                }
                Read-Host "Press Enter to continue..."
            }
            "r" {
                return
            }
            default {
                Write-Host "Invalid selection."
                Read-Host "Press Enter to continue..."
            }
        }
    }
}

function Check-BinariesExist {
    $found = Get-ChildItem -Path $Global:BinaryPath -Filter "*.exe" | Select-Object -First 1
    if ($null -eq $found) {
        Write-Host "No compiled binaries found in '$($Global:BinaryPath)'." -ForegroundColor Yellow
        Write-Host "Please use the 'Manage Binaries' menu to compile them before running tests." -ForegroundColor Yellow
        return $false
    }
    return $true
}

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

# --- Main Menu (TUI) ---
function Show-MainMenu {
    Clear-Host
    Write-Host "X32 Rust Binaries - Test Suite"
    Write-Host "-------------------------------"
	Write-Host "Connection Status: $($Global:X32Connection.Type) $($Global:X32Connection.IPAddress)"
    Write-Host "1. Manage Binaries"
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
            Show-CompilationMenu
        }
        "2" {
            Detect-X32Connection
            Read-Host "Press Enter to continue..."
        }
        "3" {
            Log-Message "Running all tests..."
            if (-not (Check-BinariesExist)) {
                Read-Host "Press Enter to continue..."
                continue
            }
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
            if (-not (Check-BinariesExist)) {
                Read-Host "Press Enter to continue..."
                continue
            }
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
