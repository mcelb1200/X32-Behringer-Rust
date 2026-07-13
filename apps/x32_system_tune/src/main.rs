use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = x32_system_tune::Args::parse();
    x32_system_tune::run(args).await
}
