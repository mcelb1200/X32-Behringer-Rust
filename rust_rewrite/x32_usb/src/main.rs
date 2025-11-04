
use anyhow::{anyhow, Result};
use clap::Parser;
use std::io::{self, Write};
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;
use x32_lib::cparse;

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

    let info_cmd = cparse::xcparse("/info").map_err(|e| anyhow!(e))?;
    loop {
        socket.send(&info_cmd)?;
        let mut buf = [0; 512];
        if let Ok(len) = socket.recv(&mut buf) {
            if &buf[..len] == b"/info" {
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
                        prompt.push_str(&format!("{}/", get_dir_name(&file_tree, index).unwrap_or("")));
                    } else {
                        if let Some(file) = file_tree.iter().find(|f| f.name == arg && f.file_type == FileType::Directory) {
                            change_directory(&socket, file.index)?;
                            prompt.push_str(&format!("{}/", file.name));
                        } else {
                            println!("Directory not found!");
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
            "exit" | "quit" => break,
            _ => {
                 if !mounted {
                    let mounted_cmd = cparse::xcparse("/-stat/usbmounted").map_err(|e| anyhow!(e))?;
                    socket.send(&mounted_cmd)?;
                    let mut buf = [0; 512];
                    if let Ok(len) = socket.recv(&mut buf) {
                        if &buf[..len] == b"/-stat/usbmounted" {
                            if buf[24..28] == [0, 0, 0, 1] {
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
    let maxpos_cmd = cparse::xcparse("/-usb/dir/maxpos").map_err(|e| anyhow!(e))?;
    socket.send(&maxpos_cmd)?;
    let mut buf = [0; 512];
    if let Ok(len) = socket.recv(&mut buf) {
        if &buf[..len] == b"/-usb/dir/maxpos" {
            let num_files = i32::from_be_bytes([buf[24], buf[25], buf[26], buf[27]]);
            for i in 1..=num_files {
                let name_cmd = cparse::xcparse(&format!("/-usb/dir/{:03}/name", i)).map_err(|e| anyhow!(e))?;
                socket.send(&name_cmd)?;
                if let Ok(len) = socket.recv(&mut buf) {
                    if let Some(start) = buf.iter().position(|&b| b == 0) {
                        let name = String::from_utf8_lossy(&buf[start+1..len]).to_string();
                        let file_type = parse_file_type(&name);
                        file_tree.push(FileNode { name, file_type, index: i });
                    }
                }
            }
        }
    }
    Ok(file_tree)
}

fn change_directory(socket: &UdpSocket, index: i32) -> Result<()> {
    let cd_cmd = cparse::xcparse(&format!("/-action/recselect,i,{}", index)).map_err(|e| anyhow!(e))?;
    socket.send(&cd_cmd)?;
    Ok(())
}

fn play_file(socket: &UdpSocket, index: i32) -> Result<()> {
    let play_cmd = cparse::xcparse(&format!("/-action/recselect,i,{}", index)).map_err(|e| anyhow!(e))?;
    socket.send(&play_cmd)?;
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
