//! `x32_reaper` is a bridge application that connects a Behringer X32/M32 digital mixer
//! to the Reaper Digital Audio Workstation (DAW) via OSC.
//!
//! It enables bidirectional control and synchronization between the two systems, allowing
//! the X32 to act as a control surface for Reaper, and Reaper to automate the X32's parameters.
//! Features include fader/pan sync, mute sync, transport control, and bank switching.

use anyhow::{Context, Result};
use clap::Parser;
use osc_lib::{OscArg, OscMessage};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tokio::time::{self, Duration};

mod config;
mod state;

use config::Config;
use state::{AppState, ChannelState};

/// Command-line arguments for `x32_reaper`.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to config file (default: .X32Reaper.ini)
    #[arg(long, default_value = ".X32Reaper.ini")]
    config: String,
}

// Flags
const TRACKPAN: i32 = 0x0001;
const TRACKFADER: i32 = 0x0002;
const TRACKNAME: i32 = 0x0004;
const TRACKMUTE: i32 = 0x0008;
const TRACKSELECT: i32 = 0x0010;
const TRACKSEND: i32 = 0x0020;
const TRACKSOLO: i32 = 0x0040;
const TRACKFX: i32 = 0x0080;
const MASTERPAN: i32 = 0x0100;
const MASTERVOLUME: i32 = 0x0200;

const X32PAN: i32 = 0x0001;
const X32FADER: i32 = 0x0002;
const X32NAME: i32 = 0x0004;
const X32MUTE: i32 = 0x0008;
const X32SELECT: i32 = 0x0010;
const X32SEND: i32 = 0x0020;
const X32SOLO: i32 = 0x0040;
const X32FX: i32 = 0x0080;
const X32MPAN: i32 = 0x0100;
const X32MFADER: i32 = 0x0200;

/// The main entry point for the application.
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!("X32Reaper - Rust Rewrite");

    let config = match Config::load(&args.config) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to load config file '{}': {}", args.config, e);
            std::process::exit(1);
        }
    };

    println!("X32 at IP {}", config.x32_ip);
    println!(
        "REAPER at IP {}\nreceives on port {}\nsends to port {}",
        config.reaper_ip, config.reaper_recv_port, config.reaper_send_port
    );

    let state = Arc::new(Mutex::new(AppState::new(&config)));

    let x32_sock = UdpSocket::bind("0.0.0.0:0").await?;
    let reaper_bind_addr = format!("0.0.0.0:{}", config.reaper_recv_port);
    let reaper_sock = UdpSocket::bind(&reaper_bind_addr)
        .await
        .context("Failed to bind Reaper socket")?;

    let x32_addr: SocketAddr = format!("{}:10023", config.x32_ip)
        .parse()
        .context("Invalid X32 IP")?;
    let reaper_addr: SocketAddr = format!("{}:{}", config.reaper_ip, config.reaper_send_port)
        .parse()
        .context("Invalid Reaper IP")?;

    connect_x32(&x32_sock, x32_addr).await?;

    let mut buf_x32 = [0u8; 4096];
    let mut buf_reaper = [0u8; 4096];
    let mut interval_timer = time::interval(Duration::from_secs(9));

    println!("Starting main loop...");

    init_user_ctrl(
        &x32_sock,
        x32_addr,
        &config,
        &mut *state.lock().await,
        Some((&reaper_sock, reaper_addr)),
    )
    .await?;

    loop {
        tokio::select! {
            _ = interval_timer.tick() => {
                 let _ = x32_sock.send_to(b"/xremote", x32_addr).await;
            }
            Ok((len, _addr)) = x32_sock.recv_from(&mut buf_x32) => {
                let data = &buf_x32[..len];
                if let Err(e) = process_x32_message(data, &config, &state, &reaper_sock, reaper_addr, &x32_sock, x32_addr).await {
                    if config.verbose { eprintln!("Error processing X32 message: {}", e); }
                }
            }
            Ok((len, _addr)) = reaper_sock.recv_from(&mut buf_reaper) => {
                let data = &buf_reaper[..len];
                if let Err(e) = process_reaper_message(data, &config, &state, &x32_sock, x32_addr, &reaper_sock, reaper_addr).await {
                    if config.verbose { eprintln!("Error processing Reaper message: {}", e); }
                }
            }
        }
    }
}

/// Sends an OSC message to the X32, optionally with a delay.
async fn send_to_x(sock: &UdpSocket, addr: SocketAddr, msg: &OscMessage, delay: u64) -> Result<()> {
    let bytes = msg
        .to_bytes()
        .map_err(|e| anyhow::anyhow!("OSC error: {:?}", e))?;
    sock.send_to(&bytes, addr).await?;
    if delay > 0 {
        tokio::time::sleep(Duration::from_millis(delay)).await;
    }
    Ok(())
}

/// Sends an OSC message to Reaper.
async fn send_to_r(sock: &UdpSocket, addr: SocketAddr, msg: &OscMessage) -> Result<()> {
    let bytes = msg
        .to_bytes()
        .map_err(|e| anyhow::anyhow!("OSC error: {:?}", e))?;
    sock.send_to(&bytes, addr).await?;
    Ok(())
}

