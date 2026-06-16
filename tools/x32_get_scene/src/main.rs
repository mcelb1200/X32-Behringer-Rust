use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = x32_get_scene::Args::parse();
    Ok(x32_get_scene::run(args).await?)
}
