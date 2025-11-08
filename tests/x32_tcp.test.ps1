# Test module for x32_tcp

function Test-X32Tcp {
    Log-Message "--- Running x32_tcp tests ---"

    $binaryPath = ".\target\release\x32_tcp.exe"
    if (-not (Test-Path $binaryPath)) {
        Log-Message "ERROR: x32_tcp.exe not found. Please compile first."
        return
    }

    if ($Global:X32Connection.Type -eq "None") {
        Log-Message "Skipping x32_tcp tests as no connection is available."
        return
    }

    $ip = $Global:X32Connection.IPAddress
    $serverPort = 10041 # Default port for x32_tcp

    # --- Test 1: TCP Bridge Functional Test ---
    Log-Message "Test 1: TCP Bridge Functional Test"

    # 1. Start the server in the background
    Log-Message "Starting x32_tcp server in the background..."
    $process = Start-Process -FilePath $binaryPath -ArgumentList "--ip $ip --port $serverPort" -PassThru -NoNewWindow
    if ($null -eq $process) {
        Log-Message "Test 1 FAILED: Could not start x32_tcp process."
        return
    }
    Start-Sleep -Seconds 2 # Give the server time to start up

    try {
        # 2. Create and run the TCP client test
        Log-Message "Connecting TCP client to localhost:$serverPort..."
        $tcpClient = New-Object System.Net.Sockets.TcpClient("localhost", $serverPort)
        $stream = $tcpClient.GetStream()
        $writer = New-Object System.IO.StreamWriter($stream)
        $reader = New-Object System.IO.StreamReader($stream)

        # Send /info command
        $command = "/info"
        Log-Message "Client sending: $command"
        $writer.WriteLine($command)
        $writer.Flush()

        # Read the response
        $response = $reader.ReadLine()
        Log-Message "Client received: $response"

        # 3. Verification
        if ($response -like "/info*") {
            Log-Message "Test 1 PASSED: Received a valid /info response from the bridge."
        } else {
            Log-Message "Test 1 FAILED: The response was not in the expected format."
            Log-Message "Expected a string starting with '/info', but got: $response"
        }

    } catch {
        Log-Message "Test 1 FAILED with an exception: $_"
    } finally {
        # 4. Teardown
        if ($writer) { $writer.Close() }
        if ($reader) { $reader.Close() }
        if ($stream) { $stream.Close() }
        if ($tcpClient) { $tcpClient.Close() }

        Log-Message "Stopping x32_tcp server..."
        Stop-Process -Id $process.Id -Force
    }

    Log-Message "--- Finished x32_tcp tests ---"
}
