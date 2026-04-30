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
        .arg("5")
        .assert()
        .failure()
        .stderr(predicate::str::contains("FX slot must be between 1 and 4"));
}
