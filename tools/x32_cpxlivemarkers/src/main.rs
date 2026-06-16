use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let args = x32_cpxlivemarkers::Cli::parse();
    x32_cpxlivemarkers::run(args)
}
