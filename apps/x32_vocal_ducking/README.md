# X32 Vocal Ducking

Dynamic vocal ducking and spectral carver for Behringer X32 mixers.

## Modes

1. **Hybrid USB Mode (High-Precision)**
   Captures live vocal channel audio from the USB interface for sub-millisecond envelope tracking and FFT spectral peak carving. Sends dynamic EQ frequency and gain cuts to X32.
   ```bash
   cargo run --bin x32_vocal_ducking -- --ip <IP> --audio-device "X-USB" --card-channel 1 --instrument-bus 1
   ```

2. **OSC Mode (Ethernet Fallback)**
   Polls vocal bus meter level over Ethernet OSC, smoothing values with a moving average, and applies dynamic broadband EQ reduction to the instrument bus.
   ```bash
   cargo run --bin x32_vocal_ducking -- --ip <IP> --vocal-bus 2 --instrument-bus 1
   ```

## CLI Options

* `--ip`: X32 IP Address (default: `192.168.1.50`)
* `--audio-device`: Substring match for audio interface (USB Mode)
* `--card-channel`: Vocal input channel index on the USB interface (1-32)
* `--vocal-bus`: Vocal bus index on X32 to act as trigger (1-16)
* `--instrument-bus`: Target bus index to duck (1-16)
* `--threshold`: Ducking threshold in dBFS (default: `-35.0`)
* `--ratio`: Compression Ratio (default: `4.0`)
* `--attack`: Attack time in milliseconds (default: `15.0`)
* `--release`: Release time in milliseconds (default: `150.0`)
* `--max-duck`: Maximum gain reduction in dB (default: `6.0`)
* `--use-key-filter`: Tune sidechain key filter frequency based on vocal peak
* `--list-devices`: List available cpal audio input devices and exit
