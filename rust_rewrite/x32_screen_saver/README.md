# X32 Screen Saver

`x32_screen_saver` is a command-line utility that connects to a Behringer X32 or Midas M32
digital mixer and activates a screen saver mode after a period of inactivity. When the
screen saver is active, the mixer's screen and LED brightness are turned down to their
minimum values. Any activity on the mixer will restore the original brightness levels.

This tool is a Rust rewrite of the original `X32SsaverGW.c` utility by
Patrick-Gilles Maillot.

## Usage

```
x32_screen_saver [OPTIONS]
```

### Options

- `-i`, `--ip <IP>`: The IP address of the X32/M32 console.
- `-d`, `--delay <DELAY>`: The delay in seconds before the screen saver activates.
  Defaults to 5 seconds.
- `--save-config <SAVE_CONFIG>`: Save the current configuration to a file.
- `--load-config <LOAD_CONFIG>`: Load configuration from a file.
- `-h`, `--help`: Print help information.
- `-V`, `--version`: Print version information.

## Example

```
x32_screen_saver --ip 192.168.1.62 --delay 10
```
