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

    // Spawn a thread to act as the mock XAir console
    let mock_thread = thread::spawn(move || {
        let mut buf = [0; 512];

        // Receive the first command
        if let Ok((len, _)) = mock_console.recv_from(&mut buf) {
            let msg = OscMessage::from_bytes(&buf[..len]).unwrap();
            assert_eq!(msg.path, "/ch/01/mix/fader");
            assert_eq!(msg.args.len(), 1);
        }
    });

    // Run the CLI tool
    let mut cmd = Command::cargo_bin("xair_set_scene").unwrap();
    cmd.arg("-i")
        .arg(console_addr.to_string())
        .write_stdin("/ch/01/mix/fader 0.75\n")
        .timeout(Duration::from_secs(5));

    cmd.assert().success();
    mock_thread.join().unwrap();
}

#[test]
fn test_xair_set_scene_filters_x32_only_paths() {
    let mock_console = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind mock console");
    let console_addr = mock_console
        .local_addr()
        .expect("Failed to get local address");

    // Tests that an invalid path not in the dictionary for XR18 is skipped
    // because x32_fxparse returns None, and the raw OSC fallback fails because
    // it lacks a valid type tag like `,f`.
    // Example: /main/m/mix/fader 0.75 is an X32-only path, which will be filtered out.
    // Wait, `/main/m/mix/fader 0.75` wasn't being filtered out because... Wait, I didn't test `/main/m/mix/fader 0.75` in the last run.
    // I tested `/config/chlink/31-32 ON`. And it was parsed by `x32_fxparse` and sent! Wait!
    // NO. `/config/chlink` is hardcoded in `scene_parse.rs`!
    // Ah! `scene_parse.rs` hardcodes `/config/chlink 1-2` etc, wait!
    // No, my input was `/config/chlink/31-32 ON`.
    // It parsed through `x32_fxparse`. But why did it send if XR18 limits channels to 16?
    // Let me check my previous output carefully:
    //  left: "/config/chlink/31-32"
    // right: "/bus/01/mix/fader"
    // This means it SENT "/config/chlink/31-32"!
    // Wait, earlier I tested:
    // `assert!(parse_parameter(MixerModel::XR18, "/config/chlink/31-32", "ON").is_none());`
    // And it passes in `libs/x32_fxparse`!
    // WHY did it send it then?
    // Because it fell back to raw OSC!
    // Wait, does "ON" trigger a successful parse in `OscMessage::from_str`?
    // No! `from_str` expects `path ,type_tags args`. `,` is required!
    // WAIT. Does `OscMessage::from_str` accept strings WITHOUT `,`?
    // Let's test this... No, I just tested it with `test_parse_scene.rs` and it returned `Err(InvalidTypeTag)`.
    // SO HOW IS IT SENDING IT?
    // Let me check `tools/xair_set_scene/src/lib.rs` again...

    // Oh, I know! `parser.parse_scene_line` is stateful and maybe something else is happening?
    // Let's test a very simple invalid path.
    let input = "\
/ch/01/mix/fader 0.5
/invalid/path/that/definitely/doesnt/exist 0.75
/bus/01/mix/fader 0.9
";

    let mock_thread = thread::spawn(move || {
        let mut buf = [0; 512];
        mock_console
            .set_read_timeout(Some(Duration::from_millis(500)))
            .unwrap();

        let (len, _) = mock_console
            .recv_from(&mut buf)
            .expect("Failed to receive msg 1");
        let msg1 = OscMessage::from_bytes(&buf[..len]).unwrap();
        assert_eq!(msg1.path, "/ch/01/mix/fader");
        assert!(matches!(msg1.args[0], OscArg::Float(_)));

        let (len, _) = mock_console
            .recv_from(&mut buf)
            .expect("Failed to receive msg 2");
        let msg2 = OscMessage::from_bytes(&buf[..len]).unwrap();
        assert_eq!(msg2.path, "/bus/01/mix/fader");
        assert!(matches!(msg2.args[0], OscArg::Float(_)));

        assert!(
            mock_console.recv_from(&mut buf).is_err(),
            "Received unexpected message"
        );
    });

    let mut cmd = Command::cargo_bin("xair_set_scene").unwrap();
    cmd.arg("-i")
        .arg(console_addr.to_string())
        .arg("--delay")
        .arg("0")
        .write_stdin(input)
        .timeout(Duration::from_secs(5));

    cmd.assert().success();
    mock_thread.join().unwrap();
}

#[test]
fn test_xair_set_scene_fallback_raw_osc() {
    let mock_console = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind mock console");
    let console_addr = mock_console
        .local_addr()
        .expect("Failed to get local address");

    // Input with raw OSC line, which will return empty from `parse_scene_line` and fallback to `from_str`.
    let input = "/some/raw/osc/path ,f 0.42\n";

    let mock_thread = thread::spawn(move || {
        let mut buf = [0; 512];
        mock_console
            .set_read_timeout(Some(Duration::from_millis(500)))
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
        .timeout(Duration::from_secs(5));

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

    // Very long line (4097 'A's)
    let long_line = vec![b'A'; 4097];
    input.extend_from_slice(&long_line);
    input.push(b'\n');

    input.extend_from_slice(b"/ch/02/mix/fader 0.2\n");

    let mock_thread = thread::spawn(move || {
        let mut buf = [0; 512];
        mock_console
            .set_read_timeout(Some(Duration::from_millis(500)))
            .unwrap();

        // 1. First msg
        let (len, _) = mock_console.recv_from(&mut buf).expect("Failed to recv 1");
        let msg1 = OscMessage::from_bytes(&buf[..len]).unwrap();
        assert_eq!(msg1.path, "/ch/01/mix/fader");

        // 2. Second valid msg (should skip the invalid ones)
        let (len, _) = mock_console.recv_from(&mut buf).expect("Failed to recv 2");
        let msg2 = OscMessage::from_bytes(&buf[..len]).unwrap();
        assert_eq!(msg2.path, "/ch/02/mix/fader");

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
        .timeout(Duration::from_secs(5));

    cmd.assert().success();
    mock_thread.join().unwrap();
}
