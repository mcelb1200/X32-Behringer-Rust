use std::net::UdpSocket;
use std::thread;
use std::time::Duration;
use assert_cmd::prelude::*;
use std::process::Command;
use osc_lib::{OscMessage, OscArg};
use std::sync::{Arc, Mutex};

fn mock_x32_server(port: u16, brightness_changed: Arc<Mutex<bool>>) {
    let socket = UdpSocket::bind(format!("127.0.0.1:{}", port)).unwrap();
    let mut buf = [0; 512];

    loop {
        if let Ok((len, src)) = socket.recv_from(&mut buf) {
            if let Ok(msg) = OscMessage::from_bytes(&buf[..len]) {
                if msg.path == "/xremote" {
                    // Respond to keepalive
                    let response = OscMessage::new("/xremote".to_string(), vec![]);
                    socket.send_to(&response.to_bytes().unwrap(), src).unwrap();
                } else if msg.path == "/-prefs/bright" && msg.args.is_empty() {
                    // Respond with current brightness
                    let response = OscMessage::new("/-prefs/bright".to_string(), vec![OscArg::Float(0.8)]);
                    socket.send_to(&response.to_bytes().unwrap(), src).unwrap();
                } else if msg.path == "/-prefs/ledbright" && msg.args.is_empty() {
                    // Respond with current brightness
                    let response = OscMessage::new("/-prefs/ledbright".to_string(), vec![OscArg::Float(0.7)]);
                    socket.send_to(&response.to_bytes().unwrap(), src).unwrap();
                } else if msg.path == "/-prefs/bright" && msg.args.get(0) == Some(&OscArg::Float(0.0)) {
                    let mut bc = brightness_changed.lock().unwrap();
                    *bc = true;
                }
            }
        }
    }
}

#[test]
fn test_screen_saver_activation() {
    let port = 10025;
    let brightness_changed = Arc::new(Mutex::new(false));
    let brightness_changed_clone = Arc::clone(&brightness_changed);
    thread::spawn(move || {
        mock_x32_server(port, brightness_changed_clone);
    });
    thread::sleep(Duration::from_millis(100));

    let mut cmd = Command::cargo_bin("x32_screen_saver").unwrap();
    let mut child = cmd.arg("--ip").arg("127.0.0.1")
                       .arg("--delay").arg("1")
                       .env("X32_PORT", port.to_string())
                       .spawn()
                       .unwrap();

    thread::sleep(Duration::from_secs(2));

    let bc = brightness_changed.lock().unwrap();
    assert!(*bc, "Screen saver did not activate");

    child.kill().unwrap();
}
