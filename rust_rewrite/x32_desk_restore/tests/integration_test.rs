use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::time::Duration;
use std::fs::File;
use std::io::Write;
use assert_cmd::Command;
use predicates::prelude::*;
use osc_lib::OscMessage;

fn setup_mock_x32_server() -> (UdpSocket, SocketAddr) {
    let socket = UdpSocket::bind("127.0.0.1:0").expect("couldn't bind to address");
    let server_addr = socket.local_addr().unwrap();
    socket.set_read_timeout(Some(Duration::from_millis(100))).unwrap();
    let server_socket = socket.try_clone().unwrap();
    thread::spawn(move || {
        let mut buf = [0; 512];
        loop {
            match server_socket.recv_from(&mut buf) {
                Ok((number_of_bytes, src_addr)) => {
                    let received_msg = OscMessage::from_bytes(&buf[..number_of_bytes]).unwrap();
                    server_socket.send_to(received_msg.to_bytes().unwrap().as_slice(), src_addr).expect("couldn't send data");
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
    (socket, server_addr)
}

#[test]
fn test_desk_restore_command() {
    let (_socket, server_addr) = setup_mock_x32_server();

    // Create a mock data file
    let mut file = File::create("test_restore.txt").unwrap();
    writeln!(file, "# This is a comment").unwrap();
    writeln!(file, "/-stat/solosw ,i 1").unwrap();
    writeln!(file, "/-prefs/remote ,s \"HUI\"").unwrap();

    let mut cmd = Command::cargo_bin("x32_desk_restore").unwrap();
    cmd.args(&["--ip", &server_addr.to_string(), "test_restore.txt"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(&format!("Successfully connected to X32 at {}", server_addr)))
        .stdout(predicate::str::contains("Successfully restored data from test_restore.txt"));

    // Clean up the files
    std::fs::remove_file("test_restore.txt").unwrap();
}
