use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = x32_udp::Args::parse();
    x32_udp::run(args).await
}
