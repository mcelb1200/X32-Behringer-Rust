use assert_cmd::Command;
use osc_lib::{OscArg, OscMessage};
use std::net::UdpSocket;
use std::thread;
use std::time::Duration;

#[test]
fn test_x32_set_scene_with_cli_args() {
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

    let mut cmd = Command::cargo_bin("x32_set_scene").unwrap();
    cmd.arg("-i")
        .arg(console_addr.to_string())
        .arg("--transport")
        .arg("osc")
        .write_stdin("/ch/01/mix/fader 0.75\n")
        .timeout(Duration::from_secs(10));

    cmd.assert().success();
    mock_thread.join().unwrap();
}

#[test]
fn test_x32_set_scene_filters_out_of_bounds_paths_per_model() {
    let mock_console = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind mock console");
    let console_addr = mock_console
        .local_addr()
        .expect("Failed to get local address");

    let input = "\
/ch/01/mix/fader 0.5
/ch/20/mix/fader 0.75
/bus/01/mix/fader 0.9
/mtx/01/mix/fader 0.8
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
                .set_read_timeout(Some(Duration::from_millis(150)))
                .unwrap();
        }

        // Under XR18: channel 20 and mtx 01 are not supported.
        assert_eq!(
            received.len(),
            2,
            "Expected 2 messages for XR18, got {:?}",
            received
        );
        assert_eq!(received[0], "/ch/01/mix/fader");
        assert_eq!(received[1], "/bus/01/mix/fader");
    });

    let mut cmd = Command::cargo_bin("x32_set_scene").unwrap();
    cmd.arg("-i")
        .arg(console_addr.to_string())
        .arg("-M")
        .arg("XR18")
        .arg("--transport")
        .arg("osc")
        .arg("--delay")
        .arg("0")
        .write_stdin(input)
        .timeout(Duration::from_secs(10));

    cmd.assert().success();
    mock_thread.join().unwrap();
}

#[test]
fn test_x32_set_scene_wing_model_support() {
    let mock_console = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind mock console");
    let console_addr = mock_console
        .local_addr()
        .expect("Failed to get local address");

    // Wing supports up to 40 channels and 28 buses
    let input = "\
/ch/40/mix/fader 0.8
/bus/28/mix/fader 0.9
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
                .set_read_timeout(Some(Duration::from_millis(150)))
                .unwrap();
        }

        assert_eq!(received.len(), 2);
        assert_eq!(received[0], "/ch/40/mix/fader");
        assert_eq!(received[1], "/bus/28/mix/fader");
    });

    let mut cmd = Command::cargo_bin("x32_set_scene").unwrap();
    cmd.arg("-i")
        .arg(console_addr.to_string())
        .arg("-M")
        .arg("Wing")
        .arg("--transport")
        .arg("osc")
        .arg("--delay")
        .arg("0")
        .write_stdin(input)
        .timeout(Duration::from_secs(10));

    cmd.assert().success();
    mock_thread.join().unwrap();
}

#[test]
fn test_x32_set_scene_raw_osc_fallback() {
    let mock_console = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind mock console");
    let console_addr = mock_console
        .local_addr()
        .expect("Failed to get local address");

    let input = "/custom/osc/path ,f 0.42\n";

    let mock_thread = thread::spawn(move || {
        let mut buf = [0; 512];
        mock_console
            .set_read_timeout(Some(Duration::from_millis(5000)))
            .unwrap();
        let (len, _) = mock_console
            .recv_from(&mut buf)
            .expect("Failed to receive raw osc msg");
        let msg = OscMessage::from_bytes(&buf[..len]).unwrap();
        assert_eq!(msg.path, "/custom/osc/path");
        assert_eq!(msg.args[0], OscArg::Float(0.42));
    });

    let mut cmd = Command::cargo_bin("x32_set_scene").unwrap();
    cmd.arg("-i")
        .arg(console_addr.to_string())
        .arg("--transport")
        .arg("osc")
        .arg("--delay")
        .arg("0")
        .write_stdin(input)
        .timeout(Duration::from_secs(10));

    cmd.assert().success();
    mock_thread.join().unwrap();
}

#[test]
fn test_x32_set_scene_long_lines_error() {
    let mock_console = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind mock console");
    let console_addr = mock_console
        .local_addr()
        .expect("Failed to get local address");

    let mut input = Vec::new();
    let long_line = vec![b'A'; 4097];
    input.extend_from_slice(&long_line);
    input.push(b'\n');

    let mut cmd = Command::cargo_bin("x32_set_scene").unwrap();
    cmd.arg("-i")
        .arg(console_addr.to_string())
        .arg("--transport")
        .arg("osc")
        .arg("--delay")
        .arg("0")
        .write_stdin(input)
        .timeout(Duration::from_secs(10));

    cmd.assert().failure();
}
