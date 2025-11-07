# x32_set_preset

`x32_set_preset` is a command-line tool for loading channel, effect, and routing presets to a Behringer X32 or Midas M32 digital mixing console.

## Usage

```
x32_set_preset [OPTIONS] --ip <IP> --file <FILE>
```

## Options

*   `--ip <IP>`: The IP address of the X32/M32 console.
*   `--port <PORT>`: The port of the X32/M32 console (default: 10023).
*   `-f, --file <FILE>`: The preset file to load. The file extension determines the preset type (`.chn` for channel, `.efx` for effect, `.rou` for routing).
*   `-s, --slot <SLOT>`: The destination slot for the preset (default: 1). The valid range and meaning of the slot number depends on the preset type.
*   `--master-safe`: When set, the master fader levels will be set to 0 before loading the preset.
*   `--preamp-safe`: When set, preamp settings (gain, phantom power) will not be loaded from a channel preset.
*   `--config-safe`: When set, channel configuration settings will not be loaded from a channel preset.
*   `--gate-safe`: When set, gate settings will not be loaded from a channel preset.
*   `--dynamics-safe`: When set, dynamics settings will not be loaded from a channel preset.
*   `--eq-safe`: When set, EQ settings will not be loaded from a channel preset.
*   `--sends-safe`: When set, send settings will not be loaded from a channel preset.
*   `-v, --verbose`: Enables verbose output, printing each OSC message as it is sent.
*   `-h, --help`: Prints help information.
*   `-V, --version`: Prints version information.
