use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let args = x32_cpxlivemarkers::Cli::parse();
    Ok(x32_cpxlivemarkers::run(args)?)
}
