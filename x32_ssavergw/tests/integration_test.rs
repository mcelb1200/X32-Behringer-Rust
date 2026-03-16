use predicates::prelude::*;
use std::process::{Child, Command as StdCommand};
use std::time::Duration;

struct EmulatorGuard {
    child: Child,
}

impl EmulatorGuard {
    fn start() -> Self {
        let child = StdCommand::new("cargo")
            .args(["run", "-p", "x32_emulator"])
            .spawn()
            .expect("Failed to start x32_emulator");

        // Give the emulator time to start up
        std::thread::sleep(Duration::from_millis(500));

        Self { child }
    }
}

impl Drop for EmulatorGuard {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn test_ssavergw_connection_timeout() {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("x32_ssavergw");
    cmd.arg("--ip").arg("127.0.0.99").arg("--delay").arg("1"); // Invalid IP to force timeout

    cmd.assert()
        .success()
        .stderr(predicate::str::contains("Connection timeout"));
}

#[test]
fn test_ssavergw_connects_and_dims() {
    let _emulator = EmulatorGuard::start();

    // We start ssavergw, let it run for a bit longer than the delay, then kill it
    // since it's a daemon. We'll use escargot to start the process to be able to control it
    // easily if assert_cmd timeout isn't sufficient, but assert_cmd with timeout is good enough.

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("x32_ssavergw");
    cmd.arg("--ip").arg("127.0.0.1").arg("--delay").arg("1");

    let result = cmd.timeout(Duration::from_secs(3)).assert();

    // The process will be killed by the timeout, which is expected.
    // We want to ensure it connected and entered low light mode.

    let output = result.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Connected!"));
    assert!(stdout.contains("Delay before Low Light: 1 seconds"));
    assert!(stdout.contains("Entered Low Light mode."));
}
