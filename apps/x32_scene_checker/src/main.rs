use clap::Parser;
use x32_scene_checker::{Args, run};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    run(args).await
}
