use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = x32_get_scene_name::Args::parse();
    Ok(x32_get_scene_name::run(args).await?)
}
