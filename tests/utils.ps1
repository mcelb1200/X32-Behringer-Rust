# Process management utilities for the X32 test suite

function Start-ManagedProcess {
    param (
        [string]$BinaryName,
        [string]$Arguments
    )

    $binaryPath = Join-Path $Global:BinaryPath "$BinaryName.exe"
    if (-not (Test-Path $binaryPath)) {
        Log-Message "ERROR: $BinaryName.exe not found."
        return $null
    }

    $processInfo = New-Object System.Diagnostics.ProcessStartInfo
    $processInfo.FileName = $binaryPath
    $processInfo.Arguments = $Arguments
    $processInfo.RedirectStandardOutput = $true
    $processInfo.RedirectStandardError = $true
    $processInfo.UseShellExecute = $false

    $process = New-Object System.Diagnostics.Process
    $process.StartInfo = $processInfo
    $process.EnableRaisingEvents = $true

    $output = @()
    $errorOutput = @()

    $process.add_OutputDataReceived({
        if (-not [string]::IsNullOrEmpty($_.Data)) {
            $output += $_.Data
            Log-Message "[$BinaryName] STDOUT: $($_.Data)"
        }
    })

    $process.add_ErrorDataReceived({
        if (-not [string]::IsNullOrEmpty($_.Data)) {
            $errorOutput += $_.Data
            Log-Message "[$BinaryName] STDERR: $($_.Data)"
        }
    })

    Log-Message "Starting $BinaryName..."
    $process.Start() | Out-Null
    $process.BeginOutputReadLine()
    $process.BeginErrorReadLine()

    return @{
        Process = $process
        Output = $output
        ErrorOutput = $errorOutput
    }
}

function Stop-ManagedProcess {
    param (
        $ManagedProcess
    )

    if ($null -eq $ManagedProcess) {
        return
    }

    $process = $ManagedProcess.Process
    if (-not $process.HasExited) {
        Log-Message "Stopping $($process.ProcessName)..."
        $process.Kill()
        $process.WaitForExit()
    }
    Log-Message "$($process.ProcessName) stopped."
}
