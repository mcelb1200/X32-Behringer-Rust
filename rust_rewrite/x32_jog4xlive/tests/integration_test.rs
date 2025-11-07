
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use tokio::net::UdpSocket;
use anyhow::Result;

async fn get_free_port() -> u16 {
    (1025..=65535)
        .find(|port| std::net::UdpSocket::bind(("127.0.0.1", *port)).is_ok())
        .expect("Failed to find a free port")
}

#[tokio::test]
async fn test_cli_help() -> Result<()> {
    let mut cmd = Command::cargo_bin("x32_jog4xlive")?;
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
    Ok(())
}

#[tokio::test]
async fn test_integration() -> Result<()> {
    let mock_server_port = get_free_port().await;
    let mock_server_ip = "127.0.0.1";
    let mock_server_addr = format!("{}:{}", mock_server_ip, mock_server_port);

    let server_socket = UdpSocket::bind(&mock_server_addr).await?;

    let mut cmd = Command::cargo_bin("x32_jog4xlive")?;
    let mut child = cmd.arg("--ip").arg(mock_server_ip).spawn()?;

    let mut buf = [0; 512];

    // Capture the client address from the first setup message
    let (_, client_addr) = server_socket.recv_from(&mut buf).await?;

    // Receive the rest of the setup messages
    for _ in 0..4 {
        server_socket.recv_from(&mut buf).await?;
    }

    // Simulate jog wheel movement
    let jog_msg = osc_lib::OscMessage::new("/-stat/userpar/33/value".to_string(), vec![osc_lib::OscArg::Int(65)]);
    server_socket.send_to(&jog_msg.to_bytes()?, &client_addr).await?;

    // Expect the app to request the current time
    let (len, _) = server_socket.recv_from(&mut buf).await?;
    let etime_req = osc_lib::OscMessage::from_bytes(&buf[..len])?;
    assert_eq!(etime_req.path, "/-stat/urec/etime");

    // Send a fake etime response
    let etime_resp = osc_lib::OscMessage::new("/-stat/urec/etime".to_string(), vec![osc_lib::OscArg::Int(1000)]);
    server_socket.send_to(&etime_resp.to_bytes()?, &client_addr).await?;

    // Expect the app to set the new position
    let (len, _) = server_socket.recv_from(&mut buf).await?;
    let msg = osc_lib::OscMessage::from_bytes(&buf[..len])?;
    assert_eq!(msg.path, "/-action/setposition");

    // Expect the app to reset the encoder
    let (len, _) = server_socket.recv_from(&mut buf).await?;
    let msg = osc_lib::OscMessage::from_bytes(&buf[..len])?;
    assert_eq!(msg.path, "/-stat/userpar/33/value");


    child.kill()?;
    Ok(())
}
