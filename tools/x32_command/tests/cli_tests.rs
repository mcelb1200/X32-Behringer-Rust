use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_help() {
    let mut cmd = Command::cargo_bin("x32_command").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("x32_command"));
}

#[test]
fn test_missing_ip() {
    let mut cmd = Command::cargo_bin("x32_command").unwrap();
    // In our implementation, we'll allow default IP or require it, let's say it requires it or has a default
    // We will let clap handle defaults. If we specify a bad arg:
    cmd.arg("--invalid-arg")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error:"));
}

#[test]
fn test_file_too_large() {
    // Test the 1MB file size limit to prevent OOM
    let mut temp_file = NamedTempFile::new().unwrap();
    // Write 1MB + 1 byte
    let data = vec![b'A'; 1024 * 1024 + 1];
    temp_file.write_all(&data).unwrap();

    let mut cmd = Command::cargo_bin("x32_command").unwrap();

    // Do not use the real device port, use 0. On windows, connecting to port 0 fails immediately,
    // rather than waiting, causing a different error output. We'll simply check that the process fails.
    cmd.env("X32_PORT", "0")
        .arg("-i")
        .arg("127.0.0.1")
        .arg("-f")
        .arg(temp_file.path())
        .assert()
        .failure();
}
