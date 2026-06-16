use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = x32_replay::Args::parse();
    x32_replay::run(args).await
}
