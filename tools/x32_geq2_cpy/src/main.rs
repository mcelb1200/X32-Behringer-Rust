use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = x32_geq2_cpy::Args::parse();
    x32_geq2_cpy::run(args).await
}
