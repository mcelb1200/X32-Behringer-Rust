use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = x32_reaper::Args::parse();
    x32_reaper::run(args).await
}
