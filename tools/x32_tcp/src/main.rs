use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = x32_tcp::Args::parse();
    Ok(x32_tcp::run(args).await?)
}
