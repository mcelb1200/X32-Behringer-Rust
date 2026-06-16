use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = x32_get_lib::Args::parse();
    x32_get_lib::run(args).await
}
