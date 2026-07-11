use anyhow::Result;
use clap::Parser;
use x32_usb::{run, Args};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    if let Err(e) = run(args).await {
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