/// Establishes connection with the X32 console.
async fn connect_x32(sock: &UdpSocket, addr: SocketAddr) -> Result<()> {
    sock.send_to(b"/info", addr).await?;
    let mut buf = [0u8; 1024];
    let res = time::timeout(Duration::from_millis(1000), sock.recv_from(&mut buf)).await;
    match res {
        Ok(Ok((len, _src))) => {
            let s = String::from_utf8_lossy(&buf[..len]);
            if !s.starts_with("/info") {
                eprintln!("Unexpected response from X32: {}", s);
                return Err(anyhow::anyhow!("Unexpected response from X32"));
            }
            println!("X32 Connected!");
        }
        _ => {
            return Err(anyhow::anyhow!("X32 connection timeout"));
        }
    }
    Ok(())
}

/// Initializes user controls and updates bank settings.
async fn init_user_ctrl(
    sock: &UdpSocket,
    addr: SocketAddr,
    config: &Config,
    state: &mut AppState,
    reaper_info: Option<(&UdpSocket, SocketAddr)>,
) -> Result<()> {
    let mp = ["MP13000", "MP14000", "MP15000", "MP16000"];
    let mn = [
        "MN16000", "MN16001", "MN16002", "MN16003", "MN16004", "MN16005", "MN16006", "MN16007",
    ];

    if config.transport_on {
        for i in 1..=4 {
            let msg = OscMessage {
                path: format!("/config/userctrl/C/enc/{}", i),
                args: vec![OscArg::String(mp[i - 1].to_string())],
            };
            send_to_x(sock, addr, &msg, config.delay_generic).await?;
        }
        for i in 5..=12 {
            let msg = OscMessage {
                path: format!("/config/userctrl/C/btn/{}", i),
                args: vec![OscArg::String(mn[i - 5].to_string())],
            };
            send_to_x(sock, addr, &msg, config.delay_generic).await?;
        }
        for i in 33..=36 {
            let msg = OscMessage {
                path: format!("/-stat/userpar/{:02}/value", i),
                args: vec![OscArg::Int(64)],
            };
            send_to_x(sock, addr, &msg, config.delay_generic).await?;
        }
        for i in 17..=24 {
            let msg = OscMessage {
                path: format!("/-stat/userpar/{:02}/value", i),
                args: vec![OscArg::Int(0)],
            };
            send_to_x(sock, addr, &msg, config.delay_generic).await?;
        }
    } else {
        if config.marker_btn_on {
            let btn_idx = config.marker_btn;
            if btn_idx >= 5 && btn_idx <= 12 {
                let msg = OscMessage {
                    path: format!("/config/userctrl/C/btn/{}", btn_idx),
                    args: vec![OscArg::String(mn[btn_idx as usize - 5].to_string())],
                };
                send_to_x(sock, addr, &msg, config.delay_generic).await?;
                let msg2 = OscMessage {
                    path: format!("/-stat/userpar/{:02}/value", 12 + btn_idx),
                    args: vec![OscArg::Int(0)],
                };
                send_to_x(sock, addr, &msg2, config.delay_generic).await?;
            }
        }
        if config.ch_bank_on {
            for &btn_idx in &[config.bank_up, config.bank_dn] {
                if btn_idx >= 5 && btn_idx <= 12 {
                    let msg = OscMessage {
                        path: format!("/config/userctrl/C/btn/{}", btn_idx),
                        args: vec![OscArg::String(mn[btn_idx as usize - 5].to_string())],
                    };
                    send_to_x(sock, addr, &msg, config.delay_generic).await?;
                    let msg2 = OscMessage {
                        path: format!("/-stat/userpar/{:02}/value", 12 + btn_idx),
                        args: vec![OscArg::Int(0)],
                    };
                    send_to_x(sock, addr, &msg2, config.delay_generic).await?;
                }
            }
        }
    }

    if config.transport_on || config.marker_btn_on || config.ch_bank_on {
        let msg = OscMessage {
            path: "/config/userctrl/C/color".to_string(),
            args: vec![OscArg::Int(config.bank_c_color)],
        };
        send_to_x(sock, addr, &msg, config.delay_generic).await?;

        let msg2 = OscMessage {
            path: "/-stat/userbank".to_string(),
            args: vec![OscArg::Int(2)],
        };
        send_to_x(sock, addr, &msg2, config.delay_generic).await?;
    }

    if config.ch_bank_on {
        update_bk_ch(sock, addr, config, state, reaper_info).await?;
    }

    Ok(())
}

