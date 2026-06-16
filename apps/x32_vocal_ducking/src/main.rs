use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = x32_vocal_ducking::Cli::parse();
    Ok(x32_vocal_ducking::run(args).await?)
}
