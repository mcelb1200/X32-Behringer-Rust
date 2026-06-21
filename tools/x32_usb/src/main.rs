use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = x32_usb::Args::parse();

    if let Err(e) = x32_usb::run(args).await {
        if e.to_string().contains("timeout") || e.to_string().contains("connect") {
            println!("Not connected to X32.");
        } else {
            eprintln!("Error: {}", e);
        }
        std::process::exit(1);
    }
    Ok(())
}
