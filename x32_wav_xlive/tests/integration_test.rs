use tempfile::tempdir;
use hound::{WavWriter, WavSpec, WavReader};
use std::path::Path;
use std::fs::{self, File};
use std::io::{Cursor, Read};
use byteorder::{LittleEndian, ReadBytesExt};


fn create_test_wav(dir: &Path, name: &str, spec: WavSpec, duration_ms: u32) {
    let path = dir.join(name);
    let mut writer = WavWriter::create(&path, spec).unwrap();
    let num_samples = (spec.sample_rate as u32 * duration_ms) / 1000;
    for _ in 0..num_samples {
        writer.write_sample(0i32).unwrap();
    }
    writer.finalize().unwrap();
}

#[test]
fn test_cli_e2e() {
    let dir = tempdir().unwrap();
    let spec = WavSpec {
        channels: 1,
        sample_rate: 48000,
        bits_per_sample: 24,
        sample_format: hound::SampleFormat::Int,
    };
    create_test_wav(dir.path(), "ch_1.wav", spec, 100);
    create_test_wav(dir.path(), "ch_2.wav", spec, 100);

    let bin = escargot::CargoBuild::new()
        .bin("x32_wav_xlive")
        .run()
        .unwrap();
    let mut cmd = bin.command();
    cmd.arg(dir.path())
        .arg("TestSession")
        .arg("-m")
        .arg("0.05");

    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Found 2 WAV files to process."));


    let session_dir = fs::read_dir(dir.path())
        .unwrap()
        .find(|entry| entry.as_ref().unwrap().path().is_dir())
        .expect("No session directory found")
        .unwrap()
        .path();

    let log_path = session_dir.join("SE_LOG.BIN");
    assert!(log_path.exists());

    let wav_files: Vec<_> = fs::read_dir(&session_dir)
        .unwrap()
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            if path.extension().map_or(false, |ext| ext == "wav" || ext == "WAV") {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    assert_eq!(wav_files.len(), 1);
}

#[test]
fn test_single_take_se_log_bin() {
    let dir = tempdir().unwrap();
    let spec = WavSpec {
        channels: 1,
        sample_rate: 48000,
        bits_per_sample: 24,
        sample_format: hound::SampleFormat::Int,
    };
    create_test_wav(dir.path(), "ch_1.wav", spec, 100);

    let bin = escargot::CargoBuild::new()
        .bin("x32_wav_xlive")
        .run()
        .unwrap();
    let mut cmd = bin.command();
    cmd.arg(dir.path());

    let output = cmd.output().unwrap();
    assert!(output.status.success());

    let session_dir = fs::read_dir(dir.path())
        .unwrap()
        .find(|entry| entry.as_ref().unwrap().path().is_dir())
        .expect("No session directory found")
        .unwrap()
        .path();

    let log_path = session_dir.join("SE_LOG.BIN");
    let mut file = File::open(log_path).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    let mut cursor = Cursor::new(&buffer);
    cursor.set_position(28); // Skip to the take sizes array

    let reader = WavReader::open(dir.path().join("ch_1.wav")).unwrap();
    let expected_size = reader.duration();
    let actual_size = cursor.read_u32::<LittleEndian>().unwrap();
    assert_eq!(actual_size, expected_size);
}
