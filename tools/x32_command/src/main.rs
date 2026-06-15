use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = x32_command::Args::parse();
    x32_command::run(args).await
}
