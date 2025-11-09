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

# Initial connection detection
Detect-X32Connection

# --- Test Modules ---$TestModules = @{}
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
            foreach ($module in $TestModules.GetEnumerator()) {
                . $module.Value
                $testFunctionName = "Test-$($module.Name)"
                Invoke-Expression -Command $testFunctionName
            }
            Read-Host "Press Enter to continue..."
        }
        "4" {
            Log-Message "Running specific test..."

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
