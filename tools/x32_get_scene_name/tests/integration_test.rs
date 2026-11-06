use osc_lib::{OscArg, OscMessage};
use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::time::Duration;

fn setup_mock_x32_server() -> SocketAddr {
    let socket = UdpSocket::bind("127.0.0.1:0").expect("couldn't bind to address");
    let addr = socket.local_addr().unwrap();
    thread::spawn(move || {
        let mut buf = [0; 512];
        loop {
            if let Ok((len, src)) = socket.recv_from(&mut buf) {
                let msg = OscMessage::from_bytes(&buf[..len]).unwrap();
                if msg.path == "/info" {
                    let response = OscMessage::new("/info".to_string(), vec![]);
                    socket.send_to(&response.to_bytes().unwrap(), src).unwrap();
                }

                // Send a scene change event
                thread::sleep(Duration::from_millis(100));
                let scene_change_msg =
                    OscMessage::new("/-show/prepos/current".to_string(), vec![OscArg::Int(5)]);
                socket
                    .send_to(&scene_change_msg.to_bytes().unwrap(), src)
                    .unwrap();

                // Respond to scene name request
                if let Ok((len, _)) = socket.recv_from(&mut buf) {
                    let msg = OscMessage::from_bytes(&buf[..len]).unwrap();
                    if msg.path == "/-show/showfile/scene/005" {
                        let response = OscMessage::new(
                            "/-show/showfile/scene/005".to_string(),
                            vec![OscArg::String("My Scene".to_string()), OscArg::Int(5)],
                        );
                        socket.send_to(&response.to_bytes().unwrap(), src).unwrap();
                    }
                }
            } else {
                break;
            }
        }
    });
    // Give the server a moment to start up
    thread::sleep(Duration::from_millis(100));
    addr
}

#[test]
fn test_get_scene_name_command() {
    let addr = setup_mock_x32_server();

    let bin = escargot::CargoBuild::new()
        .bin("x32_get_scene_name")
        .run()
        .unwrap();
    let mut cmd = bin.command();
    cmd.args(&["--ip", &addr.to_string(), "-o", "1", "--transport", "osc"]);

    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("05 - My Scene"));
}

#[test]
fn test_get_scene_name_with_model_args() {
    let addr = setup_mock_x32_server();

    let bin = escargot::CargoBuild::new()
        .bin("x32_get_scene_name")
        .run()
        .unwrap();

    // Test with Wing model flag and explicit mock port
    let mut cmd_wing = bin.command();
    cmd_wing.args(&[
        "-M",
        "Wing",
        "--ip",
        &addr.to_string(),
        "-o",
        "1",
        "--transport",
        "osc",
        "-v",
        "1",
    ]);
    let output_wing = cmd_wing.output().unwrap();
    assert!(output_wing.status.success());
    let stdout_wing = String::from_utf8(output_wing.stdout).unwrap();
    assert!(stdout_wing.contains("Connecting to Wing"));
    assert!(stdout_wing.contains("05 - My Scene"));

    // Test with XR18 model flag
    let mut cmd_xr18 = bin.command();
    cmd_xr18.args(&[
        "--model",
        "XR18",
        "--ip",
        &addr.to_string(),
        "-o",
        "1",
        "--transport",
        "osc",
        "-v",
        "1",
    ]);
    let output_xr18 = cmd_xr18.output().unwrap();
    assert!(output_xr18.status.success());
    let stdout_xr18 = String::from_utf8(output_xr18.stdout).unwrap();
    assert!(stdout_xr18.contains("Connecting to XR18"));
    assert!(stdout_xr18.contains("05 - My Scene"));
}
