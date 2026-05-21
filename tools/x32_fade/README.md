# x32_fade

`x32_fade` is a command-line utility for creating smooth, timed fader transitions on Behringer X32 and Midas M32 consoles. It allows you to fade one or more faders in or out over a specified duration, with control over the smoothness of the fade. This tool is a Rust rewrite of the original `X32Fade.c` utility by Patrick-Gilles Maillot.

## How It Works

The tool connects to the X32 and sends a sequence of OSC messages to incrementally adjust the specified fader(s) from their current position to the target level (either fully on or fully off). You can also save and load configurations to a JSON file, making it easy to repeat complex fade operations.

## Command-Line Arguments

| Argument          | Short Flag | Long Flag         | Default Value | Description                                                                                             |
| ----------------- | ---------- | ----------------- | ------------- | ------------------------------------------------------------------------------------------------------- |
| IP Address        | `-i`       | `--ip`            | (none)        | The IP address of the X32/M32 console. Can be omitted if loading a config file.                         |
| Fader Address     |            | `--fader`         | (none)        | An OSC fader address to control (e.g., `/ch/01/mix/fader`). Can be specified multiple times.                |
| Fade-In Time      |            | `--fade-in`       | (none)        | The duration of the fade-in in seconds (e.g., `2.5`).                                                     |
| Fade-Out Time     |            | `--fade-out`      | (none)        | The duration of the fade-out in seconds (e.g., `5.0`).                                                    |
| Steps             |            | `--steps`         | `32`          | The number of steps to use for the fade. A higher number will result in a smoother fade.                 |
| Load Config       |            | `--load-config`   | (none)        | The path to a JSON config file to load.                                                                 |
| Save Config       |            | `--save-config`   | (none)        | The path to save the current settings to a JSON file. If this is used, the program will save and exit without performing a fade. |
| Verbose           | `-v`       | `--verbose`       | (false)       | If specified, enables verbose output to show the OSC messages being sent.                               |
| Debug             | `-d`       | `--debug`         | (false)       | If specified, enables debug output.                                                                     |

## Example Usage

### Simple Fade-In

Fade in the fader for channel 1 over 3 seconds using 50 steps.
```bash
x32_fade --ip 192.168.1.64 --fader /ch/01/mix/fader --fade-in 3.0 --steps 50
```

### Fade-Out Multiple Faders

Fade out the faders for channel 1 and bus 1 simultaneously over 5 seconds.
```bash
x32_fade --ip 192.168.1.64 --fader /ch/01/mix/fader --fader /bus/01/mix/fader --fade-out 5.0
```

### Saving a Configuration

Save a configuration for later use. This command will create a file named `my_fade_config.json` and exit.
```bash
x32_fade --save-config my_fade_config.json --ip 192.168.1.64 --fader /ch/01/mix/fader --fader /ch/02/mix/fader --fade-in 4.0 --steps 100
```

### Loading a Configuration

Load the previously saved configuration and execute the fade.
```bash
x32_fade --load-config my_fade_config.json
```

### Overriding a Loaded Configuration

Load a configuration but override the fade-in time and add a new fader from the command line.
```bash
x32_fade --load-config my_fade_config.json --fade-in 1.0 --fader /mtx/01/mix/fader
```
