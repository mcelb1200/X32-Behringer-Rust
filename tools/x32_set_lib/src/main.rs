use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = x32_set_lib::Args::parse();
    Ok(x32_set_lib::run(args).await?)
}
