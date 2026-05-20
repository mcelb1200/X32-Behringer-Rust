# `x32_midi2osc`

A flexible MIDI-to-OSC translation engine for Behringer X32 and Midas M32 consoles.

## 🚀 Features

*   **Template Engine:** Supports complex string and variable interpolation using a Reverse Polish Notation (RPN) math engine.
*   **MTC Support:** Can respond to MIDI Time Code for time-locked command triggers.
*   **Configurable Mapping:** Easily map Note On/Off, Control Change (CC), and Program Change messages to any X32 OSC endpoint.

## 🛠️ Usage

```bash
# Start translation using a configuration file
./x32_midi2osc --ip 192.168.1.50 --config my_midi_setup.json
```

### Configuration Example
```json
{
  "midi_port": "X32 MIDI",
  "mappings": [
    { "type": "NoteOn", "note": 60, "path": "/ch/01/mix/on", "value": 1 }
  ]
}
```

## 🧪 Testing
Run translation engine tests with:
```bash
cargo test -p x32_midi2osc
```
