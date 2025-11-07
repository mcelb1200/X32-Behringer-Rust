use assert_cmd::prelude::*;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use std::net::{UdpSocket};
use tempfile::NamedTempFile;
use std::io::Write;
use std::sync::{Arc, Mutex};

#[test]
fn test_record_and_play() -> Result<(), Box<dyn std::error::Error>> {
    // Create a temporary file for the recording
    let temp_file = NamedTempFile::new()?;
    let file_path = temp_file.path().to_str().unwrap();
    println!("Using temp file: {}", file_path);

    let received_messages = Arc::new(Mutex::new(Vec::new()));
    let received_messages_clone = received_messages.clone();

    // Start the mock server
    let _mock_server = thread::spawn(move || {
        let socket = UdpSocket::bind("127.0.0.1:10023").unwrap();
        let mut buf = [0; 512];
        // Use a blocking socket for the mock server
        socket.set_read_timeout(None).unwrap();
        loop {
            let (amt, src) = socket.recv_from(&mut buf).unwrap();
            let received_buf = &buf[..amt];
            println!("Mock server received: {:?}", received_buf);
            if received_buf == b"/info\x00\x00\x00" {
                socket.send_to(b"/info\x00\x00\x00,s\x00\x00/some/info", src).unwrap();
            } else {
                 let mut messages = received_messages_clone.lock().unwrap();
                 messages.push(received_buf.to_vec());
            }
        }
    });

    thread::sleep(Duration::from_millis(500)); // Give the server a moment to start

    // Run the replay command to record
    println!("Recording...");
    let mut cmd = Command::cargo_bin("x32_replay")?;
    let mut child = cmd
        .arg("-i")
        .arg("127.0.0.1")
        .arg("-f")
        .arg(file_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(b"record on\n")?;
    stdin.flush()?;

    // Give the client time to start recording
    thread::sleep(Duration::from_millis(100));

    // Send a message to be recorded
    let client_socket = UdpSocket::bind("0.0.0.0:0")?;
    client_socket.connect("127.0.0.1:10024")?;
    client_socket.send(b"/ch/01/mix/fader\x00\x00,f\x00\x00\x00\x00\x00\x00")?;

    thread::sleep(Duration::from_millis(100));
    stdin.write_all(b"exit\n")?;
    stdin.flush()?;

    let output = child.wait_with_output()?;
    assert!(output.status.success());
    println!("Recording finished.");

    // Run the replay command to play back
    println!("Playing back...");
    let mut cmd_play = Command::cargo_bin("x32_replay")?;
    let mut child_play = cmd_play
        .arg("-i")
        .arg("127.0.0.1")
        .arg("-f")
        .arg(file_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let mut stdin_play = child_play.stdin.take().unwrap();
    stdin_play.write_all(b"play on\n")?;
    stdin_play.flush()?;
    thread::sleep(Duration::from_millis(200));
    stdin_play.write_all(b"exit\n")?;
    stdin_play.flush()?;

    let output_play = child_play.wait_with_output()?;
    assert!(output_play.status.success());
    println!("Playback finished.");

    let received = received_messages.lock().unwrap();
    println!("Received messages: {:?}", received);
    assert_eq!(received.len(), 1);
    assert_eq!(received[0], b"/ch/01/mix/fader\x00\x00,f\x00\x00\x00\x00\x00\x00");

    Ok(())
}
