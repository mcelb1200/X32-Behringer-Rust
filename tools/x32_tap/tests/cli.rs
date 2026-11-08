use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn test_help() {
    let mut cmd = cargo_bin_cmd!("x32_tap");
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Set the delay time"));
}

#[test]
fn test_invalid_slot() {
    let mut cmd = cargo_bin_cmd!("x32_tap");
    cmd.arg("--slot")
        .arg("9")
        .assert()
        .failure()
        .stderr(predicate::str::contains("FX slot must be between 1 and 8 for model X32"));
}

#[test]
fn test_invalid_slot_for_xr18() {
    let mut cmd = cargo_bin_cmd!("x32_tap");
    cmd.arg("-M")
        .arg("XR18")
        .arg("--slot")
        .arg("5")
        .assert()
        .failure()
        .stderr(predicate::str::contains("FX slot must be between 1 and 4 for model XR18"));
}

#[test]
fn test_invalid_channel_for_xr12() {
    let mut cmd = cargo_bin_cmd!("x32_tap");
    cmd.arg("--model")
        .arg("XR12")
        .arg("--channel")
        .arg("13")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Channel must be between 1 and 12 for model XR12"));
}
