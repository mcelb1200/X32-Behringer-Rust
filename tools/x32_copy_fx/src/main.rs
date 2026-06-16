use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = x32_copy_fx::Args::parse();
    Ok(x32_copy_fx::run(args).await?)
}
