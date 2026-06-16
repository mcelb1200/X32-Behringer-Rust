use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = x32_midi2osc::Args::parse();
    x32_midi2osc::run(args).await
}
