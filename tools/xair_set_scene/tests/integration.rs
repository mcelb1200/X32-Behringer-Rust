use assert_cmd::Command;
use osc_lib::OscMessage;
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
        .write_stdin("/ch/01/mix/fader ,f 0.75\n")
        .timeout(Duration::from_secs(5));

    cmd.assert().success();
    mock_thread.join().unwrap();
}
