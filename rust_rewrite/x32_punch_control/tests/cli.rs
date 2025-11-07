use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn test_missing_ip() {
    let mut cmd = Command::cargo_bin("x32_punch_control").unwrap();
    cmd.arg("--output-file")
        .arg("test.xpc")
        .assert()
        .failure()
        .stderr(predicates::str::contains("the following required arguments were not provided"));
}

#[test]
fn test_missing_output_file() {
    let mut cmd = Command::cargo_bin("x32_punch_control").unwrap();
    cmd.arg("--ip")
        .arg("127.0.0.1")
        .assert()
        .failure()
        .stderr(predicates::str::contains("the following required arguments were not provided"));
}

#[test]
fn test_help_flag() {
    let mut cmd = Command::cargo_bin("x32_punch_control").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("Usage: x32_punch_control"));
}

#[test]
fn test_list_midi_devices() {
    let pm = portmidi::PortMidi::new().unwrap();
    let devices = pm.devices().unwrap();
    for device in devices {
        println!("{:?}", device);
    }
}
