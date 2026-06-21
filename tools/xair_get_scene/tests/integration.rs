use assert_cmd::Command;
use osc_lib::{OscArg, OscMessage};
use predicates::prelude::*;
use std::net::UdpSocket;
use std::thread;
use std::time::Duration;

#[test]
fn test_xair_get_scene_with_cli_args() {
    let mock_console = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind mock console");
    let console_addr = mock_console
        .local_addr()
        .expect("Failed to get local address");

    mock_console
        .set_read_timeout(Some(Duration::from_millis(500)))
        .unwrap();

    thread::spawn(move || {
        let mut buf = [0; 512];

        for _ in 0..10 {
            if let Ok((len, addr)) = mock_console.recv_from(&mut buf) {
                if let Ok(msg) = OscMessage::from_bytes(&buf[..len]) {
                    if msg.path == "/node" && msg.args.len() == 1 {
                        // The test gets stuck in the code because xair_get_scene issues "/node"
                        // but then actually awaits `client.query_value` which issues a plain `line` GET request implicitly?
                        // Let's check `client.query_value(line)`. It sends an empty OSC message with `line` as path.
                    } else if msg.path == "/ch/01/mix/fader" && msg.args.is_empty() {
                        let response = OscMessage::new(
                            "/ch/01/mix/fader".to_string(),
                            vec![OscArg::String("0.75".to_string())], // xair_get_scene converts whatever back to string mostly or outputs format
                        );
                        let _ = mock_console.send_to(&response.to_bytes().unwrap(), addr);
                        break;
                    }
                }
            }
        }
    });

    let mut cmd = Command::cargo_bin("xair_get_scene").unwrap();
    cmd.arg("-i")
        .arg(console_addr.to_string())
        .arg("-s")
        .arg("TestScene")
        .arg("-n")
        .arg("TestNote")
        .write_stdin("/ch/01/mix/fader\n")
        .timeout(Duration::from_secs(5));

    cmd.assert().success().stdout(predicate::str::contains(
        "#2.1# \"TestScene\" \"TestNote\" %000000000 1 XAirGetScene V1.4 (c)2014 Patrick-Gilles Maillot"
    )).stdout(predicate::str::contains(
        "/ch/01/mix/fader 0.75"
    ));
}
