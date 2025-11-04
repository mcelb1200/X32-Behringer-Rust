
use anyhow::{anyhow, Result};
use clap::Parser;
use std::io::{self, BufRead, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::time::{Duration, Instant};
use osc_lib::{OscMessage, OscArg};

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

            let command = OscMessage::new("/info".to_string(), vec![]).to_bytes().map_err(|e: String| anyhow!(e))?;

            for i in 0..5 {
                print!(".");
                io::stdout().flush()?;
                socket.send_to(&command, broadcast_addr)?;
                let mut buf = [0; 512];
                if let Ok((len, _)) = socket.recv_from(&mut buf) {
                    let response = OscMessage::from_bytes(&buf[..len]).map_err(|e: String| anyhow!(e))?;
                    if response.path == "/info" {
                        if let Some(OscArg::String(ip)) = response.args.get(0) {
                            discovered_ip = Some(ip.to_string());
                            break;
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
        let file = std::fs::File::open(scene_file)?;
        let reader = io::BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            if !line.starts_with('#') {
                let msg = OscMessage::new(
                    format!("/{}", line),
                    vec![OscArg::String(line.clone())]
                );
                let command = msg.to_bytes().map_err(|e: String| anyhow!(e))?;
                socket.send(&command)?;
                check_for_response(&socket, verbose, args.debug)?;
            }
        }
    } else if let Some(batch_file) = args.file {
        let file = std::fs::File::open(batch_file)?;
        let reader = io::BufReader::new(file);
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
                let xremote_cmd = OscMessage::new("/xremote".to_string(), vec![]).to_bytes().map_err(|e: String| anyhow!(e))?;
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

fn handle_command(command_str: &str, socket: &UdpSocket, xremote_on: &mut bool, verbose: &mut bool, debug: bool) -> Result<()> {
    match command_str {
        "xremote on" => *xremote_on = true,
        "xremote off" => *xremote_on = false,
        "verbose on" => *verbose = true,
        "verbose off" => *verbose = false,
        _ => {
            let osc_command = parse_command(command_str)?;
            if *verbose {
                println!("->X: {} {:?}", osc_command.path, osc_command.args);
            }
            socket.send(&osc_command.to_bytes().map_err(|e: String| anyhow!(e))?)?;
        }
    }
    Ok(())
}

fn check_for_response(socket: &UdpSocket, verbose: bool, debug: bool) -> Result<()> {
    let mut buf = [0; 512];
    while let Ok(len) = socket.recv(&mut buf) {
        if verbose {
            let response = OscMessage::from_bytes(&buf[..len]).map_err(|e: String| anyhow!(e))?;
            println!("X->: {} {:?}", response.path, response.args);
        }
    }
    Ok(())
}

fn parse_command(command_str: &str) -> Result<OscMessage> {
    let mut parts = command_str.split_whitespace();
    let path = parts.next().ok_or_else(|| anyhow!("Empty command"))?.to_string();
    let mut args = Vec::new();
    if let Some(type_tags) = parts.next() {
        if type_tags.starts_with(',') {
            for (i, tag) in type_tags.chars().skip(1).enumerate() {
                let arg_str = parts.next().ok_or_else(|| anyhow!(format!("Missing argument for type tag '{}'", tag)))?;
                match tag {
                    'i' => args.push(OscArg::Int(arg_str.parse()?)),
                    'f' => args.push(OscArg::Float(arg_str.parse()?)),
                    's' => args.push(OscArg::String(arg_str.to_string())),
                    _ => return Err(anyhow!(format!("Unsupported type tag: {}", tag))),
                }
            }
        } else {
            // No type tags, treat remaining as string arguments
            let mut arg_string = type_tags.to_string();
            for part in parts {
                arg_string.push(' ');
                arg_string.push_str(part);
            }
            args.push(OscArg::String(arg_string));
        }
    }

    Ok(OscMessage::new(path, args))
}
