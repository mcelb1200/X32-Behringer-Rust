# Test module for x32_wav_xlive
#
# Credits
#
# *   Original concept and work on the C library: Patrick-Gilles Maillot
# *   Additional concepts by: [User]
# *   Rust implementation by: [User]

# Helper function to generate a minimal, valid, 24-bit mono WAV file.
function New-MonoWavFile {
    param(
        [string]$Path,
        [int]$SampleRate = 48000,
        [int]$DurationSeconds = 1
    )
    $numSamples = $SampleRate * $DurationSeconds
    $bitsPerSample = 24
    $numChannels = 1
    $blockAlign = $numChannels * ($bitsPerSample / 8)
    $byteRate = $SampleRate * $blockAlign
    $subchunk2Size = $numSamples * $blockAlign

    $fileStream = [System.IO.File]::Create($Path)
    $writer = New-Object System.IO.BinaryWriter($fileStream)

    # RIFF Header
    $writer.Write([char[]]"RIFF")
    $writer.Write([int](36 + $subchunk2Size))
    $writer.Write([char[]]"WAVE")

    # FMT Subchunk
    $writer.Write([char[]]"fmt ")
    $writer.Write([int]16) # Subchunk1Size
    $writer.Write([short]1) # AudioFormat (PCM)
    $writer.Write([short]$numChannels)
    $writer.Write([int]$SampleRate)
    $writer.Write([int]$byteRate)
    $writer.Write([short]$blockAlign)
    $writer.Write([short]$bitsPerSample)

    # DATA Subchunk
    $writer.Write([char[]]"data")
    $writer.Write([int]$subchunk2Size)

    # Write silent audio data (24-bit samples are 3 bytes)
    $sample = New-Object byte[] 3
    for ($i = 0; $i -lt $numSamples; $i++) {
        $writer.Write($sample)
    }

    $writer.Close()
    $fileStream.Close()
}

function Test-X32WavXlive {
    Log-Message "--- Running x32_wav_xlive tests ---"

    $binaryPath = ".\target\release\x32_wav_xlive.exe"
    if (-not (Test-Path $binaryPath)) {
        Log-Message "ERROR: x32_wav_xlive.exe not found. Please compile first."
        return
    }

    # This test is non-interactive and does not require a connection to the X32.

    $testDir = "temp_wav_source"

    # --- Setup ---
    if (Test-Path $testDir) { Remove-Item $testDir -Recurse -Force }
    New-Item -ItemType Directory -Path $testDir | Out-Null
    Log-Message "Created temporary directory: $testDir"

    try {
        # Generate two mono WAV files
        New-MonoWavFile -Path (Join-Path $testDir "ch_1.wav")
        New-MonoWavFile -Path (Join-Path $testDir "ch_2.wav")
        Log-Message "Generated synthetic mono WAV files."

        # --- Test 1: Session Creation and WAV file verification ---
        Log-Message "Test 1: Session Creation and WAV file verification"

        Log-Message "Running: x32_wav_xlive $testDir"
        & $binaryPath $testDir --silent

        # Verification
        $sessionDir = Get-ChildItem -Path $testDir -Directory | Select-Object -First 1
        if ($null -eq $sessionDir) {
            Log-Message "Test 1 FAILED: No session directory was created."
            return
        }
        Log-Message "Session directory found: $($sessionDir.Name)"

        if (-not (Test-Path (Join-Path $sessionDir.FullName "SE_LOG.BIN"))) {
            Log-Message "Test 1 FAILED: SE_LOG.BIN was not found in the session directory."
            return
        }
        Log-Message "SE_LOG.BIN found."

        $wavFile = Get-ChildItem -Path $sessionDir.FullName -Filter "*.wav" | Select-Object -First 1
        if ($null -eq $wavFile) {
            Log-Message "Test 1 FAILED: No .wav file was found in the session directory."
            return
        }
        Log-Message "Multichannel WAV file found: $($wavFile.Name)"

        # Read the header of the output WAV to verify its properties
        $fileStream = [System.IO.File]::OpenRead($wavFile.FullName)
        $reader = New-Object System.IO.BinaryReader($fileStream)
        $reader.BaseStream.Position = 22 # Position for Number of Channels
        $numChannels = $reader.ReadInt16()
        $reader.BaseStream.Position = 34 # Position for Bits per Sample
        $bitsPerSample = $reader.ReadInt16()
        $reader.Close()
        $fileStream.Close()

        if (($numChannels -eq 2) -and ($bitsPerSample -eq 32)) {
            Log-Message "Test 1 PASSED: Output WAV file has the correct properties (2 channels, 32-bit)."
        } else {
            Log-Message "Test 1 FAILED: Output WAV file has incorrect properties."
            Log-Message "Expected: 2 Channels, 32 Bits per Sample"
            Log-Message "Actual:   $numChannels Channels, $bitsPerSample Bits per Sample"
        }

    } finally {
        # --- Teardown ---
        if (Test-Path $testDir) {
            Remove-Item $testDir -Recurse -Force
            Log-Message "Cleaned up temporary directory."
        }
    }

    Log-Message "--- Finished x32_wav_xlive tests ---"
}
