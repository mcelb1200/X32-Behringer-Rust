use clap::Parser;
use x32_speech_mode::{run, Args};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    run(args).await
}
