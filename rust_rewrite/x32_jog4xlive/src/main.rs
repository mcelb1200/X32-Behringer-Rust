
use clap::Parser;
use anyhow::Result;
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use osc_lib::{OscMessage, OscArg};
use std::time::Duration;

/// A command-line tool to provide jog wheel functionality for the Behringer X32's X-Live! card.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// IP address of the X32 console
    #[arg(long)]
    ip: String,
}

enum AppState {
    Listening,
    WaitingForEtime,
}

/// The main application loop.
/// Connects to the X32, sets up the user controls, and then listens for OSC messages from the jog wheel encoders.
async fn run(args: Args) -> Result<()> {
    let remote_addr: SocketAddr = format!("{}:10023", args.ip).parse()?;
    let local_addr: SocketAddr = "0.0.0.0:0".parse()?;
    let socket = UdpSocket::bind(local_addr).await?;
    socket.connect(remote_addr).await?;

    // Set up user controls
    let messages = vec![
        // Set Bank C Encoder 1 to its default value: 64
        OscMessage::new("/config/userctrl/C/enc/1".to_string(), vec![OscArg::String("MP13000".to_string())]),
        OscMessage::new("/-stat/userpar/33/value".to_string(), vec![OscArg::Int(64)]),
        // Set X32 Bank C Encoder 3 to its default value: 0
        OscMessage::new("/config/userctrl/C/enc/3".to_string(), vec![OscArg::String("MP14000".to_string())]),
        OscMessage::new("/-stat/userpar/35/value".to_string(), vec![OscArg::Int(0)]),
        // Select X32 Bank C
        OscMessage::new("/-stat/userbank".to_string(), vec![OscArg::Int(2)]),
    ];

    for msg in messages {
        socket.send(&msg.to_bytes()?).await?;
    }

    println!("X32Jog4Xlive is running. Press Ctrl+C to exit.");

    let mut delta_time = 1;
    let mut buf = [0; 512];
    let mut state = AppState::Listening;

    loop {
        match state {
            AppState::Listening => {
                let len = socket.recv(&mut buf).await?;
                let msg = OscMessage::from_bytes(&buf[..len])?;
                match msg.path.as_str() {
                    "/-stat/userpar/33/value" => { // Encoder 1
                        socket.send(&OscMessage::new("/-stat/urec/etime".to_string(), vec![]).to_bytes()?).await?;
                        state = AppState::WaitingForEtime;
                    },
                    "/-stat/userpar/35/value" => { // Encoder 3
                        if let Some(OscArg::Int(move_val)) = msg.args.get(0) {
                            delta_time = (move_val * move_val) + 1;
                        }
                    },
                    _ => {}
                }
            },
            AppState::WaitingForEtime => {
                match tokio::time::timeout(Duration::from_millis(100), socket.recv(&mut buf)).await {
                    Ok(Ok(len)) => {
                        let msg = OscMessage::from_bytes(&buf[..len])?;
                        if msg.path == "/-stat/urec/etime" {
                            if let Some(OscArg::Int(etime)) = msg.args.get(0) {
                                let new_time = etime + delta_time;
                                socket.send(&OscMessage::new("/-action/setposition".to_string(), vec![OscArg::Int(new_time)]).to_bytes()?).await?;
                                socket.send(&OscMessage::new("/-stat/userpar/33/value".to_string(), vec![OscArg::Int(64)]).to_bytes()?).await?;
                            }
                        }
                        state = AppState::Listening;
                    },
                    _ => {
                        // Timeout or error, go back to listening
                        state = AppState::Listening;
                    }
                }
            }
        }
    }
}

/// The entry point of the application.
/// Parses command-line arguments and calls the `run` function.
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    if let Err(e) = run(args).await {
        eprintln!("Error: {}", e);
    }
    Ok(())
}
