
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use x32_emulator::run;
use x32_core::Mixer;
use x32_lib::OscArg;

#[test]
fn test_save_channel_presets_integration() -> Result<(), Box<dyn std::error::Error>> {
    let mut mixer = Mixer::new();
    mixer.seed("/-libs/ch/001/hasdata".to_string(), OscArg::Int(1));
    mixer.seed("/-libs/ch/001".to_string(), OscArg::String("                                TestPreset".to_string()));


    // Start the emulator
    let emulator_addr = run(mixer, "127.0.0.1".to_string(), 0)?;
    let client_port = emulator_addr.port() + 1;

    let mut cmd = Command::cargo_bin("x32_get_lib")?;
    cmd.arg("--ip")
        .arg(emulator_addr.ip().to_string())
        .arg("--port")
        .arg(client_port.to_string())
        .arg("--remote-port")
        .arg(emulator_addr.port().to_string())
        .arg("--directory")
        .arg("/tmp/x32_get_lib_test")
        .arg("--library-type")
        .arg("channel");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Saving channel presets..."))
        .stdout(predicate::str::contains("Saving channel preset 1: TestPreset"));

    // Verify that the file was created
    let path = std::path::Path::new("/tmp/x32_get_lib_test").join("TestPreset.chn");
    assert!(path.exists());

    let content = std::fs::read_to_string(path)?;
    assert!(content.contains("#2.1#"));
    assert!(content.contains("TestPreset"));


    // Clean up
    std::fs::remove_dir_all("/tmp/x32_get_lib_test")?;

    Ok(())
}
