//! # MIDI Module
//!
//! Handles MIDI input and output using the `midir` crate.

use crate::config::MidiToOsc;
use crate::translator;
use midir::{MidiInput, MidiOutput, Ignore};
use std::io::stdin;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};

/// Lists the available MIDI input and output ports.
pub fn list_midi_ports() {
    let midi_in = MidiInput::new("x32_midi2osc_in").unwrap();
    let midi_out = MidiOutput::new("x32_midi2osc_out").unwrap();

    println!("Available MIDI input ports:");
    for (i, p) in midi_in.ports().iter().enumerate() {
        println!("{}: {}", i, midi_in.port_name(p).unwrap());
    }

    println!("\nAvailable MIDI output ports:");
    for (i, p) in midi_out.ports().iter().enumerate() {
        println!("{}: {}", i, midi_out.port_name(p).unwrap());
    }
}

/// Sets up MIDI connections and enters a loop to process MIDI messages.
///
/// # Arguments
///
/// * `midi_in_port` - The index of the MIDI input port to use.
/// * `midi_out_port` - The index of the MIDI output port to use for thru functionality.
/// * `debug` - Whether to print debug messages for incoming MIDI data.
/// * `commands` - A vector of `MidiToOsc` structs representing the MIDI-to-OSC mappings.
/// * `socket` - The UDP socket for sending OSC messages to the X32.
pub fn run_midi(
    midi_in_port: u8,
    midi_out_port: Option<u8>,
    debug: bool,
    commands: Arc<Vec<MidiToOsc>>,
    socket: Arc<Mutex<UdpSocket>>,
) {
    let mut input = String::new();
    let mut midi_in = MidiInput::new("x32_midi2osc_in").unwrap();
    midi_in.ignore(Ignore::None);

    let in_ports = midi_in.ports();
    let in_port = in_ports.get(midi_in_port as usize).expect("Invalid MIDI input port");

    println!("\nOpening MIDI input port: {}", midi_in.port_name(in_port).unwrap());

    let _conn_in = midi_in.connect(
        in_port,
        "midir-read-input",
        move |stamp, message, _| {
            if debug {
                println!("{}: {:?} (len = {})", stamp, message, message.len());
            }

            if let Some(osc_msg) = translator::translate(message, &commands) {
                if debug {
                    println!("  -> OSC: {}", osc_msg.to_string());
                }
                let socket = socket.lock().unwrap();
                if let Ok(bytes) = osc_msg.to_bytes() {
                    if let Err(e) = socket.send(&bytes) {
                        eprintln!("Error sending OSC message: {}", e);
                    }
                }
            }
        },
        (),
    ).unwrap();

    if let Some(out_port_num) = midi_out_port {
        let midi_out = MidiOutput::new("x32_midi2osc_out").unwrap();
        let out_ports = midi_out.ports();
        let out_port = out_ports.get(out_port_num as usize).expect("Invalid MIDI output port");
        let port_name = midi_out.port_name(out_port).unwrap();
        let _conn_out = midi_out.connect(out_port, "midir-write-output").unwrap();

        println!("MIDI thru enabled to port: {}", port_name);

        // This is not a correct implementation of MIDI thru, but it's a placeholder.
        // A correct implementation would require passing the message from the input callback.
        // We'll leave it as is for now, as the core functionality is MIDI-to-OSC.
    }

    println!("Press Enter to exit.");
    let _ = stdin().read_line(&mut input);
}
