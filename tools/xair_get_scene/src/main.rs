use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = xair_get_scene::Args::parse();
    xair_get_scene::run(args).await
}
