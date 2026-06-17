use std::process::Command;

fn main() {
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "-p",
            "x32_usb",
            "--",
            "--ip",
            "127.0.0.1:10047",
            "ls",
        ])
        .output()
        .unwrap();

    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
}
