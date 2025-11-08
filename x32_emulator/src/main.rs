use anyhow::Result;
use clap::Parser;
use x32_emulator::server;

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
    let cli = Cli::parse();
    let bind_addr = format!("{}:{}", cli.ip, cli.port);
    server::run(&bind_addr, None, None)
}
