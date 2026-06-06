use assert_cmd::prelude::*;
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

#[test]
fn test_x32_dump_consecutive_messages() {
    let mut file = NamedTempFile::new().unwrap();
    let msg1 = osc_lib::OscMessage {
        path: "/test/1".to_string(),
        args: vec![osc_lib::OscArg::Int(42)],
    };
    let msg2 = osc_lib::OscMessage {
        path: "/test/2".to_string(),
        args: vec![osc_lib::OscArg::Float(3.14)],
    };

    let bytes1 = osc_lib::OscMessage::serialize_to_bytes(&msg1.path, &msg1.args).unwrap();
    let bytes2 = osc_lib::OscMessage::serialize_to_bytes(&msg2.path, &msg2.args).unwrap();

    file.write_all(&bytes1).unwrap();
    file.write_all(&bytes2).unwrap();

    let mut cmd = Command::cargo_bin("x32_dump").unwrap();
    cmd.arg(file.path());

    cmd.assert().success();
}
