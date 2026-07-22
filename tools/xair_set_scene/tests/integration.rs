use assert_cmd::Command;
use osc_lib::{OscArg, OscMessage};
use std::net::UdpSocket;
use std::thread;
use std::time::Duration;

#[test]
fn test_xair_set_scene_with_cli_args() {
    let mock_console = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind mock console");
    let console_addr = mock_console
        .local_addr()
        .expect("Failed to get local address");

    let mock_thread = thread::spawn(move || {
        let mut buf = [0; 512];
        mock_console
            .set_read_timeout(Some(Duration::from_millis(5000)))
            .unwrap();

        if let Ok((len, _)) = mock_console.recv_from(&mut buf) {
            let msg = OscMessage::from_bytes(&buf[..len]).unwrap();
            assert_eq!(msg.path, "/ch/01/mix/fader");
            assert_eq!(msg.args.len(), 1);
        }
    });

    let mut cmd = Command::cargo_bin("xair_set_scene").unwrap();
    cmd.arg("-i")
        .arg(console_addr.to_string())
        .write_stdin("/ch/01/mix/fader 0.75\n")
        .timeout(Duration::from_secs(10));

    cmd.assert().success();
    mock_thread.join().unwrap();
}

#[test]
fn test_xair_set_scene_filters_x32_only_paths() {
    let mock_console = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind mock console");
    let console_addr = mock_console
        .local_addr()
        .expect("Failed to get local address");

    let input = "\
/ch/01/mix/fader 0.5
/ch/32/mix/fader 0.75
/bus/01/mix/fader 0.9
";

    let mock_thread = thread::spawn(move || {
        let mut buf = [0; 512];
        mock_console
            .set_read_timeout(Some(Duration::from_millis(5000)))
            .unwrap();

        let mut received = Vec::new();
        while let Ok((len, _)) = mock_console.recv_from(&mut buf) {
            received.push(OscMessage::from_bytes(&buf[..len]).unwrap().path);
            mock_console
                .set_read_timeout(Some(Duration::from_millis(100)))
                .unwrap();
        }

        assert_eq!(
            received.len(),
            2,
            "Expected exactly 2 messages, but got {:?}",
            received
        );
        assert_eq!(received[0], "/ch/01/mix/fader");
        assert_eq!(received[1], "/bus/01/mix/fader");
    });

    let mut cmd = Command::cargo_bin("xair_set_scene").unwrap();
    cmd.arg("-i")
        .arg(console_addr.to_string())
        .arg("--delay")
        .arg("0")
        .write_stdin(input)
        .timeout(Duration::from_secs(10));

    cmd.assert().success();
    mock_thread.join().unwrap();
}

#[test]
fn test_xair_set_scene_fallback_raw_osc() {
    let mock_console = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind mock console");
    let console_addr = mock_console
        .local_addr()
        .expect("Failed to get local address");

    let input = "/some/raw/osc/path ,f 0.42\n";

    let mock_thread = thread::spawn(move || {
        let mut buf = [0; 512];
        mock_console
            .set_read_timeout(Some(Duration::from_millis(5000)))
            .unwrap();
        let (len, _) = mock_console
            .recv_from(&mut buf)
            .expect("Failed to receive raw osc msg");
        let msg = OscMessage::from_bytes(&buf[..len]).unwrap();
        assert_eq!(msg.path, "/some/raw/osc/path");
        assert_eq!(msg.args[0], OscArg::Float(0.42));
    });

    let mut cmd = Command::cargo_bin("xair_set_scene").unwrap();
    cmd.arg("-i")
        .arg(console_addr.to_string())
        .arg("--delay")
        .arg("0")
        .write_stdin(input)
        .timeout(Duration::from_secs(10));

    cmd.assert().success();
    mock_thread.join().unwrap();
}

#[test]
fn test_xair_set_scene_long_lines_and_invalid_utf8() {
    let mock_console = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind mock console");
    let console_addr = mock_console
        .local_addr()
        .expect("Failed to get local address");

    let mut input = Vec::new();
    input.extend_from_slice(b"/ch/01/mix/fader 0.1\n");
    input.extend_from_slice(b"/bad/utf8 \xff\xfe\x00\n");

    let long_line = vec![b'A'; 4097];
    input.extend_from_slice(&long_line);
    input.push(b'\n');

    input.extend_from_slice(b"/ch/02/mix/fader 0.2\n");

    let mock_thread = thread::spawn(move || {
        let mut buf = [0; 512];
        mock_console
            .set_read_timeout(Some(Duration::from_millis(5000)))
            .unwrap();

        let (len, _) = mock_console.recv_from(&mut buf).expect("Failed to recv 1");
        let msg1 = OscMessage::from_bytes(&buf[..len]).unwrap();
        assert_eq!(msg1.path, "/ch/01/mix/fader");

        let (len, _) = mock_console.recv_from(&mut buf).expect("Failed to recv 2");
        let msg2 = OscMessage::from_bytes(&buf[..len]).unwrap();
        assert_eq!(msg2.path, "/ch/02/mix/fader");

        mock_console
            .set_read_timeout(Some(Duration::from_millis(100)))
            .unwrap();
        assert!(
            mock_console.recv_from(&mut buf).is_err(),
            "Extra message received"
        );
    });

    let mut cmd = Command::cargo_bin("xair_set_scene").unwrap();
    cmd.arg("-i")
        .arg(console_addr.to_string())
        .arg("--delay")
        .arg("0")
        .write_stdin(input)
        .timeout(Duration::from_secs(10));

    cmd.assert().success();
    mock_thread.join().unwrap();
}
