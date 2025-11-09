//! # x32_usb
//!
//! `x32_usb` is a command-line utility for managing the USB drive on a Behringer X32 or Midas M32
//! digital mixer. It provides a shell-like interface for listing files, changing directories,
//! loading scenes and presets, and controlling WAV file playback.
//!
//! This utility is a Rust rewrite of the original C program `X32USB.c` by Patrick-Gilles Maillot.

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use x32_lib::{create_socket, error::X32Error};
use osc_lib::{OscMessage, OscArg};
use std::net::UdpSocket;
use std::str::FromStr;

#[derive(Parser, Debug)]
#[command(author, version, about = "An X32 shell for remote managing USB files", long_about = None)]
/// Command-line arguments for the x32_usb utility.
struct Args {
    #[arg(short, long, default_value = "127.0.0.1", help = "X32 console IP address")]
    /// The IP address of the X32/M32 console.
    ip: String,

    #[command(subcommand)]
    /// The subcommand to execute.
    command: Commands,
}

#[derive(Subcommand, Debug)]
/// Defines the available subcommands for the utility.
enum Commands {
    #[command(about = "List directory contents")]
    /// Lists the files and directories in the current directory on the USB drive.
    Ls,
    #[command(about = "Change directory")]
    /// Changes the current directory to the specified directory.
    Cd {
        #[arg(help = "Directory ID or name")]
        /// The index or name of the directory to change to.
        target: String,
    },
    #[command(about = "Load or run a file")]
    /// Loads a scene, snippet, or other preset file.
    Load {
        #[arg(help = "File ID or name")]
        /// The index or name of the file to load.
        target: String,
    },
    #[command(about = "Unmount the USB drive")]
    /// Unmounts the USB drive from the console.
    Umount,
    #[command(about = "Play a WAV file")]
    /// Plays the specified WAV file.
    Play {
        #[arg(help = "File ID or name")]
        /// The index or name of the WAV file to play.
        target: String,
    },
    #[command(about = "Stop a currently playing WAV file")]
    /// Stops the currently playing WAV file.
    Stop,
    #[command(about = "Pause a currently playing WAV file")]
    /// Pauses the currently playing WAV file.
    Pause,
    #[command(about = "Resume a paused WAV file")]
    /// Resumes playback of a paused WAV file.
    Resume,
}

#[derive(Debug, PartialEq)]
/// Represents the type of a file on the USB drive.
enum FileType {
    Unknown,
    Volume,
    Parent,
    Directory,
    Wav,
    Show,
    Scene,
    Snippet,
    Effects,
    Preference,
    Routing,
    Channel,
}

impl FromStr for FileType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        if s == "[..]" {
            return Ok(FileType::Parent);
        }
        if s == "[System Volume Information]" {
            return Ok(FileType::Volume);
        }
        if s.starts_with('[') && s.ends_with(']') {
            return Ok(FileType::Directory);
        }

        let extension = s.split('.').last().unwrap_or("");
        match extension.to_lowercase().as_str() {
            "wav" => Ok(FileType::Wav),
            "shw" => Ok(FileType::Show),
            "scn" => Ok(FileType::Scene),
            "snp" => Ok(FileType::Snippet),
            "efx" => Ok(FileType::Effects),
            "prf" => Ok(FileType::Preference),
            "rou" => Ok(FileType::Routing),
            "chn" => Ok(FileType::Channel),
            _ => Ok(FileType::Unknown),
        }
    }
}

#[derive(Debug)]
/// Represents a file or directory on the USB drive.
struct FileEntry {
    /// The index of the file in the directory listing.
    index: i32,
    /// The name of the file.
    name: String,
    /// The type of the file.
    file_type: FileType,
}

/// A client for communicating with the X32/M32 console.
struct X32Client {
    socket: UdpSocket,
}

impl X32Client {
    /// Creates a new `X32Client` and connects to the console.
    fn new(ip_address: &str) -> Result<Self> {
        let socket = create_socket(ip_address, 500)?;
        Ok(Self { socket })
    }

    /// Sends an OSC message to the console.
    fn send(&self, message: &OscMessage) -> Result<()> {
        let bytes = message.to_bytes()?;
        self.socket.send(&bytes)?;
        Ok(())
    }

    /// Receives an OSC message from the console.
    fn receive(&self) -> Result<OscMessage> {
        let mut buf = [0; 512];
        let len = self.socket.recv(&mut buf)?;
        let message = OscMessage::from_bytes(&buf[..len])?;
        Ok(message)
    }

    /// Checks if a USB drive is mounted on the console.
    fn is_usb_mounted(&self) -> Result<bool> {
        let msg = OscMessage::new("/-stat/usbmounted".to_string(), vec![]);
        self.send(&msg)?;
        match self.receive() {
            Ok(response) => {
                if let Some(OscArg::Int(val)) = response.args.get(0) {
                    Ok(*val == 1)
                } else {
                    Ok(false)
                }
            }
            Err(e) => {
                if let Some(x32_err) = e.downcast_ref::<X32Error>() {
                    if let X32Error::Io(io_err) = x32_err {
                        if io_err.kind() == std::io::ErrorKind::WouldBlock || io_err.kind() == std::io::ErrorKind::TimedOut {
                            return Ok(false);
                        }
                    }
                }
                Err(e)
            }
        }
    }

