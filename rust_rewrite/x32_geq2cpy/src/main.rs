
use clap::{Parser, ValueEnum};
use std::net::UdpSocket;
use osc_lib::{OscMessage, OscArg};
use x32_lib::{create_socket, get_fx_type, Result};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The IP address of the X32 console.
    #[arg(short, long, default_value = "192.168.0.64")]
    ip: String,

    /// Source FX slot number.
    #[arg(short, long, default_value_t = 1)]
    from: u8,

    /// Destination FX slot number.
    #[arg(short, long, default_value_t = 1)]
    to: u8,

    /// Operation to perform.
    #[arg(short, long, value_enum, default_value_t = CopyDirection::AtoB)]
    direction: CopyDirection,

    /// Include master level in the copy/reset.
    #[arg(short, long, default_value_t = true)]
    master: bool,
}

#[derive(Debug, Clone, ValueEnum)]
enum CopyDirection {
    AtoB,
    BtoA,
    Reset,
    Copy,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let socket = create_socket(&args.ip, 100)?;

    println!("# X32GEQ2cpy V1.3 (c)2014 Patrick-Gilles Maillot\n");

    // Verify that the source and destination slots are valid GEQ/TEQ slots
    verify_fx_slot(&socket, args.from)?;
    if let CopyDirection::Copy = args.direction {
        verify_fx_slot(&socket, args.to)?;
    }

    match args.direction {
        CopyDirection::AtoB => copy_side(&socket, args.from, true, args.master)?,
        CopyDirection::BtoA => copy_side(&socket, args.from, false, args.master)?,
        CopyDirection::Reset => reset_eq(&socket, args.from, args.master)?,
        CopyDirection::Copy => copy_slot(&socket, args.from, args.to, args.master)?,
    }

    Ok(())
}

fn verify_fx_slot(socket: &UdpSocket, slot: u8) -> Result<()> {
    let fx_type = get_fx_type(socket, slot)?;
    if fx_type != 18 && fx_type != 19 { // GEQ2 and TEQ2
        return Err(format!("No GEQ2/TEQ2 effect at FX slot #{}", slot).into());
    }
    Ok(())
}

fn copy_side(socket: &UdpSocket, slot: u8, a_to_b: bool, master: bool) -> Result<()> {
    let (src_start, dest_start) = if a_to_b { (1, 33) } else { (33, 1) };
    for i in 0..31 {
        let src_param = src_start + i;
        let dest_param = dest_start + i;
        copy_param(socket, slot, slot, src_param, dest_param)?;
    }
    if master {
        let (src_master, dest_master) = if a_to_b { (32, 64) } else { (64, 32) };
        copy_param(socket, slot, slot, src_master, dest_master)?;
    }
    Ok(())
}

fn reset_eq(socket: &UdpSocket, slot: u8, master: bool) -> Result<()> {
    for i in 1..=31 {
        set_param(socket, slot, i, 0.5)?;
        set_param(socket, slot, i + 32, 0.5)?;
    }
    if master {
        set_param(socket, slot, 32, 0.5)?;
        set_param(socket, slot, 64, 0.5)?;
    }
    Ok(())
}

fn copy_slot(socket: &UdpSocket, from: u8, to: u8, master: bool) -> Result<()> {
    for i in 1..=31 {
        copy_param(socket, from, to, i, i)?;
        copy_param(socket, from, to, i + 32, i + 32)?;
    }
    if master {
        copy_param(socket, from, to, 32, 32)?;
        copy_param(socket, from, to, 64, 64)?;
    }
    Ok(())
}

fn copy_param(socket: &UdpSocket, from_slot: u8, to_slot: u8, from_param: u8, to_param: u8) -> Result<()> {
    let msg = OscMessage::new(format!("/fx/{}/par/{:02}", from_slot, from_param), vec![]);
    socket.send(&msg.to_bytes()?)?;
    let mut buf = [0; 512];
    let len = socket.recv(&mut buf)?;
	let response = OscMessage::from_bytes(&buf[..len])?;
	if let Some(arg) = response.args.get(0) {
		let reply = OscMessage::new(format!("/fx/{}/par/{:02}", to_slot, to_param), vec![arg.clone()]);
		socket.send(&reply.to_bytes()?)?;
	}
    Ok(())
}

fn set_param(socket: &UdpSocket, slot: u8, param: u8, value: f32) -> Result<()> {
    let msg = OscMessage::new(format!("/fx/{}/par/{:02}", slot, param), vec![OscArg::Float(value)]);
    socket.send(&msg.to_bytes()?)?;
    Ok(())
}
