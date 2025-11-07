# X32 Punch Control

A utility to manage DAW punch IN/OUT down mixing updates for the Behringer X32.

This is a Rust rewrite of the original C program by Patrick-Gilles Maillot.

## Description

`x32_punch_control` is a command-line tool that allows for recording and playback of automation data for an X32 digital mixer, synchronized with a Digital Audio Workstation (DAW). It can be controlled via MIDI messages for transport control (play, stop, record, etc.) and uses MIDI Time Code (MTC) for synchronization.

The tool reads automation data from an input file (`.xpc`) and merges it with live changes from the mixer, writing the result to an output file. This allows for non-destructive automation recording and playback workflows.

## Usage

```sh
x32_punch_control --ip <X32_IP_ADDRESS> --output-file <OUTPUT_FILE> [OPTIONS]
```

## Options

- `-i, --ip <IP>`: IP address of the X32 mixer.
- `-f, --input-file <FILE>`: Input file to read automation data from (`.xpc`).
- `-o, --output-file <FILE>`: Output file to write automation data to (`.xpc`).
- `-s, --scene <SCENE>`: Scene number to load on the X32 at startup.
- `--midi-in <PORT>`: MIDI input port number.
- `--midi-out <PORT>`: MIDI output port number.
- `--use-bank <BANK>`: Enable and specify which user bank to use for hardware controls (A, B, or C).
- `--mtc`: Use MIDI Time Code for synchronization.
- `--record-implies-play`: The Record button will also trigger Play.
- `--merge-faders-only`: When merging, only protect faders from being overwritten by file data.
- `--catch-delay <DELAY>`: Delay in milliseconds used during catch-up/catch-back operations.

## MIDI Control

The application can be controlled by MIDI transport messages:
- **Start (0xFA):** Starts playback.
- **Stop (0xFC):** Stops playback and recording.
- **Continue (0xFB):** Resumes playback if paused.

When the `--mtc` flag is used, the application will synchronize to incoming MIDI Time Code.
