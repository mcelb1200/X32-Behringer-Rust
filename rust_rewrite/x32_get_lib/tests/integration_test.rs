
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use x32_emulator::run;
use x32_core::Mixer;
use x32_lib::OscArg;

fn create_mock_mixer() -> Mixer {
    let mut mixer = Mixer::new();
    mixer.seed("/-libs/ch/001/hasdata".to_string(), OscArg::Int(1));
    mixer.seed(
        "-libs/ch/001".to_string(),
        OscArg::String(" \"TestPreset\" 0 0 0".to_string()),
    );
    mixer.seed("/ch/01/config/name".to_string(), OscArg::String("Test".to_string()));
    mixer.seed("/ch/01/config/icon".to_string(), OscArg::Int(1));
    mixer.seed("/ch/01/config/color".to_string(), OscArg::Int(2));
    mixer.seed("/ch/01/config/source".to_string(), OscArg::Int(0));
    mixer.seed("/ch/01/delay/on".to_string(), OscArg::Int(0));
    mixer.seed("/ch/01/delay/time".to_string(), OscArg::Float(0.5));
    mixer.seed("/ch/01/preamp/trim".to_string(), OscArg::Float(0.25));
    mixer.seed("/ch/01/preamp/invert".to_string(), OscArg::Int(0));
    mixer.seed("/ch/01/gate/on".to_string(), OscArg::Int(0));
    mixer.seed("/ch/01/gate/mode".to_string(), OscArg::Int(0));
    mixer.seed("/ch/01/gate/thr".to_string(), OscArg::Float(-60.0));
    mixer.seed("/ch/01/gate/range".to_string(), OscArg::Float(10.0));
    mixer.seed("/ch/01/gate/attack".to_string(), OscArg::Float(0.1));
    mixer.seed("/ch/01/gate/hold".to_string(), OscArg::Float(100.0));
    mixer.seed("/ch/01/gate/release".to_string(), OscArg::Float(50.0));
    mixer.seed("/ch/01/gate/filter/on".to_string(), OscArg::Int(0));
    mixer.seed("/ch/01/gate/filter/type".to_string(), OscArg::Int(0));
    mixer.seed("/ch/01/gate/filter/f".to_string(), OscArg::Float(1000.0));
    mixer.seed("/ch/01/dyn/on".to_string(), OscArg::Int(0));
    mixer.seed("/ch/01/dyn/mode".to_string(), OscArg::Int(0));
    mixer.seed("/ch/01/dyn/det".to_string(), OscArg::Int(0));
    mixer.seed("/ch/01/dyn/env".to_string(), OscArg::Int(0));
    mixer.seed("/ch/01/dyn/thr".to_string(), OscArg::Float(-40.0));
    mixer.seed("/ch/01/dyn/ratio".to_string(), OscArg::Int(0));
    mixer.seed("/ch/01/dyn/knee".to_string(), OscArg::Float(3.0));
    mixer.seed("/ch/01/dyn/mgain".to_string(), OscArg::Float(0.0));
    mixer.seed("/ch/01/dyn/attack".to_string(), OscArg::Float(1.0));
    mixer.seed("/ch/01/dyn/hold".to_string(), OscArg::Float(10.0));
    mixer.seed("/ch/01/dyn/release".to_string(), OscArg::Float(100.0));
    mixer.seed("/ch/01/dyn/pos".to_string(), OscArg::Int(0));
    mixer.seed("/ch/01/dyn/keysrc".to_string(), OscArg::Int(0));
    mixer.seed("/ch/01/dyn/mix".to_string(), OscArg::Float(100.0));
    mixer.seed("/ch/01/dyn/auto".to_string(), OscArg::Int(0));
    mixer.seed("/ch/01/dyn/filter/on".to_string(), OscArg::Int(0));
    mixer.seed("/ch/01/dyn/filter/type".to_string(), OscArg::Int(0));
    mixer.seed("/ch/01/dyn/filter/f".to_string(), OscArg::Float(1000.0));
    mixer.seed("/ch/01/eq/on".to_string(), OscArg::Int(0));
    for i in 1..=4 {
        mixer.seed(format!("/ch/01/eq/{}/type", i), OscArg::Int(2));
        mixer.seed(format!("/ch/01/eq/{}/f", i), OscArg::Float(1000.0));
        mixer.seed(format!("/ch/01/eq/{}/g", i), OscArg::Float(0.0));
        mixer.seed(format!("/ch/01/eq/{}/q", i), OscArg::Float(1.0));
    }
    mixer.seed("/ch/01/mix/on".to_string(), OscArg::Int(1));
    mixer.seed("/ch/01/mix/fader".to_string(), OscArg::Float(0.75));
    mixer.seed("/ch/01/mix/st".to_string(), OscArg::Int(0));
    mixer.seed("/ch/01/mix/pan".to_string(), OscArg::Float(0.0));
    mixer.seed("/ch/01/mix/mono".to_string(), OscArg::Int(0));
    mixer.seed("/ch/01/mix/mlevel".to_string(), OscArg::Float(0.0));
    for i in 1..=16 {
        mixer.seed(format!("/ch/01/mix/{:02}/on", i), OscArg::Int(0));
        mixer.seed(format!("/ch/01/mix/{:02}/level", i), OscArg::Float(0.0));
    }
    mixer
}


#[test]
fn test_save_channel_presets_integration() -> Result<(), Box<dyn std::error::Error>> {
    let mixer = create_mock_mixer();

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
    assert!(content.contains("/ch/01/config/name \"Test\""));


    // Clean up
    std::fs::remove_dir_all("/tmp/x32_get_lib_test")?;

    Ok(())
}
