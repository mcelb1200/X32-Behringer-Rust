use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = x32_emulator::Cli::parse();
    x32_emulator::run(cli)
}
