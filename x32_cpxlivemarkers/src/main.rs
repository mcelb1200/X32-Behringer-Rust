use anyhow::{Context, Result, anyhow};
use byteorder::{LittleEndian, ReadBytesExt};
use clap::{Parser, ValueEnum};
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Reads markers from an XLive! session/SE_LOG.BIN file and prints them to stdout."
)]
struct Args {
    /// Path to the SE_LOG.BIN file or the session directory containing it
    #[arg(value_name = "SESSION_PATH")]
    session: PathBuf,

    /// Output format for the markers
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Reaper)]
    format: OutputFormat,

    /// Marker prefix (up to 5 characters)
    #[arg(short, long, default_value = "")]
    prefix: String,
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
enum OutputFormat {
    Reaper,
    Audition,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Validate prefix length (handle unicode chars safely)
    let prefix: String = args.prefix.chars().take(5).collect();

    // Determine the actual path to SE_LOG.BIN
    let mut file_path = args.session.clone();
    if file_path.is_dir() {
        file_path.push("SE_LOG.BIN");
    }

    if !file_path.exists() {
        return Err(anyhow!("File not found: {}", file_path.display()));
    }

    let markers = read_markers_from_file(&file_path)?;

    if markers.is_empty() {
        eprintln!("No markers found in {}", file_path.display());
        return Ok(());
    }

    let mut out = std::io::stdout();
    for (i, marker_sec) in markers.iter().enumerate() {
        let id = i + 1;
        match args.format {
            OutputFormat::Reaper => {
                // REAPER format: id time_in_seconds prefix+id 0 -1.0 0
                writeln!(out, "{} {:.6} {}{} 0 -1.0 0", id, marker_sec, prefix, id)?;
            }
            OutputFormat::Audition => {
                // Audition format: prefix+id, HH:MM:SS:FF, 00:00:00:00, 25fps, Cue, -
                // 25 fps conversion
                let xmk = *marker_sec;
                let xmkh = (xmk / 3600.0) as u32;
                let xmkm = ((xmk % 3600.0) / 60.0) as u32;
                let xmks = (xmk % 60.0) as u32;
                let xmkt = ((xmk - xmk.trunc()) * 25.0) as u32; // 100 / 4 = 25 for 25fps

                writeln!(
                    out,
                    "{}{}, {:02}:{:02}:{:02}:{:02}, 00:00:00:00, 25fps, Cue, -",
                    prefix, id, xmkh, xmkm, xmks, xmkt
                )?;
            }
        }
    }

    Ok(())
}

/// Reads the SE_LOG.BIN file and extracts markers as a vector of floats (seconds).
fn read_markers_from_file(path: &Path) -> Result<Vec<f32>> {
    let mut file =
        File::open(path).with_context(|| format!("Failed to open {}", path.display()))?;

    // Check file size, SE_LOG.BIN should be at least 2048 bytes but we only need up to markers
    let metadata = file.metadata()?;
    if metadata.len() < 1052 {
        return Err(anyhow!("File is too short to be a valid SE_LOG.BIN"));
    }

    // Read sample rate at offset 8
    file.seek(SeekFrom::Start(8))?;
    let sample_rate = file.read_u32::<LittleEndian>()?;
    if sample_rate == 0 {
        return Err(anyhow!("Invalid sample rate (0) in SE_LOG.BIN"));
    }

    // Read number of markers at offset 20
    file.seek(SeekFrom::Start(20))?;
    let nb_markers = file.read_u32::<LittleEndian>()?;

    if nb_markers == 0 {
        return Ok(Vec::new());
    }

    // Limit the number of markers based on file size to prevent OOM
    // Each marker is 4 bytes. Start offset is 1052.
    let remaining_bytes = metadata.len().saturating_sub(1052);
    let max_possible_markers = remaining_bytes / 4;
    let actual_nb_markers = std::cmp::min(nb_markers as u64, max_possible_markers) as usize;

    // Read markers starting at offset 1052
    file.seek(SeekFrom::Start(1052))?;
    let mut markers_sec = Vec::with_capacity(actual_nb_markers);

    for _ in 0..actual_nb_markers {
        let marker_samples = file.read_u32::<LittleEndian>()?;
        markers_sec.push(marker_samples as f32 / sample_rate as f32);
    }

    Ok(markers_sec)
}

#[cfg(test)]
mod tests {
    use super::*;
    use byteorder::{LittleEndian, WriteBytesExt};
    use std::io::{Cursor, Write};

    fn create_dummy_selog(sample_rate: u32, markers: &[u32]) -> Vec<u8> {
        let mut buf = vec![0u8; 2048];
        let mut cursor = Cursor::new(&mut buf);

        // Write sample rate at offset 8
        cursor.seek(SeekFrom::Start(8)).unwrap();
        cursor.write_u32::<LittleEndian>(sample_rate).unwrap();

        // Write nb_markers at offset 20
        cursor.seek(SeekFrom::Start(20)).unwrap();
        cursor
            .write_u32::<LittleEndian>(markers.len() as u32)
            .unwrap();

        // Write markers at offset 1052
        cursor.seek(SeekFrom::Start(1052)).unwrap();
        for &marker in markers {
            cursor.write_u32::<LittleEndian>(marker).unwrap();
        }

        buf
    }

    #[test]
    fn test_read_markers() {
        // 48000 sample rate
        // Marker 1: 48000 samples (1.0 sec)
        // Marker 2: 72000 samples (1.5 sec)
        let buf = create_dummy_selog(48000, &[48000, 72000]);

        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(&buf).unwrap();

        let markers = read_markers_from_file(file.path()).unwrap();
        assert_eq!(markers.len(), 2);
        assert!((markers[0] - 1.0).abs() < f32::EPSILON);
        assert!((markers[1] - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_format_audition() {
        let xmk: f32 = 3600.0 + 120.0 + 3.0 + 0.5; // 1h 2m 3s + 0.5s (12 frames at 25fps, 0.5 * 25 = 12.5 -> 12)

        let xmkh = (xmk / 3600.0) as u32;
        let xmkm = ((xmk % 3600.0) / 60.0) as u32;
        let xmks = (xmk % 60.0) as u32;
        let xmkt = ((xmk - xmk.trunc()) * 25.0) as u32;

        assert_eq!(xmkh, 1);
        assert_eq!(xmkm, 2);
        assert_eq!(xmks, 3);
        assert_eq!(xmkt, 12);

        let formatted = format!(
            "Prefix{}, {:02}:{:02}:{:02}:{:02}, 00:00:00:00, 25fps, Cue, -",
            1, xmkh, xmkm, xmks, xmkt
        );

        assert_eq!(
            formatted,
            "Prefix1, 01:02:03:12, 00:00:00:00, 25fps, Cue, -"
        );
    }
}
