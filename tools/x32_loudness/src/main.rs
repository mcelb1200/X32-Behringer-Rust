use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = x32_loudness::Cli::parse();
    Ok(x32_loudness::run(args).await?)
}
