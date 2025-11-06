
//! # x32_get_lib
//!
//! `x32_get_lib` is a command-line tool for saving preset libraries from a Behringer X32 or Midas M32 mixer to a local directory.
use anyhow::Result;
use clap::{Parser, ValueEnum};
use std::net::UdpSocket;
use strum_macros::Display;
use x32_lib::{create_socket, OscMessage, OscArg};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// IP address of the X32 mixer
    #[arg(long)]
    ip: String,

    /// Port number of the X32 mixer
    #[arg(long, default_value_t = 10024)]
    port: u16,

    /// Remote port number of the X32 mixer
    #[arg(long, default_value_t = 10023)]
    remote_port: u16,

    /// Directory to save the preset library
    #[arg(long)]
    directory: String,

    /// Type of library to save
    #[arg(long, value_enum)]
    library_type: LibraryType,
}

#[derive(Debug, Clone, ValueEnum, Display)]
pub enum LibraryType {
    /// All available libraries
    All,
    /// Channel presets
    Channel,
    /// Effects presets
    Effects,
    /// Routing presets
    Routing,
}

fn main() -> Result<()> {
    let args = Args::parse();
    println!("Saving {} library to {} from {}", args.library_type, args.directory, args.ip);

    save_presets(&args.ip, args.port, args.remote_port, &args.directory, &args.library_type)?;

    Ok(())
}

fn save_presets(ip: &str, port: u16, remote_port: u16, directory: &str, library_type: &LibraryType) -> Result<()> {
    let socket = create_socket(ip, port, remote_port, 2000)?;
    println!("Connected to X32 mixer at {}", ip);

    // Create directory if it doesn't exist
    std::fs::create_dir_all(directory)?;

    match library_type {
        LibraryType::All => {
            println!("Saving all libraries...");
            save_channel_presets(&socket, directory)?;
            save_effects_presets(&socket, directory)?;
            save_routing_presets(&socket, directory)?;
        }
        LibraryType::Channel => {
            println!("Saving channel presets...");
            save_channel_presets(&socket, directory)?;
        }
        LibraryType::Effects => {
            println!("Saving effects presets...");
            save_effects_presets(&socket, directory)?;
        }
        LibraryType::Routing => {
            println!("Saving routing presets...");
            save_routing_presets(&socket, directory)?;
        }
    }

    Ok(())
}

