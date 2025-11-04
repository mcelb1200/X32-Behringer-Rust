
use anyhow::{anyhow, Result};
use clap::Parser;
use serde::Deserialize;
use std::fs;
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};
use osc_lib::{OscMessage, OscArg};
use config::{Config as ConfigParser, File, FileFormat};


mod reaper_to_x32;
mod x32_to_reaper;

/// A command-line tool for bridging X32 and REAPER.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = ".X32Reaper.ini")]
    config: String,
}

#[derive(Debug, Deserialize)]
struct Config {
    x32_ip: String,
    reaper_ip: String,
    reaper_send_port: u16,
    reaper_recv_port: u16,
    verbose: bool,
    delay_bank: u64,
    delay_generic: u64,
    transport_on: bool,
    ch_bank_on: bool,
    marker_button_on: bool,
    bank_c_color: i32,
    eq_control_on: bool,
    master_on: bool,
    bank_size: usize,
    track_min: i32,
    track_max: i32,
    aux_min: i32,
    aux_max: i32,
    fx_return_min: i32,
    fx_return_max: i32,
    bus_min: i32,
    bus_max: i32,
    dca_min: i32,
    dca_max: i32,
    track_send_offset: i32,
    reaper_dca: Vec<ReaperDca>,
    bank_up_button: i32,
    bank_down_button: i32,
    marker_button: i32,
}

#[derive(Debug, Deserialize)]
struct ReaperDca {
    min: i32,
    max: i32,
}

#[derive(Debug, Clone)]
struct Track {
    fader: f32,
    pan: f32,
    sends: [f32; 16],
    mute: bool,
    solo: bool,
    name: String,
    color: i32,
    icon: i32,
    eq: [f32; 12],
    eq_on: bool,
}

impl Default for Track {
    fn default() -> Self {
        Track {
            fader: 0.0,
            pan: 0.5,
            sends: [0.0; 16],
            mute: false,
            solo: false,
            name: String::new(),
            color: 0,
            icon: 1,
            eq: [0.0; 12],
            eq_on: false,
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    let settings = ConfigParser::builder()
        .add_source(File::new(&args.config, FileFormat::Ini))
        .build()?;
    let config: Config = settings.try_deserialize()?;

    let x32_addr: SocketAddr = format!("{}:10023", config.x32_ip).parse()?;
    let reaper_addr: SocketAddr = format!("{}:{}", config.reaper_ip, config.reaper_recv_port).parse()?;
    let reaper_listen_addr: SocketAddr = format!("0.0.0.0:{}", config.reaper_send_port).parse()?;

    let x32_socket = UdpSocket::bind("0.0.0.0:0")?;
    x32_socket.connect(x32_addr)?;
    x32_socket.set_read_timeout(Some(Duration::from_millis(1)))?;

    let reaper_socket = UdpSocket::bind(reaper_listen_addr)?;
    reaper_socket.connect(reaper_addr)?;
    reaper_socket.set_read_timeout(Some(Duration::from_millis(1)))?;

    println!("X32Reaper - v2.67 - (c)2015 Patrick-Gilles Maillot");
    println!("X32 at IP {}", config.x32_ip);
    println!("REAPER at IP {}, receives on port {}, sends to port {}", config.reaper_ip, config.reaper_recv_port, config.reaper_send_port);

    let mut xremote_time = Instant::now();
    let mut bank_offset = 0;
    let mut tracks = vec![Track::default(); config.track_max as usize];

    loop {
        if xremote_time.elapsed() > Duration::from_secs(9) {
            let xremote_cmd = OscMessage::new("/xremote".to_string(), vec![]).to_bytes().map_err(|e: String| anyhow!(e))?;
            x32_socket.send(&xremote_cmd)?;
            xremote_time = Instant::now();
        }

        let mut x32_buf = [0; 1024];
        if let Ok(len) = x32_socket.recv(&mut x32_buf) {
            let msg = OscMessage::from_bytes(&x32_buf[..len]).map_err(|e: String| anyhow!(e))?;
            if config.verbose {
                println!("X->: {} {:?}", msg.path, msg.args);
            }
            x32_to_reaper::handle_x32_message(msg, &reaper_socket, &config, &mut tracks, bank_offset)?;
        }

        let mut reaper_buf = [0; 1024];
        if let Ok(len) = reaper_socket.recv(&mut reaper_buf) {
            let msg = OscMessage::from_bytes(&reaper_buf[..len]).map_err(|e: String| anyhow!(e))?;
            if config.verbose {
                println!("R->: {} {:?}", msg.path, msg.args);
            }
            reaper_to_x32::handle_reaper_message(msg, &x32_socket, &config, &mut tracks, bank_offset)?;
        }
    }
}
