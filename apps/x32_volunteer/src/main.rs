use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = x32_volunteer::Args::parse();
    x32_volunteer::run(args).await
}