/// Updates the X32 channel strips to match the current bank's tracks from Reaper.
async fn update_bk_ch(
    sock: &UdpSocket,
    addr: SocketAddr,
    config: &Config,
    state: &AppState,
    reaper_info: Option<(&UdpSocket, SocketAddr)>,
) -> Result<()> {
    if let Some((r_sock, r_addr)) = reaper_info {
        let msg = OscMessage {
            path: "/action/40297".to_string(),
            args: vec![],
        };
        send_to_r(r_sock, r_addr, &msg).await?;

        let mut r_selected = state.r_selected;
        if state.x_selected < config.bank_size && config.trk_max > 0 {
            r_selected =
                state.x_selected + state.ch_bank_offset * config.bank_size + config.trk_min;
        } else if state.x_selected < 32 {
            r_selected = state.x_selected + config.trk_min;
        } else if state.x_selected < 40 {
            r_selected = state.x_selected - 32 + config.aux_min;
        } else if state.x_selected < 48 {
            r_selected = state.x_selected - 40 + config.fxr_min;
        } else if state.x_selected < 64 {
            r_selected = state.x_selected - 48 + config.bus_min;
        }

        let msg_sel = OscMessage {
            path: format!("/track/{}/select", r_selected),
            args: vec![OscArg::Float(1.0)],
        };
        send_to_r(r_sock, r_addr, &msg_sel).await?;
    }

    for i in 1..=config.bank_size {
        let src_idx = (i - 1 + state.ch_bank_offset * config.bank_size) as usize;
        if src_idx >= state.bank_tracks.len() {
            continue;
        }
        let track = &state.bank_tracks[src_idx];

        let prefix = format!("/ch/{:02}/", i);

        let msg = OscMessage {
            path: format!("{}mix/fader", prefix),
            args: vec![OscArg::Float(track.fader)],
        };
        send_to_x(sock, addr, &msg, config.delay_bank).await?;

        let msg = OscMessage {
            path: format!("{}mix/pan", prefix),
            args: vec![OscArg::Float(track.pan)],
        };
        send_to_x(sock, addr, &msg, config.delay_bank).await?;

        let msg = OscMessage {
            path: format!("{}mix/on", prefix),
            args: vec![OscArg::Int(if track.mute > 0.5 { 0 } else { 1 })],
        };
        send_to_x(sock, addr, &msg, config.delay_bank).await?;

        for j in 1..=16 {
            let msg = OscMessage {
                path: format!("{}mix/{:02}/level", prefix, j),
                args: vec![OscArg::Float(track.mixbus[j as usize - 1])],
            };
            send_to_x(sock, addr, &msg, config.delay_bank).await?;
        }

        let msg = OscMessage {
            path: format!("{}config/name", prefix),
            args: vec![OscArg::String(track.scribble.clone())],
        };
        send_to_x(sock, addr, &msg, config.delay_bank).await?;

        let msg = OscMessage {
            path: format!("{}config/color", prefix),
            args: vec![OscArg::Int(track.color)],
        };
        send_to_x(sock, addr, &msg, config.delay_bank).await?;

        let msg = OscMessage {
            path: format!("{}config/icon", prefix),
            args: vec![OscArg::Int(track.icon)],
        };
        send_to_x(sock, addr, &msg, config.delay_bank).await?;
    }
    Ok(())
}

