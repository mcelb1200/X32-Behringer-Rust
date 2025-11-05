
use clap::Parser;
use std::error::Error;
use std::io::{self, BufRead};
use std::net::UdpSocket;
use std::time::Instant;
use osc_lib::{OscMessage, OscArg};
use x32_lib::create_socket;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The IP address of the X32 console.
    #[arg(short, long, default_value = "192.168.0.64")]
    ip: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let socket = create_socket(&args.ip, 100)?;

    println!("X32Tap - v1.21 - (c)2015 Patrick-Gilles Maillot");
    println!("\n '1'...'4' <cr> to select FX slot with DLY,");
    println!(" 'q' <cr> to exit,");
    println!(" <cr> to set tempo\n");

    let mut fx_slot = 0;
    let mut last_tap = Instant::now();
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    loop {
        let mut line = String::new();
        handle.read_line(&mut line)?;
        let input = line.trim();

        if input.is_empty() {
            if fx_slot != 0 {
                let now = Instant::now();
                let duration = now.duration_since(last_tap);
                last_tap = now;

                let tempo_ms = duration.as_millis();
                let mut tempo_f = tempo_ms as f32 / 3000.0;
                if tempo_f < 0.0 { tempo_f = 0.0; }
                if tempo_f > 1.0 { tempo_f = 1.0; }

                set_tempo(&socket, fx_slot, tempo_f)?;
                println!("Tempo: {}ms", (tempo_f * 3000.0) as u32);
            }
        } else {
            match input {
                "1" | "2" | "3" | "4" => {
                    let slot: u8 = input.parse()?;
                    if verify_fx_slot(&socket, slot)? {
                        fx_slot = slot;
                        println!("FX Slot {} selected.", fx_slot);
                    } else {
                        println!("No DLY effect at FX slot #{}!", slot);
                    }
                }
                "q" | "Q" => break,
                _ => (),
            }
        }
    }
    Ok(())
}

fn verify_fx_slot(socket: &UdpSocket, slot: u8) -> Result<bool, Box<dyn Error>> {
    let msg = OscMessage::new(format!("/fx/{}/type", slot), vec![]);
    socket.send(&msg.to_bytes()?)?;
    let mut buf = [0; 512];
    if let Ok(len) = socket.recv(&mut buf) {
        let response = OscMessage::from_bytes(&buf[..len])?;
        if let Some(OscArg::Int(fx_type)) = response.args.get(0) {
            if *fx_type == 10 { // Stereo Delay
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn set_tempo(socket: &UdpSocket, slot: u8, value: f32) -> Result<(), Box<dyn Error>> {
    let msg = OscMessage::new(format!("/fx/{}/par/02", slot), vec![OscArg::Float(value)]);
    socket.send(&msg.to_bytes()?)?;
    Ok(())
}
