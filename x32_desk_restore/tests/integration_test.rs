use osc_lib::OscMessage;
use osc_lib::OscMessage;
use std::fs::File;
use std::fs::File;
use std::io::Write;
use std::io::Write;
use std::net::UdpSocket;
use std::thread;
use std::time::Duration;

fn setup_mock_x32_server() -> String {
    let socket = UdpSocket::bind("127.0.0.1:0").expect("couldn't bind to address");
    let server_addr = socket.local_addr().unwrap().to_string();
    let server_socket = socket.try_clone().unwrap();
    thread::spawn(move || {
        let mut buf = [0; 512];
        // Set a short read timeout so the thread doesn't block forever
        server_socket
            .set_read_timeout(Some(Duration::from_millis(500)))
            .unwrap();
        loop {
            match server_socket.recv_from(&mut buf) {
                Ok((number_of_bytes, src_addr)) => {
                    if let Ok(received_msg) = OscMessage::from_bytes(&buf[..number_of_bytes]) {
                        // Echo the message back to the client
                        server_socket
                            .send_to(&received_msg.to_bytes().unwrap(), src_addr)
                            .expect("couldn't send data");
                    }
                }
                Err(e) => {
                    if e.kind() != std::io::ErrorKind::WouldBlock
                        && e.kind() != std::io::ErrorKind::TimedOut
                    {
                        // An actual error occurred
                        break;
                    }
                }
            }
        }
    });
    server_addr
}

#[test]
fn test_desk_restore_command() {
    let server_addr = setup_mock_x32_server();

    // Create a mock data file
    let mut file = File::create("test_restore.txt").unwrap();
    writeln!(file, "# This is a comment").unwrap();
    writeln!(file, "/-stat/solosw ,i 1").unwrap();
    writeln!(file, "/-prefs/remote ,s \"HUI\"").unwrap();

    let bin = escargot::CargoBuild::new()
        .bin("x32_desk_restore")
        .run()
        .unwrap();
    let mut cmd = bin.command();
    cmd.args(&["--ip", &server_addr, "test_restore.txt"]);

    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(&format!("Successfully connected to X32 at {}", server_addr)));
    assert!(stdout.contains("Successfully restored data from test_restore.txt"));

    // Clean up the files
    std::fs::remove_file("test_restore.txt").unwrap();
}
