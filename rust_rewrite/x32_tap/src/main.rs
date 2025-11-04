
use anyhow::{anyhow, Result};
use clap::Parser;
use crossterm::{event::{self, Event, KeyCode}, terminal};
use std::io::{self, Write};
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};
use osc_lib::{OscMessage, OscArg};

/// A command-line tool for setting the tap tempo of a delay effect on an X32 mixer.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The IP address of the X32 console.
    #[arg(short, long, default_value = "192.168.0.64")]
    ip: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    write!(stdout, "X32Tap - v1.21 - (c)2015 Patrick-Gilles Maillot\n\n")?;
    write!(stdout, " '1'...'4' <cr> to select FX slot with DLY,\n")?;
    write!(stdout, " 'q' <cr> to exit,\n")?;
    write!(stdout, " <cr> to set tempo\n")?;
    stdout.flush()?;

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_read_timeout(Some(Duration::from_millis(500)))?;

    let x32_addr: SocketAddr = format!("{}:10023", args.ip).parse()?;
    socket.connect(x32_addr)?;

    write!(stdout, "Connecting to X32.")?;
    stdout.flush()?;

    let info_cmd = OscMessage::new("/info".to_string(), vec![]).to_bytes().map_err(|e: String| anyhow!(e))?;
    loop {
        socket.send(&info_cmd)?;
        let mut buf = [0; 512];
        if let Ok(len) = socket.recv(&mut buf) {
            let msg = OscMessage::from_bytes(&buf[..len]).map_err(|e: String| anyhow!(e))?;
            if msg.path == "/info" {
                break;
            }
        }
        write!(stdout, ".")?;
        stdout.flush()?;
    }

    write!(stdout, " Done!\n")?;
    stdout.flush()?;

    let mut last_tap = Instant::now();
    let mut fx_slot = 0;

    loop {
        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Enter => {
                        if fx_slot != 0 {
                            let now = Instant::now();
                            let delta = now.duration_since(last_tap).as_millis();
                            last_tap = now;

                            if delta > 0 && delta < 3000 {
                                let tempo = delta as f32 / 3000.0;
                                let msg = OscMessage::new(
                                    format!("/fx/{}/par/02", fx_slot),
                                    vec![OscArg::Float(tempo)]
                                );
                                let command = msg.to_bytes().map_err(|e: String| anyhow!(e))?;
                                socket.send(&command)?;
                                write!(stdout, "Tempo: {}ms\n", delta)?;
                                stdout.flush()?;
                            }
                        }
                    }
                    KeyCode::Char(c) => {
                        match c {
                            '1'..='4' => {
                                let slot = c.to_digit(10).unwrap();
                                let command = OscMessage::new(format!("/fx/{}/type", slot), vec![]).to_bytes().map_err(|e: String| anyhow!(e))?;
                                socket.send(&command)?;
                                let mut buf = [0; 512];
                                if let Ok(len) = socket.recv(&mut buf) {
                                    let msg = OscMessage::from_bytes(&buf[..len]).map_err(|e: String| anyhow!(e))?;
                                    if msg.path == format!("/fx/{}/type", slot) {
                                        if let Some(OscArg::Int(fx_type)) = msg.args.get(0) {
                                            if *fx_type == 10 { // Stereo Delay
                                                fx_slot = slot;
                                                write!(stdout, "Found FX!\n")?;
                                            } else {
                                                write!(stdout, "No DLY effect at FX#{}\n", slot)?;
                                            }
                                            stdout.flush()?;
                                        }
                                    }
                                }
                            }
                            'q' | 'Q' => break,
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    terminal::disable_raw_mode()?;
    Ok(())
}
