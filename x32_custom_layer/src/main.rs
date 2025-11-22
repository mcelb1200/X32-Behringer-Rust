use clap::{Parser, Subcommand};
use osc_lib::{OscArg, OscMessage};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::net::UdpSocket;
use std::str::FromStr;
use x32_lib::{
    create_socket,
    error::{Result, X32Error},
};

const SNIP_HEAD: &str = "#2.1# \"CustLayer\" 8191 -1 255 0 1\n";

const SCH_NODES: [&str; 35] = [
    "/headamp/000",
    "/ch/01/config",
    "/ch/01/delay",
    "/ch/01/eq",
    "/ch/01/eq/1",
    "/ch/01/eq/2",
    "/ch/01/eq/3",
    "/ch/01/eq/4",
    "/ch/01/gate",
    "/ch/01/gate/filter",
    "/ch/01/dyn",
    "/ch/01/dyn/filter",
    "/ch/01/insert",
    "/ch/01/grp",
    "/ch/01/mix/fader",
    "/ch/01/mix/pan",
    "/ch/01/mix/on",
    "/ch/01/mix/01",
    "/ch/01/mix/02",
    "/ch/01/mix/03",
    "/ch/01/mix/04",
    "/ch/01/mix/05",
    "/ch/01/mix/06",
    "/ch/01/mix/07",
    "/ch/01/mix/08",
    "/ch/01/mix/09",
    "/ch/01/mix/10",
    "/ch/01/mix/11",
    "/ch/01/mix/12",
    "/ch/01/mix/13",
    "/ch/01/mix/14",
    "/ch/01/mix/15",
    "/ch/01/mix/16",
    "/ch/01/mix/mono",
    "/ch/01/mix/mlevel",
];

const SAUX_NODES: [&str; 29] = [
    "/headamp/000",
    "/auxin/01/config",
    "/auxin/01/eq",
    "/auxin/01/eq/1",
    "/auxin/01/eq/2",
    "/auxin/01/eq/3",
    "/auxin/01/eq/4",
    "/auxin/01/grp",
    "/auxin/01/mix/fader",
    "/auxin/01/mix/pan",
    "/auxin/01/mix/on",
    "/auxin/01/mix/01",
    "/auxin/01/mix/02",
    "/auxin/01/mix/03",
    "/auxin/01/mix/04",
    "/auxin/01/mix/05",
    "/auxin/01/mix/06",
    "/auxin/01/mix/07",
    "/auxin/01/mix/08",
    "/auxin/01/mix/09",
    "/auxin/01/mix/10",
    "/auxin/01/mix/11",
    "/auxin/01/mix/12",
    "/auxin/01/mix/13",
    "/auxin/01/mix/14",
    "/auxin/01/mix/15",
    "/auxin/01/mix/16",
    "/auxin/01/mix/mono",
    "/auxin/01/mix/mlevel",
];

const CH_INISTR: [&str; 35] = [
    "/headamp/000 +0.0 OFF",
    "/ch/01/config \"\" 1 YE 1",
    "/ch/01/delay OFF 0.3",
    "/ch/01/eq OFF",
    "/ch/01/eq/1 PEQ 124.7 +0.00 2.0",
    "/ch/01/eq/2 PEQ 496.6 +0.00 2.0",
    "/ch/01/eq/3 PEQ 1k97 +0.00 2.0",
    "/ch/01/eq/4 HShv 10k02 +0.00 2.0",
    "/ch/01/gate OFF GATE -80.0 60.0 1 502 983 0",
    "/ch/01/gate/filter OFF 3.0 990.9",
    "/ch/01/dyn OFF COMP PEAK LOG 0.0 3.0 1 0.00 10 10.0 151 POST 0 100 OFF",
    "/ch/01/dyn/filter OFF 3.0 990.9",
    "/ch/01/insert OFF POST OFF",
    "/ch/01/grp %00000000 %000000",
    "/ch/01/mix/fader -oo",
    "/ch/01/mix/pan +0",
    "/ch/01/mix/on ON",
    "/ch/01/mix/01 ON -oo +0 EQ->",
    "/ch/01/mix/02 ON -oo",
    "/ch/01/mix/03 ON -oo +0 EQ->",
    "/ch/01/mix/04 ON -oo",
    "/ch/01/mix/05 ON -oo +0 EQ->",
    "/ch/01/mix/06 ON -oo",
    "/ch/01/mix/07 ON -oo +0 EQ->",
    "/ch/01/mix/08 ON -oo",
    "/ch/01/mix/09 ON -oo +0 POST",
    "/ch/01/mix/10 ON -oo",
    "/ch/01/mix/11 ON -oo +0 POST",
    "/ch/01/mix/12 ON -oo",
    "/ch/01/mix/13 ON -oo +0 POST",
    "/ch/01/mix/14 ON -oo",
    "/ch/01/mix/15 ON -oo +0 POST",
    "/ch/01/mix/16 ON -oo",
    "/ch/01/mix/mono OFF",
    "/ch/01/mix/mlevel -oo",
];

