//! Consolidated X32 Web UI binary entry point.

use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = x32_ui::Args::parse();
    x32_ui::run(args).await
}
