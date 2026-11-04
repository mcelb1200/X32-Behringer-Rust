use assert_cmd::Command;
use osc_lib::{OscArg, OscMessage};
use predicates::prelude::*;
use std::net::UdpSocket;
use std::thread;
use std::time::Duration;

#[test]
fn test_sync_cli_help() {
    let mut cmd = Command::cargo_bin("x32_sync").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("x32_sync"))
        .stdout(predicate::str::contains("-M"))
        .stdout(predicate::str::contains("--model-b"));
}

#[test]
fn test_sync_cli_missing_args() {
    let mut cmd = Command::cargo_bin("x32_sync").unwrap();
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("error:"));
}

#[test]
fn test_sync_between_x32_and_xr18_filters_out_of_bounds() {
    // Console A is X32, Console B is XR18.
    // Console A sends /ch/01/mix/fader (valid for XR18) and /ch/32/mix/fader (out-of-bounds for XR18: max 16).
    // XR18 should only receive /ch/01/mix/fader.

    let socket_a = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind mock console A");
    let addr_a = socket_a.local_addr().unwrap();

    let socket_b = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind mock console B");
    let addr_b = socket_b.local_addr().unwrap();

    let server_a_thread = thread::spawn(move || {
        let mut buf = [0; 512];
        let mut client_addr = None;
        socket_a
            .set_read_timeout(Some(Duration::from_millis(5000)))
            .unwrap();

        // Wait for x32_sync to connect and send heartbeat
        while client_addr.is_none() {
            if let Ok((len, from)) = socket_a.recv_from(&mut buf) {
                if let Ok(msg) = OscMessage::from_bytes(&buf[..len]) {
                    if msg.path == "/xremote" {
                        client_addr = Some(from);
                    }
                }
            }
        }

        let target = client_addr.unwrap();
        // Give x32_sync a brief moment to finish setting up Console B connection
        thread::sleep(Duration::from_millis(150));

        // Console A produces an update on valid path
        let msg_valid = OscMessage {
            path: "/ch/01/mix/fader".to_string(),
            args: vec![OscArg::Float(0.75)],
        };
        let bytes_valid = msg_valid.to_bytes().unwrap();
        socket_a.send_to(&bytes_valid, target).unwrap();

        thread::sleep(Duration::from_millis(50));

        // Console A produces an update on out-of-bounds path for XR18 (ch 32)
        let msg_oob = OscMessage {
            path: "/ch/32/mix/fader".to_string(),
            args: vec![OscArg::Float(0.85)],
        };
        let bytes_oob = msg_oob.to_bytes().unwrap();
        socket_a.send_to(&bytes_oob, target).unwrap();
    });

    let server_b_thread = thread::spawn(move || {
        let mut buf = [0; 512];
        socket_b
            .set_read_timeout(Some(Duration::from_millis(5000)))
            .unwrap();

        let mut received = Vec::new();
        let start = std::time::Instant::now();
        while start.elapsed() < Duration::from_millis(2500) {
            if let Ok((len, _)) = socket_b.recv_from(&mut buf) {
                if let Ok(msg) = OscMessage::from_bytes(&buf[..len]) {
                    if msg.path != "/xremote" {
                        received.push(msg.path);
                    }
                }
            }
        }

        assert_eq!(
            received,
            vec!["/ch/01/mix/fader"],
            "Console B (XR18) should only receive valid paths; received: {:?}",
            received
        );
    });

    let mut cmd = Command::cargo_bin("x32_sync").unwrap();
    cmd.arg("-a")
        .arg(addr_a.to_string())
        .arg("-b")
        .arg(addr_b.to_string())
        .arg("-M")
        .arg("X32")
        .arg("--model-b")
        .arg("XR18")
        .timeout(Duration::from_millis(3000));

    // The command runs indefinitely until timeout kills it or error
    let _ = cmd.output();

    server_a_thread.join().unwrap();
    server_b_thread.join().unwrap();
}