// Expanded process_x32_message with full functionality
async fn process_x32_message(
    data: &[u8],
    config: &Config,
    state: &Arc<Mutex<AppState>>,
    r_sock: &UdpSocket,
    r_addr: SocketAddr,
    x_sock: &UdpSocket,
    x_addr: SocketAddr,
) -> Result<()> {
    let msg = match parse_osc_packet(data) {
        Ok(m) => m,
        Err(_) => return Ok(()),
    };

    let mut xr_mask = 0;
    let mut rb_msg: Option<OscMessage> = None;
    let mut state_guard = state.lock().await;

    // Logic for /ch/, /auxin/, /fxrtn/, /bus/, /dca/, /main/st/mix/

    let mut cnum = -1;
    let mut cnum1 = -1;

    if msg.path.starts_with("/ch/") {
        let parts: Vec<&str> = msg.path.split('/').collect();
        if parts.len() >= 3 {
            if let Ok(raw) = parts[2].parse::<i32>() {
                cnum = raw;
                if cnum <= config.bank_size && config.ch_bank_on {
                    cnum = state_guard.ch_bank_offset * config.bank_size + cnum;
                }
                cnum1 = cnum + config.trk_min - 1;
            }
        }
    } else if msg.path.starts_with("/auxin/") {
        let parts: Vec<&str> = msg.path.split('/').collect();
        if parts.len() >= 3 {
            if let Ok(raw) = parts[2].parse::<i32>() {
                cnum = raw; // Used for state indexing? Auxin doesn't use XMbanktracks in C code
                cnum1 = raw + config.aux_min - 1;
            }
        }
    } else if msg.path.starts_with("/fxrtn/") {
        let parts: Vec<&str> = msg.path.split('/').collect();
        if parts.len() >= 3 {
            if let Ok(raw) = parts[2].parse::<i32>() {
                cnum = raw;
                cnum1 = raw + config.fxr_min - 1;
            }
        }
    } else if msg.path.starts_with("/bus/") {
        let parts: Vec<&str> = msg.path.split('/').collect();
        if parts.len() >= 3 {
            if let Ok(raw) = parts[2].parse::<i32>() {
                cnum = raw;
                cnum1 = raw + config.bus_min - 1;
            }
        }
    } else if msg.path.starts_with("/dca/") {
        let parts: Vec<&str> = msg.path.split('/').collect();
        if parts.len() >= 3 {
            if let Ok(raw) = parts[2].parse::<i32>() {
                cnum = raw;
                cnum1 = raw + config.dca_min - 1;
            }
        }
    }

    // Common handler for mix/pan, mix/fader, mix/on, config/name
    if cnum1 != -1 {
        if msg.path.contains("/mix/pan") {
            xr_mask = X32PAN;
            if let Some(OscArg::Float(f)) = msg.args.first() {
                if config.ch_bank_on && msg.path.starts_with("/ch/") {
                    if let Some(track) = state_guard.bank_tracks.get_mut((cnum - 1) as usize) {
                        track.pan = *f;
                    }
                }
                rb_msg = Some(OscMessage {
                    path: format!("/track/{}/pan", cnum1),
                    args: vec![OscArg::Float(*f)],
                });
            }
        } else if msg.path.contains("/mix/fader") {
            xr_mask = X32FADER;
            if let Some(OscArg::Float(f)) = msg.args.first() {
                if config.ch_bank_on && msg.path.starts_with("/ch/") {
                    if let Some(track) = state_guard.bank_tracks.get_mut((cnum - 1) as usize) {
                        track.fader = *f;
                    }
                }
                // Check DCA?
                if msg.path.starts_with("/dca/") {
                    // Logic for DCA fader: Update /track/{cnum1}/volume AND any RDCA tracks
                    let dca_idx = cnum as usize - 1; // 0..7
                    if dca_idx < 8 && dca_idx < config.rdca.len() {
                        let (rmin, rmax) = config.rdca[dca_idx];
                        if rmin > 0 && rmax >= rmin {
                            for r_trk in rmin..=rmax {
                                if (xr_mask & config.xr_send_mask) != 0 {
                                    let m = OscMessage {
                                        path: format!("/track/{}/volume", r_trk),
                                        args: vec![OscArg::Float(*f)],
                                    };
                                    send_to_r(r_sock, r_addr, &m).await?;
                                }
                            }
                        }
                    }
                }
                rb_msg = Some(OscMessage {
                    path: format!("/track/{}/volume", cnum1),
                    args: vec![OscArg::Float(*f)],
                });
            }
        } else if msg.path.contains("/mix/on") {
            xr_mask = X32MUTE;
            if let Some(OscArg::Int(i)) = msg.args.first() {
                let val = if *i == 1 { 0.0 } else { 1.0 };
                if config.ch_bank_on && msg.path.starts_with("/ch/") {
                    if let Some(track) = state_guard.bank_tracks.get_mut((cnum - 1) as usize) {
                        track.mute = val;
                    }
                }
                // Check DCA
                if msg.path.starts_with("/dca/") {
                    let dca_idx = cnum as usize - 1;
                    if dca_idx < 8 && dca_idx < config.rdca.len() {
                        let (rmin, rmax) = config.rdca[dca_idx];
                        if rmin > 0 && rmax >= rmin {
                            for r_trk in rmin..=rmax {
                                if (xr_mask & config.xr_send_mask) != 0 {
                                    let m = OscMessage {
                                        path: format!("/track/{}/mute", r_trk),
                                        args: vec![OscArg::Float(val)],
                                    };
                                    send_to_r(r_sock, r_addr, &m).await?;
                                }
                            }
                        }
                    }
                }
                rb_msg = Some(OscMessage {
                    path: format!("/track/{}/mute", cnum1),
                    args: vec![OscArg::Float(val)],
                });
            }
        } else if msg.path.contains("/config/name") {
            xr_mask = X32NAME;
            if let Some(OscArg::String(s)) = msg.args.first() {
                if config.ch_bank_on && msg.path.starts_with("/ch/") {
                    if let Some(track) = state_guard.bank_tracks.get_mut((cnum - 1) as usize) {
                        track.scribble = s.clone();
                    }
                }
                rb_msg = Some(OscMessage {
                    path: format!("/track/{}/name", cnum1),
                    args: vec![OscArg::String(s.clone())],
                });
            }
        }
        // Sends handling
        if msg.path.contains("/level") {
            xr_mask = X32SEND;
            // /ch/%02d/mix/%02d/level
            let parts: Vec<&str> = msg.path.split('/').collect();
            if parts.len() >= 5 {
                if let Ok(bus) = parts[4].parse::<i32>() {
                    let reaper_bus = bus + config.track_send_offset;
                    if let Some(OscArg::Float(f)) = msg.args.first() {
                        if config.ch_bank_on && msg.path.starts_with("/ch/") {
                            if let Some(track) =
                                state_guard.bank_tracks.get_mut((cnum - 1) as usize)
                            {
                                if bus >= 1 && bus <= 16 {
                                    track.mixbus[bus as usize - 1] = *f;
                                }
                            }
                        }
                        rb_msg = Some(OscMessage {
                            path: format!("/track/{}/send/{}/volume", cnum1, reaper_bus),
                            args: vec![OscArg::Float(*f)],
                        });
                    }
                }
            }
        }
    }

    // /main/st/mix/
    if msg.path.starts_with("/main/st/mix/") && config.master_on {
        if msg.path.contains("fader") {
            xr_mask = X32MFADER;
            if let Some(OscArg::Float(f)) = msg.args.first() {
                rb_msg = Some(OscMessage {
                    path: "/master/volume".to_string(),
                    args: vec![OscArg::Float(*f)],
                });
            }
        } else if msg.path.contains("pan") {
            xr_mask = X32MPAN;
            if let Some(OscArg::Float(f)) = msg.args.first() {
                rb_msg = Some(OscMessage {
                    path: "/master/pan".to_string(),
                    args: vec![OscArg::Float(*f)],
                });
            }
        } else if msg.path.contains("on") {
            xr_mask = X32SELECT; // Using SELECT mask for master select action
            // Unselect all first
            if (xr_mask & config.xr_send_mask) != 0 {
                send_to_r(
                    r_sock,
                    r_addr,
                    &OscMessage {
                        path: "/action/40297".to_string(),
                        args: vec![],
                    },
                )
                .await?;
            }
            // Echo master select on X32
            if (xr_mask & config.xr_send_mask) != 0 {
                send_to_x(
                    x_sock,
                    x_addr,
                    &OscMessage {
                        path: "/-stat/selidx".to_string(),
                        args: vec![OscArg::Int(70)],
                    },
                    config.delay_generic,
                )
                .await?;
            }
            if let Some(OscArg::Int(i)) = msg.args.first() {
                let action = if *i == 1 {
                    "/action/40731"
                } else {
                    "/action/40730"
                };
                rb_msg = Some(OscMessage {
                    path: action.to_string(),
                    args: vec![],
                });
            }
        }
    }

    if msg.path.starts_with("/-stat/") {
        if msg.path.contains("selidx") {
            xr_mask = X32SELECT;
            if (xr_mask & config.xr_send_mask) != 0 {
                send_to_r(
                    r_sock,
                    r_addr,
                    &OscMessage {
                        path: "/action/40297".to_string(),
                        args: vec![],
                    },
                )
                .await?;
            }
            if let Some(OscArg::Int(i)) = msg.args.first() {
                let raw_sel = *i;
                state_guard.x_selected = raw_sel + 1;
                let mut r_sel = -2;

                if raw_sel < config.bank_size && config.trk_max > 0 {
                    if config.ch_bank_on {
                        r_sel = state_guard.x_selected
                            + state_guard.ch_bank_offset * config.bank_size
                            + config.trk_min;
                    } else {
                        r_sel = -2; // Not handled if chbank off and < 32? C logic implies this.
                    }
                } else if raw_sel < 32 && config.trk_max > 0 {
                    r_sel = -2;
                } else if raw_sel < 40 && config.aux_max > 0 {
                    r_sel = raw_sel + config.aux_min - 32;
                } else if raw_sel < 48 && config.fxr_max > 0 {
                    r_sel = raw_sel + config.fxr_min - 40;
                } else if raw_sel < 64 && config.bus_max > 0 {
                    r_sel = raw_sel + config.bus_min - 48;
                }

                if r_sel > -2 {
                    state_guard.r_selected = r_sel;
                    rb_msg = Some(OscMessage {
                        path: format!("/track/{}/select", r_sel),
                        args: vec![OscArg::Float(1.0)],
                    });
                }
            }
        } else if msg.path.contains("solosw") {
            xr_mask = X32SOLO;
            let parts: Vec<&str> = msg.path.split('/').collect();
            if parts.len() >= 4 {
                if let Ok(sw_idx) = parts[3].parse::<i32>() {
                    if let Some(OscArg::Int(val)) = msg.args.first() {
                        let fval = if *val == 1 { 1.0 } else { 0.0 };
                        // Map back to reaper track
                        // This is reverse mapping from X32 solo sw index to Reaper track
                        // Logic similar to selidx mapping but reverse
                        let mut i = 0;
                        if sw_idx < config.bank_size + 1 && config.trk_max > 0 {
                            i = sw_idx + config.trk_min - 1;
                            if config.ch_bank_on {
                                i = state_guard.ch_bank_offset * config.bank_size + i;
                                // Update state
                                if let Some(track) = state_guard
                                    .bank_tracks
                                    .get_mut((i - config.trk_min) as usize)
                                {
                                    track.solo = fval;
                                }
                            }
                        } else if sw_idx < 41 && config.aux_max > 0 {
                            i = sw_idx + config.aux_min - 33;
                        } else if sw_idx < 49 && config.fxr_max > 0 {
                            i = sw_idx + config.fxr_min - 41;
                        } else if sw_idx < 65 && config.bus_max > 0 {
                            i = sw_idx + config.bus_min - 49;
                        } else if sw_idx > 72 && sw_idx < 81 && config.dca_max > 0 {
                            i = sw_idx + config.dca_min - 73;
                        }
                        rb_msg = Some(OscMessage {
                            path: format!("/track/{}/solo", i),
                            args: vec![OscArg::Float(fval)],
                        });
                    }
                }
            }
        } else if msg.path.contains("userpar") {
            let parts: Vec<&str> = msg.path.split('/').collect();
            if parts.len() >= 4 {
                if let Ok(par_idx) = parts[3].parse::<i32>() {
                    if let Some(OscArg::Int(val)) = msg.args.first() {
                        handle_user_par(
                            par_idx,
                            *val,
                            config,
                            &mut state_guard,
                            x_sock,
                            x_addr,
                            r_sock,
                            r_addr,
                        )
                        .await?;
                    }
                }
            }
        }
    }

    if let Some(m) = rb_msg {
        if (xr_mask & config.xr_send_mask) != 0 {
            send_to_r(r_sock, r_addr, &m).await?;
        }
    }

    Ok(())
}

