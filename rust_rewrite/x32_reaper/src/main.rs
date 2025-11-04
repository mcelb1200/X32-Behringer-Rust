
use anyhow::{anyhow, Result};
use clap::Parser;
use serde::Deserialize;
use std::fs;
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};
use x32_lib::{cparse, dump};

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
    let config_str = fs::read_to_string(args.config)?;
    let config: Config = parse_config(&config_str)?;

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
            let xremote_cmd = cparse::xcparse("/xremote").map_err(|e| anyhow!(e))?;
            x32_socket.send(&xremote_cmd)?;
            xremote_time = Instant::now();
        }

        let mut x32_buf = [0; 1024];
        if let Ok(len) = x32_socket.recv(&mut x32_buf) {
            if config.verbose {
                println!("{}", dump::xfdump("X->", &x32_buf[..len], false));
            }
            x32_to_reaper::handle_x32_message(&x32_buf[..len], &reaper_socket, &config, &mut tracks, bank_offset)?;
        }

        let mut reaper_buf = [0; 1024];
        if let Ok(len) = reaper_socket.recv(&mut reaper_buf) {
            if config.verbose {
                println!("{}", dump::xfdump("R->", &reaper_buf[..len], false));
            }
            reaper_to_x32::handle_reaper_message(&reaper_buf[..len], &x32_socket, &config, &mut tracks, bank_offset)?;
        }
    }
}

fn parse_config(config_str: &str) -> Result<Config> {
    let mut lines = config_str.lines();

    let flags1_line = lines.next().ok_or_else(|| anyhow!("Config file is empty"))?;
    let flags1: Vec<&str> = flags1_line.split_whitespace().collect();
    if flags1.len() < 7 { return Err(anyhow!("Invalid first line in config")); }
    let verbose = flags1[2].parse::<i32>()? == 1;
    let delay_bank = flags1[3].parse::<u64>()?;
    let delay_generic = flags1[4].parse::<u64>()?;

    let x32_ip = lines.next().ok_or_else(|| anyhow!("Missing X32 IP"))?.trim().to_string();
    let reaper_ip = lines.next().ok_or_else(|| anyhow!("Missing Reaper IP"))?.trim().to_string();
    let reaper_send_port = lines.next().ok_or_else(|| anyhow!("Missing Reaper send port"))?.trim().parse()?;
    let reaper_recv_port = lines.next().ok_or_else(|| anyhow!("Missing Reaper receive port"))?.trim().parse()?;

    let flags2_line = lines.next().ok_or_else(|| anyhow!("Missing flags line"))?;
    let flags2: Vec<&str> = flags2_line.split_whitespace().collect();
    if flags2.len() < 6 { return Err(anyhow!("Invalid flags line")); }
    let transport_on = flags2[0].parse::<i32>()? == 1;
    let ch_bank_on = flags2[1].parse::<i32>()? == 1;
    let marker_button_on = flags2[2].parse::<i32>()? == 1;
    let bank_c_color = flags2[3].parse::<i32>()?;
    let eq_control_on = flags2[4].parse::<i32>()? == 1;
    let master_on = flags2[5].parse::<i32>()? == 1;

    let map_line = lines.next().ok_or_else(|| anyhow!("Missing map line"))?;
    let map: Vec<&str> = map_line.split_whitespace().collect();
    if map.len() < 11 { return Err(anyhow!("Invalid map line")); }
    let track_min = map[0].parse::<i32>()?;
    let track_max = map[1].parse::<i32>()?;
    let aux_min = map[2].parse::<i32>()?;
    let aux_max = map[3].parse::<i32>()?;
    let fx_return_min = map[4].parse::<i32>()?;
    let fx_return_max = map[5].parse::<i32>()?;
    let bus_min = map[6].parse::<i32>()?;
    let bus_max = map[7].parse::<i32>()?;
    let dca_min = map[8].parse::<i32>()?;
    let dca_max = map[9].parse::<i32>()?;
    let track_send_offset = map[10].parse::<i32>()?;

    let mut reaper_dca = Vec::new();
    for i in 0..8 {
        let dca_line = lines.next().ok_or_else(|| anyhow!(format!("Missing DCA map for DCA {}", i+1)))?;
        let dca_map: Vec<&str> = dca_line.split_whitespace().collect();
        if dca_map.len() < 2 { return Err(anyhow!(format!("Invalid DCA map for DCA {}: expected 2 values", i+1))); }
        reaper_dca.push(ReaperDca { min: dca_map[0].parse()?, max: dca_map[1].parse()? });
    }

    let buttons_line = lines.next().ok_or_else(|| anyhow!("Missing buttons line"))?;
    let buttons: Vec<&str> = buttons_line.split_whitespace().collect();
    if buttons.len() < 5 { return Err(anyhow!("Invalid buttons line")); }
    let bank_up_button = buttons[0].parse::<i32>()?;
    let bank_down_button = buttons[1].parse::<i32>()?;
    let marker_button = buttons[2].parse::<i32>()?;
    let bank_size = buttons[4].parse::<usize>()?;


    Ok(Config {
        x32_ip,
        reaper_ip,
        reaper_send_port,
        reaper_recv_port,
        verbose,
        delay_bank,
        delay_generic,
        transport_on,
        ch_bank_on,
        marker_button_on,
        bank_c_color,
        eq_control_on,
        master_on,
        bank_size,
        track_min,
        track_max,
        aux_min,
        aux_max,
        fx_return_min,
        fx_return_max,
        bus_min,
        bus_max,
        dca_min,
        dca_max,
        track_send_offset,
        reaper_dca,
        bank_up_button,
        bank_down_button,
        marker_button,
    })
}
