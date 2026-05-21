use assert_cmd::cargo::cargo_bin_cmd;
use hound::{SampleFormat, WavReader, WavSpec, WavWriter};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn create_mock_xlive_session(dir: &Path, num_channels: u16, num_samples: u32, uppercase: bool) {
    let spec = WavSpec {
        channels: num_channels,
        sample_rate: 48000,
        bits_per_sample: 24,
        sample_format: SampleFormat::Int,
    };

    let filename = if uppercase {
        "00000001.WAV"
    } else {
        "00000001.wav"
    };

    let path = dir.join(filename);
    let mut writer = WavWriter::create(path, spec).unwrap();

    for s in 0..num_samples {
        for c in 0..num_channels {
            // Write a predictable value based on channel and sample
            // Using a simple ramp for each channel
            let value = (c as i32 * 1000) + (s as i32);
            writer.write_sample(value).unwrap();
        }
    }

    writer.finalize().unwrap();
}

#[test]
fn test_explode_all_channels() {
    let temp_dir = TempDir::new().unwrap();
    let session_dir = temp_dir.path().join("session");
    let dest_dir = temp_dir.path().join("dest");

    fs::create_dir(&session_dir).unwrap();
    fs::create_dir(&dest_dir).unwrap();

    create_mock_xlive_session(&session_dir, 4, 100, false);

    let mut cmd = cargo_bin_cmd!("x32_xlive_wav");
    cmd.arg(session_dir.to_str().unwrap())
        .arg("-d")
        .arg(dest_dir.to_str().unwrap())
        .arg("-S");

    cmd.assert().success();

    // Check that 4 mono files were created
    for i in 1..=4 {
        let expected_file = dest_dir.join(format!("Xlive_Wav_{}.wav", i));
        assert!(expected_file.exists());

        let mut reader = WavReader::open(expected_file).unwrap();
        assert_eq!(reader.spec().channels, 1);
        assert_eq!(reader.spec().bits_per_sample, 24);

        let samples: Vec<i32> = reader.samples::<i32>().map(|s| s.unwrap()).collect();
        assert_eq!(samples.len(), 100);

        // Verify channel 1 values
        if i == 1 {
            assert_eq!(samples[0], 0);
            assert_eq!(samples[99], 99);
        }

        // Verify channel 4 values
        if i == 4 {
            assert_eq!(samples[0], 3000);
            assert_eq!(samples[99], 3099);
        }
    }
}

#[test]
fn test_extract_subset_of_channels() {
    let temp_dir = TempDir::new().unwrap();
    let session_dir = temp_dir.path().join("session");
    let dest_dir = temp_dir.path().join("dest");

    fs::create_dir(&session_dir).unwrap();
    fs::create_dir(&dest_dir).unwrap();

    create_mock_xlive_session(&session_dir, 8, 50, false);

    let mut cmd = cargo_bin_cmd!("x32_xlive_wav");
    cmd.arg(session_dir.to_str().unwrap())
        .arg("-d")
        .arg(dest_dir.to_str().unwrap())
        .arg("-n")
        .arg("2")
        .arg("-S");

    cmd.assert().success();

    assert!(dest_dir.join("Xlive_Wav_1.wav").exists());
    assert!(dest_dir.join("Xlive_Wav_2.wav").exists());
    assert!(!dest_dir.join("Xlive_Wav_3.wav").exists());
}

#[test]
fn test_bit_depth_conversion() {
    let temp_dir = TempDir::new().unwrap();
    let session_dir = temp_dir.path().join("session");
    let dest_dir = temp_dir.path().join("dest");

    fs::create_dir(&session_dir).unwrap();
    fs::create_dir(&dest_dir).unwrap();

    create_mock_xlive_session(&session_dir, 2, 100, false);

    let mut cmd = cargo_bin_cmd!("x32_xlive_wav");
    cmd.arg(session_dir.to_str().unwrap())
        .arg("-d")
        .arg(dest_dir.to_str().unwrap())
        .arg("-c")
        .arg("16")
        .arg("-S");

    cmd.assert().success();

    let expected_file = dest_dir.join("Xlive_Wav_1.wav");
    assert!(expected_file.exists());

    let reader = WavReader::open(expected_file).unwrap();
    assert_eq!(reader.spec().bits_per_sample, 16);
}

#[test]
fn test_channel_name_override() {
    let temp_dir = TempDir::new().unwrap();
    let session_dir = temp_dir.path().join("session");
    let dest_dir = temp_dir.path().join("dest");

    fs::create_dir(&session_dir).unwrap();
    fs::create_dir(&dest_dir).unwrap();

    create_mock_xlive_session(&session_dir, 2, 50, false);

    let mut cmd = cargo_bin_cmd!("x32_xlive_wav");
    cmd.arg(session_dir.to_str().unwrap())
        .arg("-d")
        .arg(dest_dir.to_str().unwrap())
        .arg("-w")
        .arg("1,Vocals")
        .arg("-w")
        .arg("2,Guitar")
        .arg("-S");

    cmd.assert().success();

    assert!(dest_dir.join("Vocals.wav").exists());
    assert!(dest_dir.join("Guitar.wav").exists());
    assert!(!dest_dir.join("Xlive_Wav_1.wav").exists());
}

#[test]
fn test_prepend_number() {
    let temp_dir = TempDir::new().unwrap();
    let session_dir = temp_dir.path().join("session");
    let dest_dir = temp_dir.path().join("dest");

    fs::create_dir(&session_dir).unwrap();
    fs::create_dir(&dest_dir).unwrap();

    create_mock_xlive_session(&session_dir, 2, 50, false);

    let mut cmd = cargo_bin_cmd!("x32_xlive_wav");
    cmd.arg(session_dir.to_str().unwrap())
        .arg("-d")
        .arg(dest_dir.to_str().unwrap())
        .arg("-p")
        .arg("-w")
        .arg("1,Kick")
        .arg("-S");

    cmd.assert().success();

    assert!(dest_dir.join("01_Kick.wav").exists());
    assert!(dest_dir.join("02_Xlive_Wav_2.wav").exists());
}
