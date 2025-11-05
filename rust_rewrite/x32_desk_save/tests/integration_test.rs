use std::net::UdpSocket;
use std::thread;
use std::time::Duration;
use std::fs::File;
use std::io::Write;
use assert_cmd::Command;
use predicates::prelude::*;
use osc_lib::{OscMessage, OscArg};

fn setup_mock_x32_server() {
    thread::spawn(|| {
        let socket = UdpSocket::bind("127.0.0.1:10023").expect("couldn't bind to address");
        let mut buf = [0; 512];

        loop {
            match socket.recv_from(&mut buf) {
                Ok((number_of_bytes, src_addr)) => {
                    let received_msg = OscMessage::from_bytes(&buf[..number_of_bytes]).unwrap();
                    let response_msg = OscMessage::new(received_msg.path, vec![OscArg::String("mock_response".to_string())]);
                    socket.send_to(&response_msg.to_bytes().unwrap(), src_addr).expect("couldn't send data");
                }
                Err(_) => {
                    // Timeout, break the loop
                    break;
                }
            }
        }
    });
    // Give the server a moment to start up
    thread::sleep(Duration::from_millis(100));
}

#[test]
fn test_desk_save_command() {
    setup_mock_x32_server();

    let mut cmd = Command::cargo_bin("x32_desk_save").unwrap();
    cmd.args(&["--ip", "127.0.0.1", "-d", "test_output.txt"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Successfully connected to X32 at 127.0.0.1"))
        .stdout(predicate::str::contains("Successfully saved data to test_output.txt"));

    // Verify the content of the output file
    let content = std::fs::read_to_string("test_output.txt").unwrap();
    assert!(content.contains("/node ,s \"mock_response\""));

    // Clean up the output file
    std::fs::remove_file("test_output.txt").unwrap();
}

#[test]
fn test_pattern_file_command() {
    setup_mock_x32_server();

    // Create a mock pattern file
    let mut file = File::create("test_pattern.txt").unwrap();
    writeln!(file, "# This is a comment").unwrap();
    writeln!(file, "/-stat/solosw").unwrap();
    writeln!(file, "/-prefs/remote").unwrap();

    let mut cmd = Command::cargo_bin("x32_desk_save").unwrap();
    cmd.args(&["--ip", "127.0.0.1", "-p", "test_pattern.txt", "test_output.txt"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Successfully connected to X32 at 127.0.0.1"))
        .stdout(predicate::str::contains("Successfully saved data to test_output.txt"));

    // Verify the content of the output file
    let content = std::fs::read_to_string("test_output.txt").unwrap();
    assert!(content.contains("/node ,s \"mock_response\""));
    assert_eq!(content.lines().count(), 2);

    // Clean up the files
    std::fs::remove_file("test_pattern.txt").unwrap();
    std::fs::remove_file("test_output.txt").unwrap();
}
