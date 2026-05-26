use assert_cmd::Command;
use osc_lib::{OscArg, OscMessage};
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_dump_valid_osc() -> Result<(), Box<dyn std::error::Error>> {
    let msg = OscMessage::new("/ch/01/mix/fader".to_string(), vec![OscArg::Float(0.75)]);
    let bytes = msg.to_bytes().unwrap();

    let mut file = NamedTempFile::new()?;
    file.write_all(&bytes)?;

    let mut cmd = Command::cargo_bin("x32_dump")?;
    cmd.arg(file.path());
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("/ch/01/mix/fader ,f 0.75"));

    Ok(())
}

#[test]
fn test_dump_stdin() -> Result<(), Box<dyn std::error::Error>> {
    let msg = OscMessage::new("/info".to_string(), vec![]);
    let bytes = msg.to_bytes().unwrap();

    let mut cmd = Command::cargo_bin("x32_dump")?;
    cmd.write_stdin(bytes);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("/info"));

    Ok(())
}
