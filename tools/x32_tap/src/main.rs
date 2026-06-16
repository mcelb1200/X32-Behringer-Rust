use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = x32_tap::Args::parse();
    Ok(x32_tap::run(args).await?)
}
