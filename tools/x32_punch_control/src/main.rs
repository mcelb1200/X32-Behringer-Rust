use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = x32_punch_control::Args::parse();
    Ok(x32_punch_control::run(args).await?)
}
