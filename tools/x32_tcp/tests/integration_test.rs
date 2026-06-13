use osc_lib::{OscArg, OscMessage};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpStream, UdpSocket};
use std::thread;
use std::time::Duration;

#[test]
fn test_server_e2e() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Start the mock UDP server FIRST to act as the X32 mixer
    let mock_x32_socket = UdpSocket::bind("127.0.0.1:10025")?;
    mock_x32_socket.set_read_timeout(Some(Duration::from_secs(5)))?;
    let mock_x32_socket_clone = mock_x32_socket.try_clone()?;

    let mock_server_handle = thread::spawn(move || {
        let mut buf = [0; 1024];
        match mock_x32_socket_clone.recv_from(&mut buf) {
            Ok((len, src)) => {
                let received_msg = OscMessage::from_bytes(&buf[..len]).unwrap();

                // Respond to the client
                let response_msg = OscMessage::new(
                    "/info".to_string(),
                    vec![OscArg::String("X32 ROCKS".to_string())],
                );
                let response_bytes = response_msg.to_bytes().unwrap();
                let _ = mock_x32_socket_clone.send_to(&response_bytes, src);

                received_msg
            }
            Err(_) => panic!("Mock server failed to receive message"),
        }
    });

    // 2. Start the x32_tcp server
    let bin = escargot::CargoBuild::new().bin("x32_tcp").run()?;
    let mut cmd = bin.command();
    let mut server_process = cmd
        .arg("-p")
        .arg("10043")
        .arg("-i")
        .arg("127.0.0.1:10025")
        .spawn()?;
    thread::sleep(Duration::from_millis(500)); // Wait for server to start and bind

    // 3. Connect TCP client and send command
    let mut client_stream = TcpStream::connect("127.0.0.1:10043")?;
    client_stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    let mut reader = BufReader::new(client_stream.try_clone()?);

    // 4. Send command
    client_stream.write_all(b"/info\n")?;
    client_stream.flush()?;

    // 5. Read response
    let mut response = String::new();
    reader.read_line(&mut response)?;
    assert_eq!(response.trim(), "/info \"X32 ROCKS\"");

    // 6. Verify mock server received the message
    let received_msg = mock_server_handle.join().unwrap();
    assert_eq!(received_msg.path, "/info");
    assert!(received_msg.args.is_empty());

    // Clean up
    let _ = server_process.kill();
    Ok(())
}