    /// Gets a list of files and directories in the current directory on the USB drive.
    fn get_file_list(&self) -> Result<Vec<FileEntry>> {
        let msg = OscMessage::new("/-usb/dir/maxpos".to_string(), vec![]);
        self.send(&msg)?;
        let response = self.receive()?;

        let num_files = if let Some(OscArg::Int(val)) = response.args.get(0) {
            *val
        } else {
            return Err(anyhow!("Failed to get number of files from X32."));
        };

        let mut files = Vec::new();
        for i in 1..=num_files {
            let path = format!("/-usb/dir/{:03}/name", i);
            let msg = OscMessage::new(path, vec![]);
            self.send(&msg)?;
            let response = self.receive()?;
            if let Some(OscArg::String(name)) = response.args.get(0) {
                let file_type = FileType::from_str(name)?;
                files.push(FileEntry {
                    index: i,
                    name: name.to_string(),
                    file_type,
                });
            }
        }
        Ok(files)
    }

    /// Selects a file or directory on the USB drive.
    fn select_file(&self, file_index: i32) -> Result<()> {
        let msg = OscMessage::new("/-action/recselect".to_string(), vec![OscArg::Int(file_index)]);
        self.send(&msg)
    }

    /// Finds a file or directory by its index or name.
    fn find_file(&self, target: &str) -> Result<FileEntry> {
        let files = self.get_file_list()?;
        files.into_iter().find(|f| {
            if let Ok(index) = target.parse::<i32>() {
                f.index == index
            } else {
                let name_to_compare = if f.file_type == FileType::Directory {
                    &f.name[1..f.name.len() - 1]
                } else {
                    &f.name
                };
                name_to_compare == target
            }
        }).ok_or_else(|| anyhow!("File not found: {}", target))
    }

    /// Sets the playback state of the tape deck.
    fn set_tape_state(&self, state: i32) -> Result<()> {
        let msg = OscMessage::new("/-stat/tape/state".to_string(), vec![OscArg::Int(state)]);
        self.send(&msg)
    }

    /// Unmounts the USB drive.
    fn unmount(&self) -> Result<()> {
        let msg = OscMessage::new("/-stat/usbmounted".to_string(), vec![OscArg::Int(0)]);
        self.send(&msg)
    }
}

/// The main logic for the utility.
fn run(args: Args) -> Result<()> {
    let client = X32Client::new(&args.ip)?;

    if !client.is_usb_mounted()? {
        println!("USB drive is not mounted.");
        return Ok(());
    }

    match &args.command {
        Commands::Ls => {
            let files = client.get_file_list()?;
            for file in files {
                println!("{:?}", file);
            }
        }
        Commands::Cd { target } => {
            let file = client.find_file(target)?;
            if file.file_type == FileType::Directory || file.file_type == FileType::Parent {
                client.select_file(file.index)?;
                println!("Changed directory to {}", file.name);
            } else {
                return Err(anyhow!("Not a directory: {}", file.name));
            }
        }
        Commands::Load { target } => {
            let file = client.find_file(target)?;
            match file.file_type {
                FileType::Scene | FileType::Snippet | FileType::Effects | FileType::Routing | FileType::Channel => {
                    client.select_file(file.index)?;
                    println!("Loaded file: {}", file.name);
                }
                _ => return Err(anyhow!("Not a loadable file: {}", file.name)),
            }
        }
        Commands::Umount => {
            client.unmount()?;
            println!("USB drive unmounted.");
        }
        Commands::Play { target } => {
            let file = client.find_file(target)?;
            if file.file_type == FileType::Wav {
                client.select_file(file.index)?;
                println!("Playing file: {}", file.name);
            } else {
                return Err(anyhow!("Not a WAV file: {}", file.name));
            }
        }
        Commands::Stop => {
            client.set_tape_state(0)?;
            println!("Playback stopped.");
        }
        Commands::Pause => {
            client.set_tape_state(1)?;
            println!("Playback paused.");
        }
        Commands::Resume => {
            client.set_tape_state(2)?;
            println!("Playback resumed.");
        }
    }

    Ok(())
}

/// The entry point of the application.
fn main() {
    let args = Args::parse();
    if let Err(e) = run(args) {
        // Check if the error is our custom X32Error wrapping an IO error
        if let Some(x32_err) = e.downcast_ref::<X32Error>() {
            if let X32Error::Io(io_err) = x32_err {
                if io_err.kind() == std::io::ErrorKind::ConnectionRefused {
                    println!("Not connected to X32.");
                    std::process::exit(1);
                }
            }
        // Check if the error is a direct IO error
        } else if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
            if io_err.kind() == std::io::ErrorKind::ConnectionRefused {
                println!("Not connected to X32.");
                std::process::exit(1);
            }
        }

        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
