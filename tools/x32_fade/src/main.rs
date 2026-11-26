//! CLI entry point for `x32_fade`.

use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = x32_fade::Args::parse();
    x32_fade::run(args).await
}