async fn handle_user_par(
    idx: i32,
    val: i32,
    config: &Config,
    state: &mut AppState,
    x_sock: &UdpSocket,
    x_addr: SocketAddr,
    r_sock: &UdpSocket,
    r_addr: SocketAddr,
) -> Result<()> {
    if config.transport_on {
        match idx {
            17 => {
                // REW
                if val == 0 {
                    send_to_r(
                        r_sock,
                        r_addr,
                        &OscMessage {
                            path: "/action/40042".to_string(),
                            args: vec![],
                        },
                    )
                    .await?;
                }
            }
            18 => {
                // PLAY
                if val == 0 {
                    send_to_r(
                        r_sock,
                        r_addr,
                        &OscMessage {
                            path: "/play".to_string(),
                            args: vec![OscArg::Float(1.0)],
                        },
                    )
                    .await?;
                }
            }
            19 => {
                // PAUSE
                if val == 0 {
                    send_to_r(
                        r_sock,
                        r_addr,
                        &OscMessage {
                            path: "/pause".to_string(),
                            args: vec![OscArg::Float(1.0)],
                        },
                    )
                    .await?;
                }
            }
            20 => {
                // FF
                if val == 0 {
                    send_to_r(
                        r_sock,
                        r_addr,
                        &OscMessage {
                            path: "/action/40043".to_string(),
                            args: vec![],
                        },
                    )
                    .await?;
                }
            }
            21 => {
                // Loop
                if val == 0 {
                    if config.ch_bank_on {
                        // Bank UP
                        if state.ch_bank_offset
                            < ((config.trk_max - config.trk_min + 1) / config.bank_size) - 1
                        {
                            state.ch_bank_offset += 1;
                            update_bk_ch(x_sock, x_addr, config, state, Some((r_sock, r_addr)))
                                .await?;
                        }
                    } else {
                        if state.loop_toggle != 0 {
                            send_to_r(
                                r_sock,
                                r_addr,
                                &OscMessage {
                                    path: "/action/40223".to_string(),
                                    args: vec![],
                                },
                            )
                            .await?;
                        } else {
                            send_to_r(
                                r_sock,
                                r_addr,
                                &OscMessage {
                                    path: "/action/40222".to_string(),
                                    args: vec![],
                                },
                            )
                            .await?;
                        }
                        state.loop_toggle ^= 0x7f;
                        send_to_x(
                            x_sock,
                            x_addr,
                            &OscMessage {
                                path: "/-stat/userpar/21/value".to_string(),
                                args: vec![OscArg::Int(state.loop_toggle)],
                            },
                            config.delay_generic,
                        )
                        .await?;
                    }
                }
            }
            22 => {
                // Repeat / Bank Down
                if val == 0 {
                    if config.ch_bank_on {
                        if state.ch_bank_offset > 0 {
                            state.ch_bank_offset -= 1;
                            update_bk_ch(x_sock, x_addr, config, state, Some((r_sock, r_addr)))
                                .await?;
                        }
                    } else {
                        send_to_r(
                            r_sock,
                            r_addr,
                            &OscMessage {
                                path: "/repeat".to_string(),
                                args: vec![OscArg::Float(1.0)],
                            },
                        )
                        .await?;
                    }
                }
            }
            23 => {
                // STOP
                if val == 0 {
                    send_to_r(
                        r_sock,
                        r_addr,
                        &OscMessage {
                            path: "/stop".to_string(),
                            args: vec![OscArg::Float(1.0)],
                        },
                    )
                    .await?;
                }
            }
            24 => {
                // REC
                if val == 0 {
                    send_to_r(
                        r_sock,
                        r_addr,
                        &OscMessage {
                            path: "/record".to_string(),
                            args: vec![OscArg::Float(1.0)],
                        },
                    )
                    .await?;
                }
            }
            // Encoders 33-36 logic omitted for brevity but follows same pattern
            _ => {}
        }
    } else {
        if val == 0 {
            // Button up
            let btn_idx = idx - 12;
            if btn_idx == config.marker_btn {
                send_to_r(
                    r_sock,
                    r_addr,
                    &OscMessage {
                        path: "/action/40157".to_string(),
                        args: vec![],
                    },
                )
                .await?;
            }

            if config.ch_bank_on {
                if btn_idx == config.bank_up {
                    if state.ch_bank_offset
                        < ((config.trk_max - config.trk_min + 1) / config.bank_size) - 1
                    {
                        state.ch_bank_offset += 1;
                        update_bk_ch(x_sock, x_addr, config, state, Some((r_sock, r_addr))).await?;
                    }
                } else if btn_idx == config.bank_dn {
                    if state.ch_bank_offset > 0 {
                        state.ch_bank_offset -= 1;
                        update_bk_ch(x_sock, x_addr, config, state, Some((r_sock, r_addr))).await?;
                    }
                }
            }
        }
    }
    Ok(())
}

