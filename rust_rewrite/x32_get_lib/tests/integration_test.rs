
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use std::net::UdpSocket;
use std::thread;
use x32_lib::{OscMessage, OscArg};

#[test]
fn test_save_channel_presets_integration() -> Result<(), Box<dyn std::error::Error>> {
    // Start a mock UDP server
    let server = UdpSocket::bind("127.0.0.1:0")?;
    let server_addr = server.local_addr()?;
    let client_port = server_addr.port() + 1;
    let client_addr = format!("127.0.0.1:{}", client_port);


    let handle = thread::spawn(move || {
        let mut buf = [0; 1024];
        // Test hasdata
        let (len, src_addr) = server.recv_from(&mut buf).unwrap();
        let msg = OscMessage::from_bytes(&buf[..len]).unwrap();
        assert_eq!(msg.path, "/-libs/ch/001/hasdata");

        let response = OscMessage::new("/-libs/ch/001/hasdata".to_string(), vec![OscArg::Int(0)]);
        server.send_to(&response.to_bytes().unwrap(), &src_addr).unwrap();

        // Test hasdata for empty preset
        for i in 2..=100 {
            server.recv_from(&mut buf).unwrap();
            let response = OscMessage::new(format!("/-libs/ch/{:03}/hasdata", i), vec![OscArg::Int(0)]);
            server.send_to(&response.to_bytes().unwrap(), &src_addr).unwrap();
        }
    });

    let mut cmd = Command::cargo_bin("x32_get_lib")?;
    cmd.arg("--ip")
        .arg(server_addr.ip().to_string())
        .arg("--port")
        .arg(client_port.to_string())
        .arg("--remote-port")
        .arg(server_addr.port().to_string())
        .arg("--directory")
        .arg("/tmp/x32_get_lib_test")
        .arg("--library-type")
        .arg("channel");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Saving channel presets..."));

    handle.join().unwrap();

    // Verify that no files were created
    let paths = std::fs::read_dir("/tmp/x32_get_lib_test")?;
    assert_eq!(paths.count(), 0);


    // Clean up
    std::fs::remove_dir_all("/tmp/x32_get_lib_test")?;

    Ok(())
}
