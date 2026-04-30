# x32_ssavergw

`x32_ssavergw` is a command-line tool that acts as a screen saver for the Behringer X32/M32.

It monitors the X32 console for activity via OSC. If no activity is detected for a specified amount of time, it saves the current brightness settings for the main LCD screen and LEDs, and then sets them to their lowest values (0.0). When activity is detected again, it restores the original brightness settings.

This is a Rust rewrite of the original `X32SsaverGW.c` tool by Patrick-Gilles Maillot.

## How it works
The tool establishes a UDP connection with the X32 and keeps it alive by sending `/xremote` commands. Any incoming OSC message (other than the tool's own requests) is considered activity.

If the configured delay passes without any activity:
1. The tool reads the current `/-prefs/bright` and `/-prefs/ledbright` values.
2. It sets both values to `0.0` (off/dimmed).

When the tool detects a new OSC message (meaning someone interacted with the console):
1. It restores the saved `/-prefs/bright` and `/-prefs/ledbright` values.

## Usage

```
Usage: x32_ssavergw [OPTIONS]

Options:
  -i, --ip <IP>        The IP address of the X32 console [default: 192.168.0.64]
  -d, --delay <DELAY>  Delay in seconds before entering Low Light mode [default: 5]
  -h, --help           Print help
  -V, --version        Print version
```

## Example

To run the screen saver, connecting to an X32 at `10.0.0.10` and dimming the screens after `30` seconds of inactivity:

```bash
x32_ssavergw -i 10.0.0.10 -d 30
```
