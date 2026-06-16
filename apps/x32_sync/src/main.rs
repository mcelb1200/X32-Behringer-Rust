use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = x32_sync::Args::parse();
    x32_sync::run(args).await
}
