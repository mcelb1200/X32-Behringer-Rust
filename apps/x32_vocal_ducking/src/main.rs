//! x32_vocal_ducking standalone binary

use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = x32_vocal_ducking::Cli::parse();
    x32_vocal_ducking::run(args).await
}
