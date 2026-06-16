use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = x32_tapw::Args::parse();
    x32_tapw::run(args).await
}
