use assert_cmd::Command;
use osc_lib::{OscArg, OscMessage};
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_multiple_messages_parsing() {
    let msg1 = OscMessage {
        path: "/ch/01/mix/fader".to_string(),
        args: vec![OscArg::Float(0.75)],
    };
    let msg2 = OscMessage {
        path: "/ch/02/mix/fader".to_string(),
        args: vec![OscArg::Float(0.50)],
    };

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(&msg1.to_bytes().unwrap()).unwrap();
    file.write_all(&msg2.to_bytes().unwrap()).unwrap();

    let mut cmd = Command::cargo_bin("x32_dump").unwrap();
    cmd.arg("--file").arg(file.path());
    let assert = cmd.assert();

    let output = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(output.contains("/ch/01/mix/fader ,f~[0.7500]"));
    assert!(output.contains("/ch/02/mix/fader ,f~[0.5000]"));
}
