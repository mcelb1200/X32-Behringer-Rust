use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = xair_command::Args::parse();
    xair_command::run(args).await
}
