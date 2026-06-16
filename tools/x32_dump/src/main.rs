use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let args = x32_dump::Args::parse();
    x32_dump::run(args)
}
