use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = x32_desk_save::Args::parse();
    Ok(x32_desk_save::run(args).await?)
}