const AUX_INISTR: [&str; 29] = [
    "/headamp/000 +0.0 OFF",
    "/auxin/01/config \"\" 55 GN 33",
    "/auxin/01/eq OFF",
    "/auxin/01/eq/1 PEQ 124.7 +0.00 2.0",
    "/auxin/01/eq/2 PEQ 496.6 +0.00 2.0",
    "/auxin/01/eq/3 PEQ 1k97 +0.00 2.0",
    "/auxin/01/eq/4 HShv 10k02 +0.00 2.0",
    "/auxin/01/grp %00000000 %000000",
    "/auxin/01/mix/fader -oo",
    "/auxin/01/mix/pan +0",
    "/auxin/01/mix/on ON",
    "/auxin/01/mix/01 ON -oo +0 PRE",
    "/auxin/01/mix/02 ON -oo",
    "/auxin/01/mix/03 ON -oo +0 PRE",
    "/auxin/01/mix/04 ON -oo",
    "/auxin/01/mix/05 ON -oo +0 PRE",
    "/auxin/01/mix/06 ON -oo",
    "/auxin/01/mix/07 ON -oo +0 PRE",
    "/auxin/01/mix/08 ON -oo",
    "/auxin/01/mix/09 ON -oo +0 POST",
    "/auxin/01/mix/10 ON -oo",
    "/auxin/01/mix/11 ON -oo +0 POST",
    "/auxin/01/mix/12 ON -oo",
    "/auxin/01/mix/13 ON -oo +0 POST",
    "/auxin/01/mix/14 ON -oo",
    "/auxin/01/mix/15 ON -oo +0 POST",
    "/auxin/01/mix/16 ON -oo",
    "/auxin/01/mix/mono OFF",
    "/auxin/01/mix/mlevel -oo",
];

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Sets a custom layer on the X32 mixer
    Set {
        /// The IP address of the X32 mixer
        #[clap(short, long)]
        ip: String,

        /// Channel assignments in the format "DEST=SRC DEST=SRC ..."
        #[clap(required = true)]
        assignments: Vec<String>,
    },
    /// Saves the current custom layer to a file
    Save {
        /// The IP address of the X32 mixer
        #[clap(short, long)]
        ip: String,

        /// The file to save the custom layer to
        #[clap(short, long)]
        file: String,
    },
    /// Restores a custom layer from a file
    Restore {
        /// The IP address of the X32 mixer
        #[clap(short, long)]
        ip: String,

        /// The file to restore the custom layer from
        #[clap(short, long)]
        file: String,
    },
    /// Resets channels to their default settings
    Reset {
        /// The IP address of the X32 mixer
        #[clap(short, long)]
        ip: String,

        /// A comma-separated list of channels to reset (e.g., "1,2,5-10")
        #[clap(required = true)]
        channels: String,
    },
    /// Lists the current channel sources
    List {
        /// The IP address of the X32 mixer
        #[clap(short, long)]
        ip: String,
    },
}

struct Assignment {
    dest: u8,
    src: u8,
}

fn parse_assignments(assignments_str: &[String]) -> Result<Vec<Assignment>> {
    let mut assignments = Vec::new();
    for a in assignments_str {
        let parts: Vec<&str> = a.split('=').collect();
        if parts.len() != 2 {
            return Err(X32Error::Custom(format!(
                "Invalid assignment format: {}",
                a
            )));
        }
        let dest = parts[0]
            .parse::<u8>()
            .map_err(|_| X32Error::Custom(format!("Invalid destination channel: {}", parts[0])))?;
        let src = parts[1]
            .parse::<u8>()
            .map_err(|_| X32Error::Custom(format!("Invalid source channel: {}", parts[1])))?;
        assignments.push(Assignment { dest, src });
    }
    Ok(assignments)
}

fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Set { ip, assignments } => handle_set_command(ip, assignments),
        Commands::Save { ip, file } => handle_save_command(ip, file),
        Commands::Restore { ip, file } => handle_restore_command(ip, file),
        Commands::Reset { ip, channels } => handle_reset_command(ip, channels),
        Commands::List { ip } => handle_list_command(ip),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn handle_set_command(ip: &str, assignments_str: &[String]) -> Result<()> {
    let socket = create_socket(ip, 200)?;
    let assignments = parse_assignments(assignments_str)?;
    let mut saved_strips: Vec<(u8, Vec<String>)> = Vec::new();

    println!("Saving states of source channels...");
    for a in &assignments {
        if !saved_strips.iter().any(|(src, _)| *src == a.src) {
            let mut strip_data = Vec::new();
            if a.src <= 32 {
                for &node in SCH_NODES.iter() {
                    let formatted_node = node.replace("/ch/01/", &format!("/ch/{:02}/", a.src));
                    strip_data.push(get_node_state(&socket, &formatted_node)?);
                }
            } else {
                let aux_channel = a.src - 32;
                for &node in SAUX_NODES.iter() {
                    let formatted_node =
                        node.replace("/auxin/01/", &format!("/auxin/{:02}/", aux_channel));
                    strip_data.push(get_node_state(&socket, &formatted_node)?);
                }
            }
            saved_strips.push((a.src, strip_data));
        }
    }

    println!("Applying new layer settings...");
    for a in &assignments {
        if let Some((_, strip_data)) = saved_strips.iter().find(|(src, _)| *src == a.src) {
            for line in strip_data {
                let mut new_line = line.clone();
                if a.dest <= 32 {
                    new_line = new_line.replace(
                        &format!("/ch/{:02}/", a.src),
                        &format!("/ch/{:02}/", a.dest),
                    );
                } else {
                    let aux_dest = a.dest - 32;
                    new_line = new_line.replace(
                        &format!("/auxin/{:02}/", a.src - 32),
                        &format!("/auxin/{:02}/", aux_dest),
                    );
                }
                let msg = OscMessage::from_str(&new_line)?;
                socket.send(&msg.to_bytes()?)?;
            }
        }
    }

    println!("Custom layer set successfully.");
    Ok(())
}

fn handle_save_command(ip: &str, file_path: &str) -> Result<()> {
    let socket = create_socket(ip, 200)?;
    let mut file = File::create(file_path)?;
    file.write_all(SNIP_HEAD.as_bytes())?;

    println!("Saving configuration to {}...", file_path);

    for i in 1..=40 {
        if i <= 32 {
            for &node in SCH_NODES.iter() {
                let formatted_node = node.replace("/ch/01/", &format!("/ch/{:02}/", i));
                let line = get_node_state(&socket, &formatted_node)?;
                file.write_all(line.as_bytes())?;
                file.write_all(b"\n")?;
            }
        } else {
            let aux_channel = i - 32;
            for &node in SAUX_NODES.iter() {
                let formatted_node =
                    node.replace("/auxin/01/", &format!("/auxin/{:02}/", aux_channel));
                let line = get_node_state(&socket, &formatted_node)?;
                file.write_all(line.as_bytes())?;
                file.write_all(b"\n")?;
            }
        }
    }

    println!("Save complete.");
    Ok(())
}

fn format_node_state(args: &[OscArg]) -> Result<String> {
    if args.is_empty() {
        return Err(X32Error::Custom("Empty node response".to_string()));
    }
    let mut s = if let OscArg::String(p) = &args[0] {
        p.clone()
    } else {
        return Err(X32Error::Custom(
            "Node response path is not a string".to_string(),
        ));
    };

    for arg in &args[1..] {
        s.push(' ');
        match arg {
            OscArg::Int(val) => s.push_str(&val.to_string()),
            OscArg::Float(val) => s.push_str(&val.to_string()),
            OscArg::String(val) => {
                if val.contains(' ') || val.is_empty() {
                    s.push_str(&format!("\"{}\"", val))
                } else {
                    s.push_str(val)
                }
            }
            OscArg::Blob(_) => todo!(),
        }
    }
    Ok(s)
}

