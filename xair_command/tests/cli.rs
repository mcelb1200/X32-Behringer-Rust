use assert_cmd::Command;
use assert_cmd::cargo::CommandCargoExt;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("xair_command").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("usage: xair_command"));
}

#[test]
fn test_cli_default_args() {
    let mut cmd = Command::cargo_bin("xair_command").unwrap();
    cmd.arg("--unknown-flag").assert().failure();
}

#[tokio::test]
async fn test_udp_connection_and_batch() {
    use std::io::Write;
    use tempfile::NamedTempFile;
    use tokio::net::UdpSocket;

    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let addr = socket.local_addr().unwrap();

    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "# This is a comment").unwrap();
    writeln!(file, "/ch/01/mix/fader ,f 0.5").unwrap();
    writeln!(file, "kill").unwrap();

    let file_path = file.path().to_str().unwrap().to_string();
    let port_str = addr.port().to_string();

    let mut cmd = std::process::Command::cargo_bin("xair_command").unwrap();
    cmd.arg("--ip")
        .arg("127.0.0.1")
        .arg("--file")
        .arg(&file_path)
        .env("XAIR_PORT", &port_str);

    let mut child = cmd.spawn().unwrap();

    let mut buf = [0u8; 1024];

    // The logic in main currently sends b"/xinfo" every 500ms until it receives a valid response
    let timeout = std::time::Duration::from_secs(5);
    tokio::time::timeout(timeout, async {
        loop {
            let (len, src) = socket.recv_from(&mut buf).await.unwrap();
            let msg_str = String::from_utf8_lossy(&buf[..len]);
            let msg_str = msg_str.trim_end_matches('\0'); // handle padding

            if msg_str.starts_with("/xinfo") {
                // Send back response
                socket.send_to(b"/xinfo", src).await.unwrap();
                break;
            } else {
                println!("Unexpected message: {}", msg_str);
            }
        }
    }).await.unwrap();

    // After connecting, it should send the batch commands
    tokio::time::timeout(timeout, async {
        loop {
            let (len, _src) = socket.recv_from(&mut buf).await.unwrap();
            if let Ok(msg) = osc_lib::OscMessage::from_bytes(&buf[..len]) {
                if msg.path.starts_with("/ch/01/mix/fader") {
                    break;
                }
            } else {
                let msg_str = String::from_utf8_lossy(&buf[..len]);
                let msg_str = msg_str.trim_end_matches('\0');
                if msg_str.starts_with("/xinfo") {
                    // ignore retries that arrived late
                } else if msg_str.starts_with("/ch/01/mix/fader") {
                    break;
                } else {
                    panic!("Unexpected message: {}", msg_str);
                }
            }
        }
    }).await.unwrap();

    let status = tokio::time::timeout(timeout, async {
        child.wait().unwrap()
    }).await.unwrap();
    assert!(status.success());
}
