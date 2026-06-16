use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let args = x32_wav_xlive::Args::parse();
    x32_wav_xlive::run(args)
}
