use anyhow::Result;
use clap::Parser;
use std::net::{SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use x32_core::Mixer;

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
    let addr: SocketAddr = format!("{}:{}", cli.ip, cli.port).parse()?;
    let socket = UdpSocket::bind(&addr)?;
    let mut mixer = Mixer::new(cli.ip);

    println!("X32 Emulator listening on {}", addr);

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    let mut buf = [0; 8192];
    socket.set_nonblocking(true)?;

    while running.load(Ordering::SeqCst) {
        if let Ok((len, remote_addr)) = socket.recv_from(&mut buf) {
            match mixer.dispatch(&buf[..len], remote_addr) {
                Ok(responses) => {
                    for (response_buf, response_addr) in responses {
                        socket.send_to(&response_buf, response_addr)?;
                    }
                }
                Err(e) => {
                    eprintln!("Error handling message: {}", e);
                }
            }
        }

        let meter_responses = mixer.process_subscriptions();
        for (response_buf, response_addr) in meter_responses {
            socket.send_to(&response_buf, response_addr)?;
        }

        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    mixer.save()?;
    println!("State saved. Shutting down.");
    Ok(())
}
