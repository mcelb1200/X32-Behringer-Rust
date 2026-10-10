//! Main entry point for the standalone `x32_safe_mute` binary wrapper.
use clap::Parser;
use x32_safe_mute::{run, Args};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    run(args).await
}
