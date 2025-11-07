
use anyhow::Result;
use clap::Parser;
use x32_emulator::X32Emulator;

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

fn main() -> Result<()> {
    let _cli = Cli::parse();
    let mut emulator = X32Emulator::new();
    emulator.start();
    println!("X32 Emulator listening on {}", emulator.local_addr());


    // Keep the main thread alive
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