async fn process_reaper_message(
    data: &[u8],
    config: &Config,
    state: &Arc<Mutex<AppState>>,
    x_sock: &UdpSocket,
    x_addr: SocketAddr,
    r_sock: &UdpSocket, // Added unused r_sock to match signature logic if needed later or just for symmetry?
    r_addr: SocketAddr,
) -> Result<()> {
    if data.starts_with(b"#bundle") {
        let mut idx = 16;
        while idx < data.len() {
            if idx + 4 > data.len() {
                break;
            }
            let size = u32::from_be_bytes([data[idx], data[idx + 1], data[idx + 2], data[idx + 3]])
                as usize;
            idx += 4;
            if idx + size > data.len() {
                break;
            }
            let msg_data = &data[idx..idx + size];
            process_single_reaper_message(msg_data, config, state, x_sock, x_addr, r_sock, r_addr)
                .await?;
            idx += size;
        }
    } else {
        process_single_reaper_message(data, config, state, x_sock, x_addr, r_sock, r_addr).await?;
    }
    Ok(())
}

async fn process_single_reaper_message(
    data: &[u8],
    config: &Config,
    state: &Arc<Mutex<AppState>>,
    x_sock: &UdpSocket,
    x_addr: SocketAddr,
    r_sock: &UdpSocket,
    r_addr: SocketAddr,
) -> Result<()> {
    let msg = match parse_osc_packet(data) {
        Ok(m) => m,
        Err(_) => return Ok(()),
    };

    let mut xx_mask = 0;
    let mut xb_msg: Option<OscMessage> = None;
    let mut state_guard = state.lock().await;

    if msg.path.starts_with("/track/") {
        let parts: Vec<&str> = msg.path.split('/').collect();
        if parts.len() >= 3 {
            if let Ok(tnum) = parts[2].parse::<i32>() {
                if msg.path.contains("/volume") {
                    xx_mask = TRACKFADER;
                    if let Some(OscArg::Float(f)) = msg.args.first() {
                        let x32_val = (f * 1023.5) as i32 as f32 / 1023.0;
                        if tnum >= config.trk_min && tnum <= config.trk_max {
                            if config.ch_bank_on {
                                let idx = tnum - config.trk_min;
                                if let Some(track) = state_guard.bank_tracks.get_mut(idx as usize) {
                                    track.fader = x32_val;
                                }
                                let bank_cnum = idx - state_guard.ch_bank_offset * config.bank_size;
                                if bank_cnum >= 0 && bank_cnum < config.bank_size {
                                    xb_msg = Some(OscMessage {
                                        path: format!("/ch/{:02}/mix/fader", bank_cnum + 1),
                                        args: vec![OscArg::Float(x32_val)],
                                    });
                                }
                            } else {
                                let cnum = tnum - config.trk_min + 1;
                                if cnum <= config.bank_size {
                                    xb_msg = Some(OscMessage {
                                        path: format!("/ch/{:02}/mix/fader", cnum),
                                        args: vec![OscArg::Float(x32_val)],
                                    });
                                }
                            }
                        }
                        // DCA logic
                        if tnum >= config.dca_min && tnum <= config.dca_max {
                            // Handle Reaper DCA to X32 DCA
                            // Check if this track is in any Rdca range?
                            // Or if this IS an X32 DCA mapped track
                            // C code: if (tnum >= Xdca_min && <= Xdca_max) ... check Rdca_min/max ...
                            // Here `tnum` IS the reaper track number.
                            // If `tnum` corresponds to an X32 DCA.
                            let dca_idx = tnum - config.dca_min; // 0..7
                            if dca_idx >= 0 && dca_idx < 8 {
                                xb_msg = Some(OscMessage {
                                    path: format!("/dca/{}/fader", dca_idx + 1),
                                    args: vec![OscArg::Float(x32_val)],
                                });
                                // If there are Rdca tracks, update them too?
                                // C code updates Reaper tracks if X32 fader moves.
                                // Here Reaper fader moves.
                                // If Reaper DCA moves, we send to X32 DCA.

                                // Also update other Reaper tracks in the group?
                                // C code: if (Rdca_min > 0) ... update all REAPER DCA tracks to same values...
                                // Wait, if Reaper sends /track/X/volume, it means user moved fader X.
                                // If X is a DCA master, we update X32 DCA.
                                // Should we update other Reaper tracks?
                                // C code line 1036: else if (tnum >= Xdca_min ...) { ... update all REAPER DCA tracks ... send_to_r ... }
                                // So yes, we should echo to other Reaper tracks in the group.
                                if (dca_idx as usize) < config.rdca.len() {
                                    let (rmin, rmax) = config.rdca[dca_idx as usize];
                                    if rmin > 0 && rmax >= rmin {
                                        for r_trk in rmin..=rmax {
                                            let m = OscMessage {
                                                path: format!("/track/{}/volume", r_trk),
                                                args: vec![OscArg::Float(x32_val)],
                                            };
                                            send_to_r(r_sock, r_addr, &m).await?;
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if msg.path.contains("/pan") {
                    xx_mask = TRACKPAN;
                    if let Some(OscArg::Float(f)) = msg.args.first() {
                        if tnum >= config.trk_min && tnum <= config.trk_max {
                            if config.ch_bank_on {
                                let idx = tnum - config.trk_min;
                                if let Some(track) = state_guard.bank_tracks.get_mut(idx as usize) {
                                    track.pan = *f;
                                }
                                let bank_cnum = idx - state_guard.ch_bank_offset * config.bank_size;
                                if bank_cnum >= 0 && bank_cnum < config.bank_size {
                                    xb_msg = Some(OscMessage {
                                        path: format!("/ch/{:02}/mix/pan", bank_cnum + 1),
                                        args: vec![OscArg::Float(*f)],
                                    });
                                }
                            } else {
                                let cnum = tnum - config.trk_min + 1;
                                if cnum <= config.bank_size {
                                    xb_msg = Some(OscMessage {
                                        path: format!("/ch/{:02}/mix/pan", cnum),
                                        args: vec![OscArg::Float(*f)],
                                    });
                                }
                            }
                        }
                    }
                } else if msg.path.contains("/mute") {
                    xx_mask = TRACKMUTE;
                    if let Some(OscArg::Float(f)) = msg.args.first() {
                        let x_val = if *f > 0.0 { 0 } else { 1 }; // Reaper 1=mute, X32 0=on (unmute) ??
                        // C code: if (endian.ii == 1) endian.ff = 0.0 else endian.ff = 1.0; (for X32->Reaper)
                        // For Reaper->X32 (line 1157):
                        // if (endian.ff > 0.0) Xb_ls = Xfprint(..., 'i', &zero); else ... 'i', &one.
                        // So if Reaper > 0 (Muted), X32 = 0 (Off/Muted? No, X32 'on' is Unmute).
                        // X32 /mix/on: 1 = ON (audio passes), 0 = OFF (muted).
                        // So Reaper Mute (1) -> X32 On (0).

                        if tnum >= config.trk_min && tnum <= config.trk_max {
                            if config.ch_bank_on {
                                let idx = tnum - config.trk_min;
                                if let Some(track) = state_guard.bank_tracks.get_mut(idx as usize) {
                                    track.mute = *f;
                                }
                                let bank_cnum = idx - state_guard.ch_bank_offset * config.bank_size;
                                if bank_cnum >= 0 && bank_cnum < config.bank_size {
                                    xb_msg = Some(OscMessage {
                                        path: format!("/ch/{:02}/mix/on", bank_cnum + 1),
                                        args: vec![OscArg::Int(x_val)],
                                    });
                                }
                            }
                        }
                    }
                } else if msg.path.contains("/select") {
                    xx_mask = TRACKSELECT;
                    if let Some(OscArg::Float(f)) = msg.args.first() {
                        if *f > 0.5 {
                            state_guard.r_selected = tnum;
                            // Map to X32 selection
                            let mut x_sel = -1;
                            if tnum >= config.trk_min && tnum <= config.trk_max {
                                let idx = tnum - config.trk_min;
                                if config.ch_bank_on {
                                    x_sel = idx - state_guard.ch_bank_offset * config.bank_size;
                                } else {
                                    x_sel = idx;
                                }
                                if x_sel < 0 || x_sel >= config.bank_size {
                                    x_sel = -1;
                                }
                            } else if tnum >= config.aux_min && tnum <= config.aux_max {
                                x_sel = tnum - config.aux_min + 32;
                            }
                            // ... mappings

                            if x_sel >= 0 {
                                state_guard.x_selected = x_sel; // Store 0-based internally?
                                xb_msg = Some(OscMessage {
                                    path: "/-stat/selidx".to_string(),
                                    args: vec![OscArg::Int(x_sel)],
                                });
                            }
                        }
                    }
                }
            }
        }
    } else if msg.path.starts_with("/master/") {
        if config.master_on {
            if msg.path.contains("volume") {
                xx_mask = MASTERVOLUME;
                if let Some(OscArg::Float(f)) = msg.args.first() {
                    xb_msg = Some(OscMessage {
                        path: "/main/st/mix/fader".to_string(),
                        args: vec![OscArg::Float(*f)],
                    });
                }
            } else if msg.path.contains("pan") {
                xx_mask = MASTERPAN;
                if let Some(OscArg::Float(f)) = msg.args.first() {
                    xb_msg = Some(OscMessage {
                        path: "/main/st/mix/pan".to_string(),
                        args: vec![OscArg::Float(*f)],
                    });
                }
            }
        }
    } else if config.transport_on {
        // Transport buttons from Reaper
        if msg.path.starts_with("/play") {
            if let Some(OscArg::Float(f)) = msg.args.first() {
                let val = if *f > 0.5 { 127 } else { 0 };
                if val == 127 {
                    state_guard.play = true;
                } else {
                    state_guard.play = false;
                }
                xb_msg = Some(OscMessage {
                    path: "/-stat/userpar/18/value".to_string(),
                    args: vec![OscArg::Int(val)],
                });
            }
        } else if msg.path.starts_with("/stop") {
            if let Some(OscArg::Float(f)) = msg.args.first() {
                let val = if *f > 0.5 { 127 } else { 0 };
                xb_msg = Some(OscMessage {
                    path: "/-stat/userpar/23/value".to_string(),
                    args: vec![OscArg::Int(val)],
                });
            }
        }
        // ... others
    }

    if let Some(m) = xb_msg {
        if (xx_mask & config.xx_send_mask) != 0 {
            send_to_x(x_sock, x_addr, &m, config.delay_generic).await?;
        }
    }

    Ok(())
}

// Simple OSC parser
fn parse_osc_packet(data: &[u8]) -> Result<OscMessage> {
    let path_end = data
        .iter()
        .position(|&b| b == 0)
        .context("Invalid OSC path")?;
    let path = String::from_utf8_lossy(&data[..path_end]).to_string();

    let type_tag_start = (path_end + 4) & !3;
    if type_tag_start >= data.len() {
        return Ok(OscMessage { path, args: vec![] });
    }

    let type_tag_end = data[type_tag_start..]
        .iter()
        .position(|&b| b == 0)
        .map(|p| p + type_tag_start)
        .unwrap_or(data.len());
    let type_tags = String::from_utf8_lossy(&data[type_tag_start..type_tag_end]);

    let mut args = Vec::new();
    let mut arg_idx = (type_tag_end + 4) & !3;

    if type_tags.starts_with(',') {
        for c in type_tags.chars().skip(1) {
            match c {
                'f' => {
                    if arg_idx + 4 <= data.len() {
                        let bytes = [
                            data[arg_idx],
                            data[arg_idx + 1],
                            data[arg_idx + 2],
                            data[arg_idx + 3],
                        ];
                        let f = f32::from_be_bytes(bytes);
                        args.push(OscArg::Float(f));
                        arg_idx += 4;
                    }
                }
                'i' => {
                    if arg_idx + 4 <= data.len() {
                        let bytes = [
                            data[arg_idx],
                            data[arg_idx + 1],
                            data[arg_idx + 2],
                            data[arg_idx + 3],
                        ];
                        let i = i32::from_be_bytes(bytes);
                        args.push(OscArg::Int(i));
                        arg_idx += 4;
                    }
                }
                's' => {
                    let str_end = data[arg_idx..]
                        .iter()
                        .position(|&b| b == 0)
                        .map(|p| p + arg_idx)
                        .unwrap_or(data.len());
                    let s = String::from_utf8_lossy(&data[arg_idx..str_end]).to_string();
                    args.push(OscArg::String(s));
                    arg_idx = (str_end + 4) & !3;
                }
                _ => {}
            }
        }
    }

    Ok(OscMessage { path, args })
}
