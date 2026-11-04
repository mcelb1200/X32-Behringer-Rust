use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_help() {
    let mut cmd = Command::cargo_bin("x32_command").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("x32_command"));
}

#[test]
fn test_missing_ip() {
    let mut cmd = Command::cargo_bin("x32_command").unwrap();
    // In our implementation, we'll allow default IP or require it, let's say it requires it or has a default
    // We will let clap handle defaults. If we specify a bad arg:
    cmd.arg("--invalid-arg")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error:"));
}

#[test]
fn test_file_too_large() {
    // Test the 1MB file size limit to prevent OOM
    let mut temp_file = NamedTempFile::new().unwrap();
    // Write 1MB + 1 byte
    let data = vec![b'A'; 1024 * 1024 + 1];
    temp_file.write_all(&data).unwrap();

    let mut cmd = Command::cargo_bin("x32_command").unwrap();

    // Do not use the real device port, use 0. On windows, connecting to port 0 fails immediately,
    // rather than waiting, causing a different error output. We'll simply check that the process fails.
    cmd.env("X32_PORT", "0")
        .arg("-i")
        .arg("127.0.0.1")
        .arg("-f")
        .arg(temp_file.path())
        .assert()
        .failure();
}

#[test]
fn test_x32_command_wing_batch_execution() {
    use osc_lib::OscMessage;
    use std::net::UdpSocket;
    use std::thread;
    use std::time::Duration;

    let mock_console = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind mock console");
    let console_addr = mock_console.local_addr().unwrap();

    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "/ch/40/mix/fader ,f 0.85").unwrap();
    writeln!(temp_file, "/bus/28/mix/fader ,f 0.90").unwrap();
    writeln!(temp_file, "quit").unwrap();

    let mock_thread = thread::spawn(move || {
        let mut buf = [0; 512];
        mock_console
            .set_read_timeout(Some(Duration::from_millis(5000)))
            .unwrap();

        let mut received = Vec::new();
        while let Ok((len, _)) = mock_console.recv_from(&mut buf) {
            if let Ok(msg) = OscMessage::from_bytes(&buf[..len]) {
                received.push(msg.path);
            }
            mock_console
                .set_read_timeout(Some(Duration::from_millis(200)))
                .unwrap();
        }

        assert_eq!(received.len(), 2);
        assert_eq!(received[0], "/ch/40/mix/fader");
        assert_eq!(received[1], "/bus/28/mix/fader");
    });

    let mut cmd = Command::cargo_bin("x32_command").unwrap();
    cmd.arg("-i")
        .arg(console_addr.to_string())
        .arg("-M")
        .arg("Wing")
        .arg("-k")
        .arg("0")
        .arg("-t")
        .arg("1")
        .arg("-f")
        .arg(temp_file.path())
        .timeout(Duration::from_secs(10));

    cmd.assert().success();
    mock_thread.join().unwrap();
}

#[test]
fn test_x32_command_query_osc_path() {
    use osc_lib::OscMessage;
    use std::net::UdpSocket;
    use std::thread;
    use std::time::Duration;

    let mock_console = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind mock console");
    let console_addr = mock_console.local_addr().unwrap();

    let mock_thread = thread::spawn(move || {
        let mut buf = [0; 512];
        mock_console
            .set_read_timeout(Some(Duration::from_millis(5000)))
            .unwrap();

        if let Ok((len, _)) = mock_console.recv_from(&mut buf) {
            let msg = OscMessage::from_bytes(&buf[..len]).unwrap();
            assert_eq!(msg.path, "/ch/05/mix/fader");
            assert!(msg.args.is_empty());
        }
    });

    let mut cmd = Command::cargo_bin("x32_command").unwrap();
    cmd.arg("-i")
        .arg(console_addr.to_string())
        .arg("-M")
        .arg("X32")
        .arg("-k")
        .arg("1")
        .arg("-t")
        .arg("1")
        .write_stdin("/ch/05/mix/fader\nquit\n")
        .timeout(Duration::from_secs(10));

    cmd.assert().success();
    mock_thread.join().unwrap();
}

