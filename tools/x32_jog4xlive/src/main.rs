use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = x32_jog4xlive::Args::parse();
    x32_jog4xlive::run(args).await
}
