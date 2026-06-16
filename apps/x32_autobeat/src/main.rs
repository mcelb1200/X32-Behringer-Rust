use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = x32_autobeat::Cli::parse();
    x32_autobeat::run(args).await
}
