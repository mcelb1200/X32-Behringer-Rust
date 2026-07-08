use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = x32_crossfade::Args::parse();
    x32_crossfade::run(args).await
}
