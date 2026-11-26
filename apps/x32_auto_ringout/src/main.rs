//! CLI entry point for `x32_auto_ringout`.

use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = x32_auto_ringout::Args::parse();
    x32_auto_ringout::run(args).await
}
