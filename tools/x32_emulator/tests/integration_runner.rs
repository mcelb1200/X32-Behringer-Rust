use std::process::Command;
use std::path::Path;

#[test]
fn run_scripts_integration_tests() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let workspace_root = Path::new(&manifest_dir)
        .parent()
        .expect("Failed to get parent of manifest dir")
        .parent()
        .expect("Failed to get workspace root");

    let (shell, args) = if cfg!(target_os = "windows") {
        ("powershell", vec!["-ExecutionPolicy", "Bypass", "-File", "./run_tests.ps1", "-Mode", "non_interactive", "-SkipBuild"])
    } else {
        ("bash", vec!["./run_tests.sh", "--run-tests-and-exit", "--skip-build"])
    };

    println!("Executing integration tests via: {} {:?}", shell, args);

    let output = Command::new(shell)
        .args(&args)
        .current_dir(workspace_root)
        .output()
        .expect("Failed to execute test runner script");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("--- TEST RUNNER STDOUT ---\n{}", stdout);
    eprintln!("--- TEST RUNNER STDERR ---\n{}", stderr);

    assert!(output.status.success(), "Integration test scripts failed!");
}
