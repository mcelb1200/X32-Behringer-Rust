use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = x32_usb::Args::parse();

    if let Err(e) = x32_usb::run(args).await {
        let err_str = e.to_string().to_lowercase();
        if err_str.contains("timeout") || err_str.contains("connect") {
            println!("Not connected to X32.");
        } else {
            eprintln!("Error: {}", e);
        }
        std::process::exit(1);
    }
    Ok(())
}