pub fn save_channel_presets(socket: &UdpSocket, directory: &str) -> Result<()> {
    for i in 1..=100 {
        let mut buf = [0; 1024];
        let msg = OscMessage::new(format!("/-libs/ch/{:03}/hasdata", i), vec![]);
        socket.send(&msg.to_bytes()?)?;
        std::thread::sleep(std::time::Duration::from_millis(10));
        let (len, _) = socket.recv_from(&mut buf)?;
        let response = OscMessage::from_bytes(&buf[..len])?;

        if let Some(OscArg::Int(hasdata)) = response.args.get(0) {
            if *hasdata == 0 {
                continue;
            }
            if *hasdata == 1 {
                // Get preset name
                let msg = OscMessage::new("/node".to_string(), vec![OscArg::String(format!("-libs/ch/{:03}", i))]);
                socket.send(&msg.to_bytes()?)?;
                let (len, _) = socket.recv_from(&mut buf)?;
                let response = OscMessage::from_bytes(&buf[..len])?;
                let preset_name = response.args[1].clone();
                let preset_name = match preset_name {
                    OscArg::String(s) => s,
                    _ => "unknown".to_string(),
                };

                println!("Saving channel preset {}: {}", i, preset_name);


                // Load preset
                let msg = OscMessage::new("/load".to_string(), vec![
                    OscArg::String("libchan".to_string()),
                    OscArg::Int(i as i32 - 1),
                    OscArg::Int(0),
                    OscArg::Int(63),
                ]);
                socket.send(&msg.to_bytes()?)?;
                std::thread::sleep(std::time::Duration::from_millis(30));

                // Get preset data
                let mut preset_data = String::new();
                preset_data.push_str("#2.1#\n");

                // Get node data
                let msg = OscMessage::new("/node".to_string(), vec![
                    OscArg::String(format!("-libs/ch/{:03}", i)),
                ]);
                socket.send(&msg.to_bytes()?)?;
                let (len, _) = socket.recv_from(&mut buf)?;
                let response = OscMessage::from_bytes(&buf[..len])?;
                if let OscArg::String(s) = &response.args[0] {
                    preset_data.push_str(s);
                }
                preset_data.push_str("\n");


                let path = std::path::Path::new(directory).join(format!("{}.chn", preset_name));
                std::fs::write(path, preset_data)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args() {
        let args = Args::parse_from(&[
            "x32_get_lib",
            "--ip",
            "127.0.0.1",
            "--directory",
            "/tmp",
            "--library-type",
            "all",
        ]);
        assert_eq!(args.ip, "127.0.0.1");
        assert_eq!(args.directory, "/tmp");
        assert_eq!(args.library_type.to_string().to_lowercase(), "all");
    }
}

pub fn save_effects_presets(socket: &UdpSocket, directory: &str) -> Result<()> {
    for i in 1..=100 {
        let mut buf = [0; 1024];
        let msg = OscMessage::new(format!("/-libs/fx/{:03}/hasdata", i), vec![]);
        socket.send(&msg.to_bytes()?)?;
        std::thread::sleep(std::time::Duration::from_millis(10));
        let (len, _) = socket.recv_from(&mut buf)?;
        let response = OscMessage::from_bytes(&buf[..len])?;

        if let Some(OscArg::Int(hasdata)) = response.args.get(0) {
            if *hasdata == 0 {
                continue;
            }
            if *hasdata == 1 {
                // Get preset name
                let msg = OscMessage::new("/node".to_string(), vec![OscArg::String(format!("-libs/fx/{:03}", i))]);
                socket.send(&msg.to_bytes()?)?;
                let (len, _) = socket.recv_from(&mut buf)?;
                let response = OscMessage::from_bytes(&buf[..len])?;
                let preset_name = response.args[1].clone();
                let preset_name = match preset_name {
                    OscArg::String(s) => s,
                    _ => "unknown".to_string(),
                };

                println!("Saving effects preset {}: {}", i, preset_name);


                // Load preset
                let msg = OscMessage::new("/load".to_string(), vec![
                    OscArg::String("libfx".to_string()),
                    OscArg::Int(i as i32 - 1),
                    OscArg::Int(0),
                ]);
                socket.send(&msg.to_bytes()?)?;
                std::thread::sleep(std::time::Duration::from_millis(30));

                // Get preset data
                let mut preset_data = String::new();
                preset_data.push_str("#2.1#\n");

                // Get node data
                let msg = OscMessage::new("/node".to_string(), vec![
                    OscArg::String(format!("-libs/fx/{:03}", i)),
                ]);
                socket.send(&msg.to_bytes()?)?;
                let (len, _) = socket.recv_from(&mut buf)?;
                let response = OscMessage::from_bytes(&buf[..len])?;
                if let OscArg::String(s) = &response.args[0] {
                    preset_data.push_str(s);
                }
                preset_data.push_str("\n");


                let path = std::path::Path::new(directory).join(format!("{}.efx", preset_name));
                std::fs::write(path, preset_data)?;
            }
        }
    }
    Ok(())
}

pub fn save_routing_presets(socket: &UdpSocket, directory: &str) -> Result<()> {
    for i in 1..=100 {
        let mut buf = [0; 1024];
        let msg = OscMessage::new(format!("/-libs/r/{:03}/hasdata", i), vec![]);
        socket.send(&msg.to_bytes()?)?;
        std::thread::sleep(std::time::Duration::from_millis(10));
        let (len, _) = socket.recv_from(&mut buf)?;
        let response = OscMessage::from_bytes(&buf[..len])?;

        if let Some(OscArg::Int(hasdata)) = response.args.get(0) {
            if *hasdata == 0 {
                continue;
            }
            if *hasdata == 1 {
                // Get preset name
                let msg = OscMessage::new("/node".to_string(), vec![OscArg::String(format!("-libs/r/{:03}", i))]);
                socket.send(&msg.to_bytes()?)?;
                let (len, _) = socket.recv_from(&mut buf)?;
                let response = OscMessage::from_bytes(&buf[..len])?;
                let preset_name = response.args[1].clone();
                let preset_name = match preset_name {
                    OscArg::String(s) => s,
                    _ => "unknown".to_string(),
                };

                println!("Saving routing preset {}: {}", i, preset_name);


                // Load preset
                let msg = OscMessage::new("/load".to_string(), vec![
                    OscArg::String("librout".to_string()),
                    OscArg::Int(i as i32 - 1),
                ]);
                socket.send(&msg.to_bytes()?)?;
                std::thread::sleep(std::time::Duration::from_millis(30));

                // Get preset data
                let mut preset_data = String::new();
                preset_data.push_str("#2.1#\n");

                // Get node data
                let msg = OscMessage::new("/node".to_string(), vec![
                    OscArg::String(format!("-libs/r/{:03}", i)),
                ]);
                socket.send(&msg.to_bytes()?)?;
                let (len, _) = socket.recv_from(&mut buf)?;
                let response = OscMessage::from_bytes(&buf[..len])?;
                if let OscArg::String(s) = &response.args[0] {
                    preset_data.push_str(s);
                }
                preset_data.push_str("\n");


                let path = std::path::Path::new(directory).join(format!("{}.rou", preset_name));
                std::fs::write(path, preset_data)?;
            }
        }
    }
    Ok(())
}
