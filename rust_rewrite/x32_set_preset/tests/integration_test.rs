use assert_cmd::prelude::*;
use std::process::Command;
use std::net::UdpSocket;
use tempfile::NamedTempFile;
use std::io::Write;
use osc_lib::{OscArg, OscMessage};

fn setup() {
    let _ = std::fs::create_dir_all("target/tmp");
}

#[test]
fn test_channel_preset_explicit_types() -> Result<(), Box<dyn std::error::Error>> {
    setup();
    let mut file = NamedTempFile::new_in("target/tmp")?;
    file.write_all(b"/config/name \"Test Name\" ,s\n")?;
    file.write_all(b"/mix/fader 0.75 ,f\n")?;
    let file_path = file.into_temp_path();
    let chn_file_path = file_path.with_extension("chn");
    std::fs::rename(&file_path, &chn_file_path)?;

    let socket = UdpSocket::bind("127.0.0.1:0")?;
    let addr = socket.local_addr()?;
    let ip = addr.ip().to_string();
    let port = addr.port().to_string();
    let mut cmd = Command::cargo_bin("x32_set_preset")?;
    cmd.arg("--ip")
        .arg(&ip)
        .arg("--port")
        .arg(&port)
        .arg("-f")
        .arg(chn_file_path.to_str().unwrap())
        .arg("-s")
        .arg("1");

    cmd.assert().success();

    let mut buf = [0; 1024];
    let (amt, _) = socket.recv_from(&mut buf)?;
    let msg = OscMessage::from_bytes(&buf[..amt])?;
    assert_eq!(msg.path, "/ch/01/config/name");
    assert_eq!(msg.args, vec![OscArg::String("Test Name".to_string())]);

    let (amt, _) = socket.recv_from(&mut buf)?;
    let msg = OscMessage::from_bytes(&buf[..amt])?;
    assert_eq!(msg.path, "/ch/01/mix/fader");
    assert_eq!(msg.args, vec![OscArg::Float(0.75)]);

    Ok(())
}

#[test]
fn test_channel_preset_inferred_types() -> Result<(), Box<dyn std::error::Error>> {
    setup();
    let mut file = NamedTempFile::new_in("target/tmp")?;
    file.write_all(b"/config/name \"Another Name\"\n")?;
    file.write_all(b"/mix/fader 0.5\n")?;
    file.write_all(b"/config/color 3\n")?;
    let file_path = file.into_temp_path();
    let chn_file_path = file_path.with_extension("chn");
    std::fs::rename(&file_path, &chn_file_path)?;

    let socket = UdpSocket::bind("127.0.0.1:0")?;
    let addr = socket.local_addr()?;
    let ip = addr.ip().to_string();
    let port = addr.port().to_string();
    let mut cmd = Command::cargo_bin("x32_set_preset")?;
    cmd.arg("--ip")
        .arg(&ip)
        .arg("--port")
        .arg(&port)
        .arg("-f")
        .arg(chn_file_path.to_str().unwrap())
        .arg("-s")
        .arg("2");

    cmd.assert().success();

    let mut buf = [0; 1024];
    let (amt, _) = socket.recv_from(&mut buf)?;
    let msg = OscMessage::from_bytes(&buf[..amt])?;
    assert_eq!(msg.path, "/ch/02/config/name");
    assert_eq!(msg.args, vec![OscArg::String("Another Name".to_string())]);

    let (amt, _) = socket.recv_from(&mut buf)?;
    let msg = OscMessage::from_bytes(&buf[..amt])?;
    assert_eq!(msg.path, "/ch/02/mix/fader");
    assert_eq!(msg.args, vec![OscArg::Float(0.5)]);

    let (amt, _) = socket.recv_from(&mut buf)?;
    let msg = OscMessage::from_bytes(&buf[..amt])?;
    assert_eq!(msg.path, "/ch/02/config/color");
    assert_eq!(msg.args, vec![OscArg::Int(3)]);

    Ok(())
}

#[test]
fn test_effect_preset() -> Result<(), Box<dyn std::error::Error>> {
    setup();
    let mut file = NamedTempFile::new_in("target/tmp")?;
    file.write_all(b"type 1 ,i\n")?;
    file.write_all(b"par 0.1 0.2 0.3 ,fff\n")?;
    let file_path = file.into_temp_path();
    let efx_file_path = file_path.with_extension("efx");
    std::fs::rename(&file_path, &efx_file_path)?;

    let socket = UdpSocket::bind("127.0.0.1:0")?;
    let addr = socket.local_addr()?;
    let ip = addr.ip().to_string();
    let port = addr.port().to_string();
    let mut cmd = Command::cargo_bin("x32_set_preset")?;
    cmd.arg("--ip")
        .arg(&ip)
        .arg("--port")
        .arg(&port)
        .arg("-f")
        .arg(efx_file_path.to_str().unwrap())
        .arg("-s")
        .arg("3");

    cmd.assert().success();

    let mut buf = [0; 1024];
    let (amt, _) = socket.recv_from(&mut buf)?;
    let msg = OscMessage::from_bytes(&buf[..amt])?;
    assert_eq!(msg.path, "/fx/3/type");
    assert_eq!(msg.args, vec![OscArg::Int(1)]);

    let (amt, _) = socket.recv_from(&mut buf)?;
    let msg = OscMessage::from_bytes(&buf[..amt])?;
    assert_eq!(msg.path, "/fx/3/par");
    assert_eq!(msg.args, vec![OscArg::Float(0.1), OscArg::Float(0.2), OscArg::Float(0.3)]);

    Ok(())
}
