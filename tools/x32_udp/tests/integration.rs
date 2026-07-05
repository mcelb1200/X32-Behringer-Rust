use assert_cmd::cargo::cargo_bin;
use std::net::UdpSocket;
use std::process::Command;

#[test]
fn test_x32_udp_communication() {
    let server = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind mock server");
    let server_addr = server.local_addr().expect("Failed to get local address");
    let port = server_addr.port().to_string();

    server
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .unwrap();

    let server_handle = std::thread::spawn(move || {
        let mut buf = [0u8; 512];
        loop {
            if let Ok((len, peer_addr)) = server.recv_from(&mut buf) {
                if len >= 12 && &buf[0..7] == b"/status" {
                    let msg = osc_lib::OscMessage::new(
                        "/status".to_string(),
                        vec![osc_lib::OscArg::String("active".to_string())],
                    );
                    let response = msg.to_bytes().unwrap();
                    server
                        .send_to(&response, peer_addr)
                        .expect("Mock server failed to send response");
                    break;
                }
            } else {
                break;
            }
        }
    });

    // Make sure server thread starts before we send command
    std::thread::sleep(std::time::Duration::from_millis(50));

    let output = Command::new(cargo_bin("x32_udp"))
        .arg("--ip")
        .arg("127.0.0.1")
        .arg("--port")
        .arg(&port)
        .arg("--timeout")
        .arg("1000")
        .arg("/status")
        .output()
        .expect("Failed to execute x32_udp");

    server_handle.join().expect("Mock server task panicked");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Debug print
    println!("STDOUT was:\n{}", stdout);

    assert!(stdout.contains("Connection status: 1"));
    assert!(stdout.contains("Send status: 12"));
    assert!(stdout.contains("Recv status: 20"));
    assert!(stdout.contains("/status~,s~~active~~"));

    assert!(output.status.success());
}

#[test]
fn test_x32_udp_timeout() {
    let server = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind mock server");
    let server_addr = server.local_addr().expect("Failed to get local address");
    let port = server_addr.port().to_string();

    let output = Command::new(cargo_bin("x32_udp"))
        .arg("--ip")
        .arg("127.0.0.1")
        .arg("--port")
        .arg(&port)
        .arg("--timeout")
        .arg("100")
        .arg("/status")
        .output()
        .expect("Failed to execute x32_udp");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Connection status: 1"));
    assert!(stdout.contains("Send status: 12"));
    assert!(stdout.contains("Recv status: 0"));
    assert!(stdout.contains("Receive timeout."));

    assert!(output.status.success());
}