fn get_node_state(socket: &UdpSocket, node: &str) -> Result<String> {
    let msg = OscMessage::new("/node".to_string(), vec![OscArg::String(node.to_string())]);
    socket.send(&msg.to_bytes()?)?;

    let mut buf = [0; 512];
    for _ in 0..10 {
        // Retry loop
        match socket.recv(&mut buf) {
            Ok(len) => {
                if len == 0 {
                    continue;
                }
                let response = OscMessage::from_bytes(&buf[..len])?;
                if response.path == "/node" {
                    if let Some(OscArg::String(response_node)) = response.args.first() {
                        if response_node == node {
                            return format_node_state(&response.args);
                        }
                    }
                }
            }
            Err(ref e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                // This is an expected timeout. Let the loop retry.
            }
            Err(e) => return Err(e.into()), // Other error
        }
    }

    Err(X32Error::Custom(format!(
        "Timeout waiting for response for node {}",
        node
    )))
}

fn handle_restore_command(ip: &str, file_path: &str) -> Result<()> {
    let socket = create_socket(ip, 200)?;
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    println!("Restoring configuration from {}...", file_path);

    for line in reader.lines() {
        let line = line?;
        if !line.starts_with('#') {
            let msg = OscMessage::from_str(&line)?;
            socket.send(&msg.to_bytes()?)?;
        }
    }

    println!("Restore complete.");
    Ok(())
}

fn handle_reset_command(ip: &str, channels_str: &str) -> Result<()> {
    let socket = create_socket(ip, 200)?;
    let channels = parse_channel_range(channels_str)?;

    for channel in channels {
        println!("Resetting channel {}...", channel);
        if channel >= 1 && channel <= 32 {
            for &cmd_str in CH_INISTR.iter() {
                let formatted_cmd = cmd_str.replace("/ch/01/", &format!("/ch/{:02}/", channel));
                let msg = OscMessage::from_str(&formatted_cmd)?;
                socket.send(&msg.to_bytes()?)?;
            }
        } else if channel >= 33 && channel <= 40 {
            let aux_channel = channel - 32;
            for &cmd_str in AUX_INISTR.iter() {
                let formatted_cmd =
                    cmd_str.replace("/auxin/01/", &format!("/auxin/{:02}/", aux_channel));
                let msg = OscMessage::from_str(&formatted_cmd)?;
                socket.send(&msg.to_bytes()?)?;
            }
        } else {
            return Err(X32Error::Custom(format!(
                "Invalid channel number: {}",
                channel
            )));
        }
    }

    println!("Channels reset successfully.");
    Ok(())
}

fn parse_channel_range(range_str: &str) -> Result<Vec<u8>> {
    let mut channels = Vec::new();
    for part in range_str.split(',') {
        if let Some(range) = part.split_once('-') {
            let start = range
                .0
                .trim()
                .parse::<u8>()
                .map_err(|_| X32Error::Custom("Invalid channel range format".to_string()))?;
            let end = range
                .1
                .trim()
                .parse::<u8>()
                .map_err(|_| X32Error::Custom("Invalid channel range format".to_string()))?;
            if start > end {
                return Err(X32Error::Custom(
                    "Invalid channel range: start > end".to_string(),
                ));
            }
            for i in start..=end {
                channels.push(i);
            }
        } else {
            let channel = part
                .trim()
                .parse::<u8>()
                .map_err(|_| X32Error::Custom("Invalid channel format".to_string()))?;
            channels.push(channel);
        }
    }
    channels.sort();
    channels.dedup();
    Ok(channels)
}

fn handle_list_command(ip: &str) -> Result<()> {
    let socket = create_socket(ip, 200)?;
    println!("  Channel\tSource\t\tChannel\t\tSource");

    for i in 0..16 {
        let ch1 = i + 1;
        let ch2 = i + 17;
        let src1 = get_source_name(&socket, ch1)?;
        let src2 = get_source_name(&socket, ch2)?;
        println!("     {:02}\t\t{}\t\t{:02}\t\t{}", ch1, src1, ch2, src2);
    }

    println!("\n  AuxIN\t\tSource\t\tAuxIN\t\tSource");
    for i in 0..4 {
        let aux1 = i + 33;
        let aux2 = i + 37;
        let src1 = get_source_name(&socket, aux1)?;
        let src2 = get_source_name(&socket, aux2)?;
        println!(
            "     {:02}\t\t{}\t\t{:02}\t\t{}",
            aux1 - 32,
            src1,
            aux2 - 32,
            src2
        );
    }

    Ok(())
}

