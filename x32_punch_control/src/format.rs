use anyhow::Result;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};

/// Represents a single recorded OSC message in a punch control file.
#[derive(Debug, Clone)]
pub struct PunchRecord {
    pub time: Duration,
    pub data: Vec<u8>,
}

/// A reader for `.xpc` files.
pub struct PunchReader {
    reader: BufReader<File>,
}

impl PunchReader {
    pub fn new(file: File) -> Self {
        Self {
            reader: BufReader::new(file),
        }
    }

    /// Reads the next record from the file.
    /// Returns `Ok(Some(PunchRecord))` if a record was read,
    /// `Ok(None)` if EOF was reached, or an error.
    pub async fn read_record(&mut self) -> Result<Option<PunchRecord>> {
        // In c_origin, dt_read is a `struct timeval`.
        // We'll standardize this to 8 bytes for seconds and 4 bytes for microseconds
        // Little Endian encoding, similar to X32Replay.
        let sec = match self.reader.read_u64_le().await {
            Ok(s) => s,
            Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e.into()),
        };

        // Some systems use 8 bytes for suseconds_t in timeval (like 64-bit Linux).
        // For portability across the Rust tools, let's use 8 bytes sec + 8 bytes usec or
        // we follow the 8 + 4 bytes standard of X32Replay. However, C code does `sizeof(dt_read)`.
        // If dt_read is a standard timeval on Linux x86_64, it's 16 bytes.
        // On Windows (where X32PunchControl was originally targeted), timeval is typically:
        // struct timeval { long tv_sec; long tv_usec; }; -> 8 bytes total on 32-bit, or 8 bytes on 64-bit Windows (long is 32-bit).
        // Since we are writing this in Rust, we will define our format consistently:
        // 8 bytes (sec), 4 bytes (usec).

        let usec = self.reader.read_u32_le().await?;
        let len = self.reader.read_u32_le().await?;

        if len == 0 || len > 65535 {
            return Err(anyhow::anyhow!("Invalid length read from file: {}", len));
        }

        let mut data = vec![0u8; len as usize];
        self.reader.read_exact(&mut data).await?;

        Ok(Some(PunchRecord {
            time: Duration::from_secs(sec) + Duration::from_micros(usec as u64),
            data,
        }))
    }
}

/// A writer for `.xpc` files.
pub struct PunchWriter {
    writer: BufWriter<File>,
}

impl PunchWriter {
    pub fn new(file: File) -> Self {
        Self {
            writer: BufWriter::new(file),
        }
    }

    /// Writes a record to the file.
    pub async fn write_record(&mut self, record: &PunchRecord) -> Result<()> {
        let sec = record.time.as_secs();
        let usec = record.time.subsec_micros();
        let len = record.data.len() as u32;

        self.writer.write_u64_le(sec).await?;
        self.writer.write_u32_le(usec).await?;
        self.writer.write_u32_le(len).await?;
        self.writer.write_all(&record.data).await?;

        Ok(())
    }

    pub async fn flush(&mut self) -> Result<()> {
        self.writer.flush().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use tokio::fs::File as AsyncFile;

    #[tokio::test]
    async fn test_punch_reader_writer() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_owned();

        // Write
        {
            let f = AsyncFile::create(&path).await.unwrap();
            let mut writer = PunchWriter::new(f);
            let record = PunchRecord {
                time: Duration::from_secs(1) + Duration::from_micros(500),
                data: b"/ch/01/mix/fader\0\0\0\0,f\0\0\0\0\0\0".to_vec(),
            };
            writer.write_record(&record).await.unwrap();
            writer.flush().await.unwrap();
        }

        // Read
        {
            let f = AsyncFile::open(&path).await.unwrap();
            let mut reader = PunchReader::new(f);
            let record_opt = reader.read_record().await.unwrap();
            assert!(record_opt.is_some());
            let record = record_opt.unwrap();

            assert_eq!(record.time.as_secs(), 1);
            assert_eq!(record.time.subsec_micros(), 500);
            assert_eq!(
                record.data,
                b"/ch/01/mix/fader\0\0\0\0,f\0\0\0\0\0\0".to_vec()
            );
        }
    }
}
