use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = x32_commander::Args::parse();
    x32_commander::run(args).await
}
