use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = x32_auto_gain::Args::parse();
    x32_auto_gain::run(args).await
}
