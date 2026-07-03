#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = <x32_dca_spill::Args as clap::Parser>::parse();
    x32_dca_spill::run(args).await
}
