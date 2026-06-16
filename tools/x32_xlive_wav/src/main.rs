use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let args = x32_xlive_wav::Args::parse();
    x32_xlive_wav::run(args)
}
