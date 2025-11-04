
use anyhow::{anyhow, Result};
use clap::Parser;
use std::io::{self, BufRead, Write};
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;
use osc_lib::{OscMessage, OscArg};

/// A command-line tool for getting a scene file from an X32 mixer.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The IP address of the X32 console.
    #[arg(short, long, default_value = "192.168.0.64")]
    ip: String,

    /// The name of the scene.
    #[arg(short, long)]
    name: Option<String>,

    /// Notes for the scene.
    #[arg(short, long)]
    note: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_read_timeout(Some(Duration::from_millis(100)))?;

    let x32_addr: SocketAddr = format!("{}:10023", args.ip).parse()?;
    socket.connect(x32_addr)?;

    let scene_name = match args.name {
        Some(name) => name,
        None => {
            print!("Please enter scene name: ");
            io::stdout().flush()?;
            let mut name = String::new();
            io::stdin().read_line(&mut name)?;
            name.trim().to_string()
        }
    };

    let scene_note = match args.note {
        Some(note) => note,
        None => {
            print!("Please enter note data: ");
            io::stdout().flush()?;
            let mut note = String::new();
            io::stdin().read_line(&mut note)?;
            note.trim().to_string()
        }
    };

    println!("#2.7# \"{}\" \"{}\" %000000000 1 X32GetScene V1.5 (c)2014 Patrick-Gilles Maillot\n", scene_name, scene_note);

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.starts_with('/') {
            let msg = OscMessage::new("/node".to_string(), vec![OscArg::String(line[1..].to_string())]);
            let command = msg.to_bytes().map_err(|e: String| anyhow!(e))?;
            socket.send(&command)?;
            let mut buf = [0; 512];
            if let Ok(len) = socket.recv(&mut buf) {
                let response = OscMessage::from_bytes(&buf[..len]).map_err(|e: String| anyhow!(e))?;
                println!("{}", response.path);
            }
        } else if line == "exit" {
            break;
        }
    }

    Ok(())
}
