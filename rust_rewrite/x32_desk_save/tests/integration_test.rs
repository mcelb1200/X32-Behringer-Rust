use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::time::Duration;
use std::fs::File;
use std::io::Write;
use assert_cmd::Command;
use predicates::prelude::*;
use osc_lib::{OscMessage, OscArg};

fn setup_mock_x32_server() -> SocketAddr {
    let socket = UdpSocket::bind("127.0.0.1:0").expect("couldn't bind to address");
    socket.set_read_timeout(Some(Duration::from_millis(1000))).unwrap();
    let addr = socket.local_addr().unwrap();
    thread::spawn(move || {
        let mut buf = [0; 512];
        loop {
            if let Ok((_, src_addr)) = socket.recv_from(&mut buf) {
                let response_msg = OscMessage::new("/node".to_string(), vec![OscArg::String("mock_response".to_string())]);
                socket.send_to(&response_msg.to_bytes().unwrap(), &src_addr).expect("couldn't send data");
            }
        }
    });
    // Give the server a moment to start up
    thread::sleep(Duration::from_millis(100));
    addr
}

#[test]
fn test_desk_save_command() {
    let addr = setup_mock_x32_server();

    let mut cmd = Command::cargo_bin("x32_desk_save").unwrap();
    cmd.args(&["--ip", &addr.to_string(), "-d", "test_output.txt"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(format!("Successfully connected to X32 at {}", addr)))
        .stdout(predicate::str::contains("Successfully saved data to test_output.txt"));

    // Verify the content of the output file
    let content = std::fs::read_to_string("test_output.txt").unwrap();
    assert!(content.contains("/node,s \"mock_response\""));

    // Clean up the output file
    std::fs::remove_file("test_output.txt").unwrap();
}

#[test]
fn test_pattern_file_command() {
    let addr = setup_mock_x32_server();

    // Create a mock pattern file
    let mut file = File::create("test_pattern.txt").unwrap();
    writeln!(file, "# This is a comment").unwrap();
    writeln!(file, "/-stat/solosw").unwrap();
    writeln!(file, "/-prefs/remote").unwrap();

    let mut cmd = Command::cargo_bin("x32_desk_save").unwrap();
    cmd.args(&["--ip", &addr.to_string(), "-p", "test_pattern.txt", "test_output.txt"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(format!("Successfully connected to X32 at {}", addr)))
        .stdout(predicate::str::contains("Successfully saved data to test_output.txt"));

    // Verify the content of the output file
    let content = std::fs::read_to_string("test_output.txt").unwrap();
    assert!(content.contains("/node,s \"mock_response\""));
    assert_eq!(content.lines().count(), 2);

    // Clean up the files
    std::fs::remove_file("test_pattern.txt").unwrap();
    std::fs::remove_file("test_output.txt").unwrap();
}
