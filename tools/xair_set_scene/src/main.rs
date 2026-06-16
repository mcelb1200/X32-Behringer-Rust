use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = xair_set_scene::Args::parse();
    xair_set_scene::run(args).await
}
