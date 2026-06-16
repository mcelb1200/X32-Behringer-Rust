use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = x32_custom_layer::Cli::parse();
    x32_custom_layer::run(cli).await
}
