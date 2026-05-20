import re

with open("x32_wav_xlive/src/main.rs", "r") as f:
    text = f.read()

# Fix in x32_wav_xlive
old_wav = """    let mut markers = args.markers.clone();
    if let Some(marker_file) = &args.marker_file {
        let f = File::open(marker_file)?;
        let mut s = String::new();
        std::io::Read::take(f, 1024 * 1024).read_to_string(&mut s)?;"""

new_wav = """    let mut markers = args.markers.clone();
    if let Some(marker_file) = &args.marker_file {
        let f = File::open(marker_file)?;

        // Sentinel: Prevent OOM from maliciously large or corrupted marker files
        if f.metadata()?.len() > 1024 * 1024 {
            return Err(anyhow::anyhow!("Marker file too large to load (max 1MB)"));
        }

        let mut s = String::new();
        std::io::Read::take(f, 1024 * 1024).read_to_string(&mut s)?;"""

text = text.replace(old_wav, new_wav)

# Also check for panic conditions due to integer underflow
old_wav2 = """    let zero_buf = vec![0u8; 4 * (256 - take_sizes.len())];"""
new_wav2 = """    if take_sizes.len() > 256 {
        return Err(anyhow::anyhow!("Too many takes (max 256)"));
    }
    let zero_buf = vec![0u8; 4 * (256 - take_sizes.len())];"""
text = text.replace(old_wav2, new_wav2)

old_wav3 = """    let zero_buf = vec![0u8; 4 * (125 - markers.len())];"""
new_wav3 = """    if markers.len() > 125 {
        return Err(anyhow::anyhow!("Too many markers (max 125)"));
    }
    let zero_buf = vec![0u8; 4 * (125 - markers.len())];"""
text = text.replace(old_wav3, new_wav3)

old_wav4 = """        let file = File::open(log_path).unwrap();
        let mut buffer = Vec::new();
        std::io::Read::take(file, 2048 * 2)
            .read_to_end(&mut buffer)
            .unwrap();"""
new_wav4 = """        let file = File::open(log_path).unwrap();

        if file.metadata().unwrap().len() > 1024 * 1024 {
            panic!("SE_LOG.BIN file too large");
        }

        let mut buffer = Vec::new();
        std::io::Read::take(file, 2048 * 2)
            .read_to_end(&mut buffer)
            .unwrap();"""
text = text.replace(old_wav4, new_wav4)

with open("x32_wav_xlive/src/main.rs", "w") as f:
    f.write(text)

with open("x32_fade/src/main.rs", "r") as f:
    text2 = f.read()

old_fade = """    // Load configuration from a file if specified.
    if let Some(path) = &args.load_config {
        let f = std::fs::File::open(path)?;
        let mut data = String::new();
        f.take(1024 * 1024).read_to_string(&mut data)?;"""

new_fade = """    // Load configuration from a file if specified.
    if let Some(path) = &args.load_config {
        let f = std::fs::File::open(path)?;

        // Sentinel: Prevent OOM from maliciously large or corrupted config files
        if f.metadata()?.len() > 1024 * 1024 {
            return Err(anyhow::anyhow!("Config file too large to load (max 1MB)"));
        }

        let mut data = String::new();
        f.take(1024 * 1024).read_to_string(&mut data)?;"""

text2 = text2.replace(old_fade, new_fade)
with open("x32_fade/src/main.rs", "w") as f:
    f.write(text2)


with open("x32_wav_xlive/tests/integration_test.rs", "r") as f:
    text3 = f.read()

old_test = """    let mut file = File::open(log_path).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();"""
new_test = """    let mut file = File::open(log_path).unwrap();
    if file.metadata().unwrap().len() > 1024 * 1024 {
        panic!("SE_LOG.BIN file too large");
    }
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();"""
text3 = text3.replace(old_test, new_test)

with open("x32_wav_xlive/tests/integration_test.rs", "w") as f:
    f.write(text3)
