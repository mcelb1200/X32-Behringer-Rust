use osc_lib::{OscArg, OscMessage};
use std::net::UdpSocket;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use x32_emulator::MixerModel;
use x32_emulator::server;

#[test]
fn test_emulator_with_model_wing() {
    let (tx, rx) = channel();
    let port = 10088;
    let bind_addr = format!("127.0.0.1:{}", port);

    let handle = thread::spawn(move || {
        server::run_with_model(&bind_addr, MixerModel::Wing, None, Some(rx)).unwrap();
    });

    thread::sleep(Duration::from_millis(200));

    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    socket
        .set_read_timeout(Some(Duration::from_secs(1)))
        .unwrap();

    let info_msg = OscMessage::new("/info".to_string(), vec![])
        .to_bytes()
        .unwrap();
    socket
        .send_to(&info_msg, format!("127.0.0.1:{}", port))
        .unwrap();

    let mut buf = [0u8; 1024];
    let (len, _) = socket.recv_from(&mut buf).unwrap();
    let response = OscMessage::from_bytes(&buf[..len]).unwrap();

    assert_eq!(response.path, "/info");
    assert_eq!(response.args[2], OscArg::String("Wing".to_string()));
    assert_eq!(
        response.args[1],
        OscArg::String("Wing Emulator".to_string())
    );

    let _ = tx.send(());
    handle.join().unwrap();
}

#[test]
fn test_emulator_with_model_xr18() {
    let (tx, rx) = channel();
    let port = 10089;
    let bind_addr = format!("127.0.0.1:{}", port);

    let handle = thread::spawn(move || {
        server::run_with_model(&bind_addr, MixerModel::XR18, None, Some(rx)).unwrap();
    });

    thread::sleep(Duration::from_millis(200));

    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    socket
        .set_read_timeout(Some(Duration::from_secs(1)))
        .unwrap();

    let info_msg = OscMessage::new("/info".to_string(), vec![])
        .to_bytes()
        .unwrap();
    socket
        .send_to(&info_msg, format!("127.0.0.1:{}", port))
        .unwrap();

    let mut buf = [0u8; 1024];
    let (len, _) = socket.recv_from(&mut buf).unwrap();
    let response = OscMessage::from_bytes(&buf[..len]).unwrap();

    assert_eq!(response.path, "/info");
    assert_eq!(response.args[2], OscArg::String("XR18".to_string()));
    assert_eq!(
        response.args[1],
        OscArg::String("XR18 Emulator".to_string())
    );

    let _ = tx.send(());
    handle.join().unwrap();
}

#[test]
fn test_emulator_cli_arg_parsing() {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("x32_emulator");
    cmd.arg("--help");

    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("-M, --model <MODEL>"));
}
