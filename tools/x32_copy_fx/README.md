# x32_copy_fx

`x32_copy_fx` is a command-line utility for managing effects (FX) presets on Behringer X32 and Midas M32 consoles. It allows you to copy settings between FX slots, reset slots to their default values, and copy parameters between the A and B sides of a stereo effect. This tool is a Rust rewrite of the original `X32CopyFX.c` utility by Patrick-Gilles Maillot.

## How It Works

The tool connects to the X32 over the network and sends a series of OSC (Open Sound Control) messages to perform the requested actions. It can use a built-in set of default values for each effect type, or you can provide a custom defaults file for personalized reset actions.

## Command-Line Arguments

| Argument          | Short Flag | Long Flag         | Default Value | Description                                                                                             |
| ----------------- | ---------- | ----------------- | ------------- | ------------------------------------------------------------------------------------------------------- |
| IP Address        | `-i`       | `--ip`            | (none)        | **Required.** The IP address of the X32/M32 console.                                                       |
| Source Slot       | `-f`       | `--from`          | (none)        | **Required.** The source FX slot (1-8) for the operation.                                                  |
| Destination Slot  | `-t`       | `--to`            | (none)        | The destination FX slot (1-8) for `copy` operations.                                                    |
| Defaults File     | `-d`       | `--defaults-file` | (none)        | An optional path to a custom FX defaults file. If not provided, the tool uses built-in default values.      |

## Commands

### `copy`

Copies all parameters from a source FX slot to a destination slot.

-   `--to <TO>`: **Required.** The destination FX slot.
-   `--master`: If specified, includes master parameters in the copy.

### `reset`

Resets all parameters of the source FX slot to their default values.

### `a-to-b`

Copies parameters from side A to side B within a dual-mono or stereo effect in the source slot.

-   `--master`: If specified, includes master parameters in the copy.

### `b-to-a`

Copies parameters from side B to side A within a dual-mono or stereo effect in the source slot.

-   `--master`: If specified, includes master parameters in the copy.

## Example Usage

### Copy FX from Slot 1 to Slot 2

```bash
x32_copy_fx --ip 192.168.1.64 --from 1 --to 2 copy
```

### Copy FX from Slot 3 to Slot 4, Including Master Parameters

```bash
x32_copy_fx --ip 192.168.1.64 --from 3 --to 4 copy --master
```

### Reset FX Slot 5 to its Default Values

```bash
x32_copy_fx --ip 192.168.1.64 --from 5 reset
```

### Reset FX Slot 6 Using a Custom Defaults File

```bash
x32_copy_fx --ip 192.168.1.64 --from 6 --defaults-file /path/to/my_defaults.txt reset
```

### Copy from Side A to Side B of the Effect in Slot 7

```bash
x32_copy_fx --ip 192.168.1.64 --from 7 a-to-b
```

## Custom Defaults File Format

A custom defaults file allows you to define your own default parameter values for different FX types. The file should contain pairs of lines:

1.  The first line of a pair is the FX name (e.g., `HALL`, `AMBI`).
2.  The second line is a space-separated string of the default parameter values.

### Example:

```
HALL
20 1.57 60 5k74 25 0.0 83 7k2 0.95 25 50 30

AMBI
4 0.84 60 5k06 30 0.0 71 7k9 20 50
```
