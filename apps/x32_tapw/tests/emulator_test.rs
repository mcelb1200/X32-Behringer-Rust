use std::process::Command;

// Testing interactive TUIs is tricky because x32_tapw takes over terminal raw mode.
// We test CLI parsing, model resolution, and slot/channel bounds directly in tests/cli.rs
// and in src/lib.rs unit tests. Here we verify that the binary launches and responds to --help.

#[test]
fn test_tapw_binary_exists() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_x32_tapw"));
    cmd.arg("--help");
    let output = cmd.output().expect("Failed to execute x32_tapw");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("X32Tap utility with a Text User Interface"));
    assert!(stdout.contains("--model"));
}
