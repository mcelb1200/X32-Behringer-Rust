use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = x32_emulator::Cli::parse();
    Ok(x32_emulator::run(cli)?)
}
