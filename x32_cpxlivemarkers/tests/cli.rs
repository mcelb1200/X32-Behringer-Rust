use assert_cmd::Command;
use byteorder::{LittleEndian, WriteBytesExt};
use predicates::prelude::*;
use std::io::{Seek, SeekFrom, Write};
use tempfile::NamedTempFile;

fn create_mock_bin(samprate: u32, nbmarker: u32, markers: &[u32]) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();

    // Fill the buffer with 2048 null bytes to meet size requirement
    let buf = vec![0u8; 2048];
    file.write_all(&buf).unwrap();

    // Set samprate at offset 8
    file.seek(SeekFrom::Start(8)).unwrap();
    file.write_u32::<LittleEndian>(samprate).unwrap();

    // Set nbmarker at offset 20
    file.seek(SeekFrom::Start(20)).unwrap();
    file.write_u32::<LittleEndian>(nbmarker).unwrap();

    // Write markers starting at offset 1052
    file.seek(SeekFrom::Start(1052)).unwrap();
    for &marker in markers {
        file.write_u32::<LittleEndian>(marker).unwrap();
    }

    file
}

#[test]
fn test_reaper_format() {
    // 48000 samprate, 2 markers:
    // marker 1: 48000 -> 1.0 second
    // marker 2: 96000 -> 2.0 seconds
    let file = create_mock_bin(48000, 2, &[48000, 96000]);

    let mut cmd = Command::cargo_bin("x32_cpxlivemarkers").unwrap();
    cmd.arg(file.path())
        .arg("--format")
        .arg("reaper")
        .arg("--prefix")
        .arg("M");

    cmd.assert().success().stdout(predicate::str::contains(
        "1 1.000000 M1 0 -1.0 0\n2 2.000000 M2 0 -1.0 0\n",
    ));
}

#[test]
fn test_audition_format() {
    // 48000 samprate, 1 marker:
    // marker 1: 48000 * 3661 -> 1h 1m 1s -> 3661 seconds
    let file = create_mock_bin(48000, 1, &[48000 * 3661]);

    let mut cmd = Command::cargo_bin("x32_cpxlivemarkers").unwrap();
    cmd.arg(file.path())
        .arg("--format")
        .arg("audition")
        .arg("--prefix")
        .arg("C");

    cmd.assert().success().stdout(predicate::str::contains(
        "C1, 01:01:01:00, 00:00:00:00, 25fps, Cue, -\n",
    ));
}

#[test]
fn test_no_markers() {
    let file = create_mock_bin(48000, 0, &[]);

    let mut cmd = Command::cargo_bin("x32_cpxlivemarkers").unwrap();
    cmd.arg(file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No Markers\n"));
}

#[test]
fn test_file_too_small() {
    let mut file = NamedTempFile::new().unwrap();
    let buf = vec![0u8; 100];
    file.write_all(&buf).unwrap();

    let mut cmd = Command::cargo_bin("x32_cpxlivemarkers").unwrap();
    cmd.arg(file.path());

    cmd.assert().failure().stderr(predicate::str::contains(
        "File is too small to be a valid SE_LOG.BIN file",
    ));
}

#[test]
fn test_missing_file() {
    let mut cmd = Command::cargo_bin("x32_cpxlivemarkers").unwrap();
    cmd.arg("non_existent_file.bin");

    cmd.assert().failure().stderr(predicate::str::contains(
        "Failed to open file: non_existent_file.bin",
    ));
}