fn get_source_name(socket: &UdpSocket, channel: u8) -> Result<String> {
    let (path, expected_response_prefix) = if (1..=32).contains(&channel) {
        (
            format!("/ch/{:02}/config/source", channel),
            format!("/ch/{:02}/config/source", channel),
        )
    } else if (33..=40).contains(&channel) {
        (
            format!("/auxin/{:02}/config/source", channel - 32),
            format!("/auxin/{:02}/config/source", channel - 32),
        )
    } else {
        return Err(X32Error::Custom(format!(
            "Invalid channel number: {}",
            channel
        )));
    };

    let msg = OscMessage::new(path, vec![]);
    socket.send(&msg.to_bytes()?)?;

    let mut buf = [0; 512];
    for _ in 0..10 {
        // Retry loop
        match socket.recv(&mut buf) {
            Ok(len) => {
                if len == 0 {
                    continue;
                }
                let response = OscMessage::from_bytes(&buf[..len])?;
                if response.path.starts_with(&expected_response_prefix) {
                    if let Some(OscArg::Int(source_id)) = response.args.first() {
                        return Ok(map_source_id_to_name(*source_id).to_string());
                    }
                }
            }
            Err(ref e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                // Expected timeout, retry.
            }
            Err(e) => return Err(e.into()),
        }
    }

    Err(X32Error::Custom(format!(
        "Timeout waiting for response for channel source {}",
        channel
    )))
}

fn map_source_id_to_name(id: i32) -> &'static str {
    match id {
        0..=31 => {
            const IN_NAMES: [&str; 32] = [
                "In01", "In02", "In03", "In04", "In05", "In06", "In07", "In08", "In09", "In10",
                "In11", "In12", "In13", "In14", "In15", "In16", "In17", "In18", "In19", "In20",
                "In21", "In22", "In23", "In24", "In25", "In26", "In27", "In28", "In29", "In30",
                "In31", "In32",
            ];
            IN_NAMES[id as usize]
        }
        32..=37 => {
            const AUX_NAMES: [&str; 6] = ["Aux1", "Aux2", "Aux3", "Aux4", "Aux5", "Aux6"];
            AUX_NAMES[(id - 32) as usize]
        }
        38..=39 => {
            if id == 38 {
                "USBL"
            } else {
                "USBR"
            }
        }
        40..=47 => {
            const FX_NAMES: [&str; 8] = [
                "Fx1L", "Fx1R", "Fx2L", "Fx2R", "Fx3L", "Fx3R", "Fx4L", "Fx4R",
            ];
            FX_NAMES[(id - 40) as usize]
        }
        48..=63 => {
            const BUS_NAMES: [&str; 16] = [
                "Bs01", "Bs02", "Bs03", "Bs04", "Bs05", "Bs06", "Bs07", "Bs08", "Bs09", "Bs10",
                "Bs11", "Bs12", "Bs13", "Bs14", "Bs15", "Bs16",
            ];
            BUS_NAMES[(id - 48) as usize]
        }
        _ => "OFF",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_assignments() {
        let input = vec!["1=2".to_string(), "10=32".to_string()];
        let result = parse_assignments(&input).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].dest, 1);
        assert_eq!(result[0].src, 2);
        assert_eq!(result[1].dest, 10);
        assert_eq!(result[1].src, 32);
    }

    #[test]
    fn test_parse_channel_range_single() {
        let result = parse_channel_range("1,5,10").unwrap();
        assert_eq!(result, vec![1, 5, 10]);
    }

    #[test]
    fn test_parse_channel_range_mixed() {
        let result = parse_channel_range("1,5-7,10").unwrap();
        assert_eq!(result, vec![1, 5, 6, 7, 10]);
    }

    #[test]
    fn test_parse_channel_range_duplicates() {
        let result = parse_channel_range("1,5-7,6,10").unwrap();
        assert_eq!(result, vec![1, 5, 6, 7, 10]);
    }

    #[test]
    fn test_parse_channel_range_invalid() {
        assert!(parse_channel_range("1,5-3,10").is_err());
        assert!(parse_channel_range("1,a-7,10").is_err());
        assert!(parse_channel_range("1,5=7,10").is_err());
    }
}
