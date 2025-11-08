use assert_cmd::prelude::*;
use osc_lib::{OscArg, OscMessage};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpStream, UdpSocket};
use std::process::Command;
use std::thread;
use std::time::Duration;

#[test]
fn test_server_e2e() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Start the x32_tcp server in a separate process
    let mut cmd = Command::cargo_bin("x32_tcp")?;
    let mut server_process = cmd
        .arg("-p")
        .arg("10043")
        .arg("-i")
        .arg("127.0.0.1:10025")
        .spawn()?;
    thread::sleep(Duration::from_secs(1)); // Wait for the server to start

    // 2. Run a mock UDP server to act as the X32 mixer
    let mock_x32_socket = UdpSocket::bind("127.0.0.1:10025")?;
    let mock_x32_socket_clone = mock_x32_socket.try_clone()?;
    let mock_server_handle = thread::spawn(move || {
        let mut buf = [0; 1024];
        let (len, src) = mock_x32_socket_clone.recv_from(&mut buf).unwrap();
        let received_msg = OscMessage::from_bytes(&buf[..len]).unwrap();

        // Respond to the client
        let response_msg = OscMessage::new(
            "/info".to_string(),
            vec![OscArg::String("X32 ROCKS".to_string())],
        );
        let response_bytes = response_msg.to_bytes().unwrap();
        mock_x32_socket_clone.send_to(&response_bytes, src).unwrap();

        thread::sleep(Duration::from_millis(100)); // Give the server time to process the response
        received_msg
    });

    // 3. Connect a TCP client to the x32_tcp server
    let mut client_stream = TcpStream::connect("127.0.0.1:10043")?;
    let mut reader = BufReader::new(client_stream.try_clone()?);

    // 4. Send a command from the client
    client_stream.write_all(b"/info\n")?;

    // 5. Assert that the client receives the correctly formatted string
    let mut response = String::new();
    reader.read_line(&mut response)?;
    assert_eq!(response.trim(), r#"/info,s "X32 ROCKS""#);

    // d. Assert that the correct OSC message is received by the mock X32
    let received_msg = mock_server_handle.join().unwrap();
    assert_eq!(received_msg.path, "/info");
    assert!(received_msg.args.is_empty());

    // Clean up
    server_process.kill()?;
    Ok(())
}
