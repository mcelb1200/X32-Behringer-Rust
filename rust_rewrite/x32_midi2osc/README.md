# x32_midi2osc

`x32_midi2osc` is a command-line utility that translates MIDI commands to OSC messages for the Behringer X32/M32 consoles. It is a Rust rewrite of the original `X32Midi2OSC.c` tool by Patrick-Gilles Maillot.

## Usage

```
x32_midi2osc [OPTIONS] --ip <IP> --midi-in <MIDI_IN> --config <CONFIG>
```

## Options

- `-i, --ip <IP>`: IP address of the X32 console
- `--midi-in <MIDI_IN>`: MIDI input port number
- `--midi-out <MIDI_OUT>`: MIDI output port number for thru functionality
- `-c, --config <CONFIG>`: Path to the configuration file
- `-d, --debug`: Enable debug mode to print MIDI messages
- `--list-midi`: List available MIDI ports and exit

## Configuration File

The configuration file defines the mapping between MIDI messages and OSC commands. Each line in the file represents a single mapping, in the following format:

```
<MIDI_COMMAND> <MIDI_CHANNEL> <MIDI_DATA1> <MIDI_DATA2> | <OSC_COMMAND>
```

- `<MIDI_COMMAND>`: The MIDI command byte in hexadecimal (e.g., `90` for Note On).
- `<MIDI_CHANNEL>`: The MIDI channel (1-16, or 0 for any).
- `<MIDI_DATA1>`: The first MIDI data byte (0-127, or 0 for any).
- `<MIDI_DATA2>`: The second MIDI data byte (0-127, or 0 for any).
- `<OSC_COMMAND>`: The OSC command string to be sent.

### OSC Command Placeholders

The OSC command string can contain the following placeholders, which will be replaced with the corresponding MIDI data:

- `$d1`: MIDI data byte 1
- `$d2`: MIDI data byte 2
- `$ch`: MIDI channel

### Example

```
# Mute channel 1 when Note On on channel 1, note 60 is received
90 1 60 0 | /ch/01/mix/on i 0
```