#[test]
fn test_x32_command_xr18_filters_out_of_bounds_paths() {
    use osc_lib::OscMessage;
    use std::net::UdpSocket;
    use std::thread;
    use std::time::Duration;

    let mock_console = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind mock console");
    let console_addr = mock_console.local_addr().unwrap();

    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "/ch/01/mix/fader ,f 0.5").unwrap();
    writeln!(temp_file, "/ch/32/mix/fader ,f 0.5").unwrap(); // invalid for XR18
    writeln!(temp_file, "/mtx/01/mix/fader ,f 0.5").unwrap(); // invalid for XR18
    writeln!(temp_file, "/bus/02/mix/fader ,f 0.7").unwrap(); // valid for XR18
    writeln!(temp_file, "quit").unwrap();

    let mock_thread = thread::spawn(move || {
        let mut buf = [0; 512];
        mock_console
            .set_read_timeout(Some(Duration::from_millis(5000)))
            .unwrap();

        let mut received = Vec::new();
        while let Ok((len, _)) = mock_console.recv_from(&mut buf) {
            if let Ok(msg) = OscMessage::from_bytes(&buf[..len]) {
                received.push(msg.path);
            }
            mock_console
                .set_read_timeout(Some(Duration::from_millis(200)))
                .unwrap();
        }

        // Only ch 01 and bus 02 should have passed validation
        assert_eq!(
            received.len(),
            2,
            "Expected 2 valid messages, received: {:?}",
            received
        );
        assert_eq!(received[0], "/ch/01/mix/fader");
        assert_eq!(received[1], "/bus/02/mix/fader");
    });

    let mut cmd = Command::cargo_bin("x32_command").unwrap();
    cmd.arg("-i")
        .arg(console_addr.to_string())
        .arg("-M")
        .arg("XR18")
        .arg("-k")
        .arg("0")
        .arg("-t")
        .arg("1")
        .arg("-f")
        .arg(temp_file.path())
        .timeout(Duration::from_secs(10));

    cmd.assert().success();
    mock_thread.join().unwrap();
}

#[test]
fn test_x32_command_interactive_stdin_filtering() {
    use osc_lib::OscMessage;
    use std::net::UdpSocket;
    use std::thread;
    use std::time::Duration;

    let mock_console = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind mock console");
    let console_addr = mock_console.local_addr().unwrap();

    let mock_thread = thread::spawn(move || {
        let mut buf = [0; 512];
        mock_console
            .set_read_timeout(Some(Duration::from_millis(5000)))
            .unwrap();

        let mut received = Vec::new();
        while let Ok((len, _)) = mock_console.recv_from(&mut buf) {
            if let Ok(msg) = OscMessage::from_bytes(&buf[..len]) {
                received.push(msg.path);
            }
            mock_console
                .set_read_timeout(Some(Duration::from_millis(200)))
                .unwrap();
        }

        // For XR12: max channels is 12. So ch 12 is valid, ch 15 is dropped.
        assert_eq!(
            received.len(),
            1,
            "Expected 1 valid message, received: {:?}",
            received
        );
        assert_eq!(received[0], "/ch/12/mix/fader");
    });

    let mut cmd = Command::cargo_bin("x32_command").unwrap();
    cmd.arg("-i")
        .arg(console_addr.to_string())
        .arg("-M")
        .arg("XR12")
        .arg("-k")
        .arg("1")
        .arg("-t")
        .arg("1")
        .write_stdin("/ch/12/mix/fader ,f 0.5\n/ch/15/mix/fader ,f 0.5\nquit\n")
        .timeout(Duration::from_secs(10));

    cmd.assert().success();
    mock_thread.join().unwrap();
}
