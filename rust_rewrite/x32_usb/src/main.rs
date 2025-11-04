
use anyhow::{anyhow, Result};
use clap::Parser;
use std::io::{self, Write};
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;
use osc_lib::{OscMessage, OscArg};

/// A command-line tool for managing a USB drive on an X32 mixer.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The IP address of the X32 console.
    #[arg(short, long, default_value = "192.168.1.62")]
    ip: String,

    /// Delay between commands in milliseconds.
    #[arg(short, long, default_value_t = 0)]
    delay: u64,
}

#[derive(Debug, Clone, PartialEq)]
enum FileType {
    Unknown,
    Volume,
    Parent,
    Directory,
    Wav,
    Show,
    Scene,
    Snippet,
    Effect,
    Preference,
    Routing,
    Channel,
}

#[derive(Debug, Clone)]
struct FileNode {
    name: String,
    file_type: FileType,
    index: i32,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_read_timeout(Some(Duration::from_millis(500)))?;

    let x32_addr: SocketAddr = format!("{}:10023", args.ip).parse()?;
    socket.connect(x32_addr)?;

    println!("X32USB - v0.3 - (c)2015-17 Patrick-Gilles Maillot\n");

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
        print!(".");
        io::stdout().flush()?;
    }
    println!(" Done!");

    socket.set_read_timeout(Some(Duration::from_millis(50)))?;
    let mut prompt = ">:".to_string();
    let mut mounted = false;
    let mut file_tree: Vec<FileNode> = Vec::new();

    loop {
        print!("{} ", prompt);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        let mut parts = input.split_whitespace();
        let command = parts.next().unwrap_or("");
        let arg = parts.next().unwrap_or("");

        match command {
            "ls" | "dir" => {
                if mounted {
                    file_tree = list_directory(&socket)?;
                    for file in &file_tree {
                        println!("{:3} - {:7} - {}", file.index, file_type_to_str(&file.file_type), file.name);
                    }
                } else {
                    println!("No USB stick mounted!");
                }
            }
            "cd" => {
                if mounted {
                    if let Ok(index) = arg.parse::<i32>() {
                        change_directory(&socket, index)?;
                        let dir_name = get_dir_name(&file_tree, index).unwrap_or("");
                        if dir_name == "[..]" {
                            if let Some(pos) = prompt.rfind('/') {
                                prompt.truncate(pos);
                            }
                        } else {
                            prompt.push_str(&format!("{}/", dir_name.trim_matches(|c| c == '[' || c == ']')));
                        }
                    } else {
                        if let Some(file) = file_tree.iter().find(|f| f.name == arg && f.file_type == FileType::Directory) {
                            change_directory(&socket, file.index)?;
                            prompt.push_str(&format!("{}/", file.name.trim_matches(|c| c == '[' || c == ']')));
                        } else {
                            println!("Directory not found!");
                        }
                    }
                } else {
                    println!("No USB stick mounted!");
                }
            }
            "load" | "run" => {
                if mounted {
                    if let Ok(index) = arg.parse::<i32>() {
                        load_file(&socket, index)?;
                    } else {
                        if let Some(file) = file_tree.iter().find(|f| f.name == arg) {
                            load_file(&socket, file.index)?;
                        } else {
                            println!("File not found!");
                        }
                    }
                } else {
                    println!("No USB stick mounted!");
                }
            }
            "play" => {
                 if mounted {
                    if let Ok(index) = arg.parse::<i32>() {
                        play_file(&socket, index)?;
                    } else {
                        if let Some(file) = file_tree.iter().find(|f| f.name == arg && f.file_type == FileType::Wav) {
                            play_file(&socket, file.index)?;
                        } else {
                            println!("File not found!");
                        }
                    }
                } else {
                    println!("No USB stick mounted!");
                }
            }
            "stop" => {
                let stop_cmd = OscMessage::new("/-stat/tape/state".to_string(), vec![OscArg::Int(0)]).to_bytes().map_err(|e: String| anyhow!(e))?;
                socket.send(&stop_cmd)?;
            }
            "pause" => {
                let pause_cmd = OscMessage::new("/-stat/tape/state".to_string(), vec![OscArg::Int(1)]).to_bytes().map_err(|e: String| anyhow!(e))?;
                socket.send(&pause_cmd)?;
            }
            "resume" => {
                let resume_cmd = OscMessage::new("/-stat/tape/state".to_string(), vec![OscArg::Int(2)]).to_bytes().map_err(|e: String| anyhow!(e))?;
                socket.send(&resume_cmd)?;
            }
            "umount" => {
                let umount_cmd = OscMessage::new("/-stat/usbmounted".to_string(), vec![OscArg::Int(0)]).to_bytes().map_err(|e: String| anyhow!(e))?;
                socket.send(&umount_cmd)?;
                mounted = false;
                prompt = ">:".to_string();
            }
            "help" => {
                println!("  ls                  List directory contents");
                println!("  cd <id> | <name>    Change directory");
                println!("  load <id> | <name>  Load or Run file");
                println!("  play <id> | <name>  Play WAV file");
                println!("  stop                Stops playback");
                println!("  pause               Pauses playback");
                println!("  resume              Resumes playback");
                println!("  umount              Unmount the USB drive");
                println!("  exit | quit         Exits program");
            }
            "exit" | "quit" => break,
            _ => {
                 if !mounted {
                    let mounted_cmd = OscMessage::new("/-stat/usbmounted".to_string(), vec![]).to_bytes().map_err(|e: String| anyhow!(e))?;
                    socket.send(&mounted_cmd)?;
                    let mut buf = [0; 512];
                    if let Ok(len) = socket.recv(&mut buf) {
                        let msg = OscMessage::from_bytes(&buf[..len]).map_err(|e: String| anyhow!(e))?;
                        if msg.path == "/-stat/usbmounted" {
                            if let Some(OscArg::Int(1)) = msg.args.get(0) {
                                mounted = true;
                                prompt = "$:".to_string();
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn list_directory(socket: &UdpSocket) -> Result<Vec<FileNode>> {
    let mut file_tree = Vec::new();
    let maxpos_cmd = OscMessage::new("/-usb/dir/maxpos".to_string(), vec![]).to_bytes().map_err(|e: String| anyhow!(e))?;
    socket.send(&maxpos_cmd)?;
    let mut buf = [0; 512];
    if let Ok(len) = socket.recv(&mut buf) {
        let msg = OscMessage::from_bytes(&buf[..len]).map_err(|e: String| anyhow!(e))?;
        if msg.path == "/-usb/dir/maxpos" {
            if let Some(OscArg::Int(num_files)) = msg.args.get(0) {
                for i in 1..=*num_files {
                    let name_cmd = OscMessage::new(format!("/-usb/dir/{:03}/name", i), vec![]).to_bytes().map_err(|e: String| anyhow!(e))?;
                    socket.send(&name_cmd)?;
                    if let Ok(len) = socket.recv(&mut buf) {
                        let name_msg = OscMessage::from_bytes(&buf[..len]).map_err(|e: String| anyhow!(e))?;
                        if let Some(OscArg::String(name)) = name_msg.args.get(0) {
                            let file_type = parse_file_type(name);
                            file_tree.push(FileNode { name: name.clone(), file_type, index: i });
                        }
                    }
                }
            }
        }
    }
    Ok(file_tree)
}

fn change_directory(socket: &UdpSocket, index: i32) -> Result<()> {
    let cd_cmd = OscMessage::new("/-action/recselect".to_string(), vec![OscArg::Int(index)]).to_bytes().map_err(|e: String| anyhow!(e))?;
    socket.send(&cd_cmd)?;
    Ok(())
}

fn play_file(socket: &UdpSocket, index: i32) -> Result<()> {
    let play_cmd = OscMessage::new("/-action/recselect".to_string(), vec![OscArg::Int(index)]).to_bytes().map_err(|e: String| anyhow!(e))?;
    socket.send(&play_cmd)?;
    Ok(())
}

fn load_file(socket: &UdpSocket, index: i32) -> Result<()> {
    let load_cmd = OscMessage::new("/-action/recselect".to_string(), vec![OscArg::Int(index)]).to_bytes().map_err(|e: String| anyhow!(e))?;
    socket.send(&load_cmd)?;
    Ok(())
}

fn get_dir_name(file_tree: &[FileNode], index: i32) -> Option<&str> {
    file_tree.iter().find(|f| f.index == index).map(|f| f.name.as_str())
}

fn parse_file_type(name: &str) -> FileType {
    if name == "[..]" {
        FileType::Parent
    } else if name.starts_with('[') && name.ends_with(']') {
        FileType::Directory
    } else if name.to_lowercase().ends_with(".wav") {
        FileType::Wav
    } else {
        FileType::Unknown
    }
}

fn file_type_to_str(file_type: &FileType) -> &str {
    match file_type {
        FileType::Unknown => "Unk",
        FileType::Volume => "Vol",
        FileType::Parent => "Par",
        FileType::Directory => "Dir",
        FileType::Wav => "Wav",
        FileType::Show => "Shw",
        FileType::Scene => "Scn",
        FileType::Snippet => "Snp",
        FileType::Effect => "Eff",
        FileType::Preference => "Prf",
        FileType::Routing => "Rou",
        FileType::Channel => "Chn",
    }
}
