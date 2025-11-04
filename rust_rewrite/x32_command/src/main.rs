
use anyhow::{anyhow, Result};
use clap::Parser;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::time::{Duration, Instant};
use x32_lib::{cparse, dump};

/// A command-line tool for sending OSC commands to a Behringer X32/X-Air mixer.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The IP address of the X32 console.
    #[arg(short, long)]
    ip: Option<String>,

    /// Enable debug mode.
    #[arg(short, long, default_value_t = false)]
    debug: bool,

    /// Run in batch mode, getting input from a file.
    #[arg(short, long)]
    file: Option<String>,

    /// Disable interactive keyboard mode.
    #[arg(short, long, default_value_t = false)]
    no_keyboard: bool,

    /// Read and send scene/snippet lines from a file.
    #[arg(short, long)]
    scene: Option<String>,

    /// Delay between batch commands in milliseconds.
    #[arg(short, long, default_value_t = 10)]
    delay: u64,

    /// Enable verbose output.
    #[arg(short, long, default_value_t = true)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_read_timeout(Some(Duration::from_millis(500)))?;

    let x32_addr = match args.ip {
        Some(ip_str) => ip_str.parse()?,
        None => {
            println!("No IP address provided. Attempting to discover X32 on the network...");
            socket.set_broadcast(true)?;
            let broadcast_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255)), 10023);
            let mut discovered_ip = None;

            let command = cparse::xcparse("/info").map_err(|e| anyhow!(e))?;

            for i in 0..5 {
                print!(".");
                io::stdout().flush()?;
                socket.send_to(&command, broadcast_addr)?;
                let mut buf = [0; 512];
                if let Ok((len, _)) = socket.recv_from(&mut buf) {
                    let response = &buf[..len];
                    if response.starts_with(b"/info") {
                       if let Some(ip_start) = response.windows(16).position(|window| window == b"ip\"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00") {
                            let ip_bytes_start = ip_start + 16;
                            if let Some(ip_end) = (ip_bytes_start..len).find(|&i| response[i] == 0) {
                                let ip_str = String::from_utf8_lossy(&response[ip_bytes_start..ip_end]);
                                discovered_ip = Some(ip_str.to_string());
                                break;
                            }
                        }
                    }
                }
                if i == 4 {
                     return Err(anyhow!("Could not discover X32. Please specify the IP address with the -i flag."));
                }
            }
            println!();
            let ip = discovered_ip.ok_or_else(|| anyhow!("Could not discover X32. Please specify the IP address with the -i flag."))?;
            println!("X32 discovered at {}", ip);
            ip
        }
    };

    let x32_socket_addr = SocketAddr::new(x32_addr.parse()?, 10023);
    socket.connect(x32_socket_addr)?;
    socket.set_read_timeout(Some(Duration::from_millis(1)))?;

    let mut xremote_on = false;
    let mut last_xremote_time = Instant::now();
    let mut verbose = args.verbose;

    if let Some(scene_file) = args.scene {
        let file = File::open(scene_file)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            if !line.starts_with('#') {
                let command = cparse::xcparse(&format!("/{},s,{}", line, line)).map_err(|e| anyhow!(e))?;
                socket.send(&command)?;
                check_for_response(&socket, verbose, args.debug)?;
            }
        }
    } else if let Some(batch_file) = args.file {
        let file = File::open(batch_file)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            if line.starts_with('#') {
                println!("---comment: {}", line);
            } else {
                handle_command(&line, &socket, &mut xremote_on, &mut verbose, args.debug)?;
                check_for_response(&socket, verbose, args.debug)?;
            }
            std::thread::sleep(Duration::from_millis(args.delay));
        }
    }

    if !args.no_keyboard {
        println!("Entering interactive mode. Type 'exit' or 'quit' to close.");
        let mut last_command = String::new();
        loop {
            if xremote_on && last_xremote_time.elapsed() > Duration::from_secs(9) {
                let xremote_cmd = cparse::xcparse("/xremote").map_err(|e| anyhow!(e))?;
                socket.send(&xremote_cmd)?;
                last_xremote_time = Instant::now();
            }

            check_for_response(&socket, verbose, args.debug)?;

            print!("> ");
            io::stdout().flush()?;
            let mut input = String::new();
            if io::stdin().read_line(&mut input)? > 0 {
                let mut command = input.trim().to_string();
                if command.is_empty() {
                    command = last_command.clone();
                } else {
                    last_command = command.clone();
                }

                if command == "exit" || command == "quit" {
                    break;
                }

                handle_command(&command, &socket, &mut xremote_on, &mut verbose, args.debug)?;
            }
        }
    }

    Ok(())
}

fn handle_command(command: &str, socket: &UdpSocket, xremote_on: &mut bool, verbose: &mut bool, debug: bool) -> Result<()> {
    match command {
        "xremote on" => *xremote_on = true,
        "xremote off" => *xremote_on = false,
        "verbose on" => *verbose = true,
        "verbose off" => *verbose = false,
        _ => {
            let osc_command = cparse::xcparse(command).map_err(|e| anyhow!(e))?;
            if *verbose {
                println!("{}", dump::xfdump("->X", &osc_command, debug));
            }
            socket.send(&osc_command)?;
        }
    }
    Ok(())
}

fn check_for_response(socket: &UdpSocket, verbose: bool, debug: bool) -> Result<()> {
    let mut buf = [0; 512];
    while let Ok(len) = socket.recv(&mut buf) {
        if verbose {
            println!("{}", dump::xfdump("X->", &buf[..len], debug));
        }
    }
    Ok(())
}
