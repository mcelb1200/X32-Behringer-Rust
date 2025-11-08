use osc_lib::{OscArg, OscMessage};
use std::fs::File;
use std::io::Write;
use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::time::Duration;

fn setup_mock_x32_server() -> SocketAddr {
    let socket = UdpSocket::bind("127.0.0.1:0").expect("couldn't bind to address");
    let addr = socket.local_addr().unwrap();
    thread::spawn(move || {
        let mut buf = [0; 512];
        loop {
            if let Ok((number_of_bytes, src_addr)) = socket.recv_from(&mut buf) {
                if let Ok(received_msg) = OscMessage::from_bytes(&buf[..number_of_bytes]) {
                    if received_msg.path == "/node" {
                        if let Some(OscArg::String(node_path)) = received_msg.args.get(0) {
                            let response_msg = OscMessage::new(
                                "/node".to_string(),
                                vec![
                                    OscArg::String(node_path.clone()),
                                    OscArg::String("mock_value".to_string()),
                                ],
                            );
                            socket
                                .send_to(&response_msg.to_bytes().unwrap(), src_addr)
                                .expect("couldn't send data");
                        }
                    }
                }
            } else {
                // An error occurred, break the loop
                break;
            }
        }
    });
    // Give the server a moment to start up
    thread::sleep(Duration::from_millis(500));
    addr
}

#[test]
fn test_desk_save_command() {
    let addr = setup_mock_x32_server();

    let bin = escargot::CargoBuild::new()
        .bin("x32_desk_save")
        .run()
        .unwrap();
    let mut cmd = bin.command();
    cmd.args(&["--ip", &addr.to_string(), "-d", "test_output.txt"]);

    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(&format!("Successfully connected to X32 at {}", addr)));
    assert!(stdout.contains("Successfully saved data to test_output.txt"));


    // Verify the content of the output file
    let content = std::fs::read_to_string("test_output.txt").unwrap();
    let lines: Vec<&str> = content.lines().collect();
    assert!(
        lines
            .iter()
            .any(|&line| line == "/node,ss \"-stat/solosw\" \"mock_value\"")
    );

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

    let bin = escargot::CargoBuild::new()
        .bin("x32_desk_save")
        .run()
        .unwrap();
    let mut cmd = bin.command();
    cmd.args(&[
        "--ip",
        &addr.to_string(),
        "-p",
        "test_pattern.txt",
        "test_output.txt",
    ]);

    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(&format!("Successfully connected to X32 at {}", addr)));
    assert!(stdout.contains("Successfully saved data to test_output.txt"));

    // Verify the content of the output file
    let content = std::fs::read_to_string("test_output.txt").unwrap();
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(
        lines
            .iter()
            .any(|&line| line == "/node,ss \"/-stat/solosw\" \"mock_value\"")
    );
    assert!(
        lines
            .iter()
            .any(|&line| line == "/node,ss \"/-prefs/remote\" \"mock_value\"")
    );

    // Clean up the files
    std::fs::remove_file("test_pattern.txt").unwrap();
    std::fs::remove_file("test_output.txt").unwrap();
}
