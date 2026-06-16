use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = x32_ssavergw::Args::parse();
    Ok(x32_ssavergw::run(args).await?)
}
