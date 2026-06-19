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

    // Spawn a thread to act as the mock XAir console
    thread::spawn(move || {
        let mut buf = [0; 512];

        // Receive the first `/node` command
        if let Ok((len, addr)) = mock_console.recv_from(&mut buf) {
            let msg = OscMessage::from_bytes(&buf[..len]).unwrap();
            assert_eq!(msg.path, "/node");
            assert_eq!(msg.args.len(), 1);

            if let Some(OscArg::String(s)) = msg.args.first() {
                assert_eq!(s, "/ch/01/mix/fader");

                // Send back a mock response
                let response = OscMessage::new(
                    "/ch/01/mix/fader".to_string(),
                    vec![OscArg::String("0.75".to_string())],
                );
                mock_console
                    .send_to(&response.to_bytes().unwrap(), addr)
                    .unwrap();
            }
        }
    });

    // Run the CLI tool
    let mut cmd = Command::cargo_bin("xair_get_scene").unwrap();
    cmd.arg("-i")
        .arg(console_addr.to_string())
        .arg("-s")
        .arg("TestScene")
        .arg("-n")
        .arg("TestNote")
        .write_stdin("/ch/01/mix/fader\n")
        .timeout(Duration::from_secs(5));

    // Assert the output
    cmd.assert().success().stdout(predicate::str::contains(
        "#2.1# \"TestScene\" \"TestNote\" %000000000 1 XAirGetScene V1.4 (c)2014 Patrick-Gilles Maillot\n\n/ch/01/mix/fader 0.75\n",
    ));
}
