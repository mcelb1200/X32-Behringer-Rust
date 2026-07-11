use clap::Parser;
use x32_safe_mute::{Args, run};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    run(args).await
}
