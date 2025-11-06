use anyhow::Result;
use clap::Parser;
use x32_emulator::run;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// IP address to bind to
    #[arg(short, long, default_value_t = String::from("0.0.0.0"))]
    ip: String,

    /// Port number to bind to
    #[arg(short, long, default_value_t = 10023)]
    port: u16,
}

use x32_core::Mixer;
fn main() -> Result<()> {
    let cli = Cli::parse();
    let mixer = Mixer::new();
    let addr = run(mixer, cli.ip, cli.port)?;
    println!("X32 Emulator listening on {}", addr);

    // Keep the main thread alive
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
