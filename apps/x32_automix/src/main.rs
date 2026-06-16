use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = x32_automix::Args::parse();
    Ok(x32_automix::run(args).await?)
}
