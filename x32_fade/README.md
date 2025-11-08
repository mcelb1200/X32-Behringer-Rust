# x32_fade

`x32_fade` is a command-line utility for controlling and fading faders on a Behringer X32 or Midas M32 digital mixing console. It is a Rust rewrite of the original `X32Fade.c` program by Patrick-Gilles Maillot.

## Functionality

This tool allows you to:
- Fade one or more faders in or out over a specified duration.
- Control the number of steps in the fade for smoothness.
- Save your settings to a JSON configuration file.
- Load settings from a configuration file for repeatable operations.

## Usage

```
x32_fade [OPTIONS]
```

### Arguments

| Argument | Description |
|---|---|
| `-i`, `--ip <IP>` | IP address of the X32 console. Can be omitted if a config file is loaded. |
| `--fader <FADER>` | An OSC fader address to control. Can be specified multiple times. E.g., `--fader /ch/01/mix/fader`. |
| `--fade-in <SECONDS>` | The duration of the fade-in in seconds (e.g., `2.5`). |
| `--fade-out <SECONDS>`| The duration of the fade-out in seconds (e.g., `5.0`). |
| `--steps <STEPS>` | The number of steps to use for the fade. Defaults to `32`. |
| `--load-config <PATH>`| Path to a JSON config file to load. |
| `--save-config <PATH>`| Path to save the current settings to a JSON file. If this is used, the program will save and exit without performing a fade. |
| `-v`, `--verbose` | Enable verbose output to see the OSC messages being sent. |
| `-d`, `--debug` | Enable debug output. |
| `-h`, `--help` | Print help information. |
| `-V`, `--version` | Print version information. |

### Examples

**1. Simple Fade-In**

Fade in the fader for channel 1 over 3 seconds using 50 steps.

```bash
cargo run --bin x32_fade -- --ip 192.168.1.100 --fader /ch/01/mix/fader --fade-in 3.0 --steps 50
```

**2. Fade-Out Multiple Faders**

Fade out the faders for channel 1 and bus 1 simultaneously over 5 seconds.

```bash
cargo run --bin x32_fade -- --ip 192.168.1.100 --fader /ch/01/mix/fader --fader /bus/01/mix/fader --fade-out 5.0
```

**3. Saving a Configuration**

Save a configuration for later use. This command will create a file named `my_fade_config.json` and exit.

```bash
cargo run --bin x32_fade -- --save-config my_fade_config.json --ip 192.168.1.100 --fader /ch/01/mix/fader --fader /ch/02/mix/fader --fade-in 4.0 --steps 100
```

**4. Loading a Configuration**

Load the previously saved configuration and execute the fade.

```bash
cargo run --bin x32_fade -- --load-config my_fade_config.json
```

**5. Overriding a Loaded Configuration**

Load a configuration but override the fade-in time and faders from the command line.

```bash
cargo run --bin x32_fade -- --load-config my_fade_config.json --fade-in 1.0 --fader /mtx/01/mix/fader
```
