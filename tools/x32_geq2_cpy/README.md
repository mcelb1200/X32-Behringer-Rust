# x32_geq2_cpy

`x32_geq2_cpy` is a command-line utility for managing Graphic EQ (GEQ) settings on Behringer X32 and Midas M32 consoles. It provides a simple and efficient way to copy settings within and between FX slots, as well as reset GEQs to their default state. This tool is a Rust rewrite of the original `X32GEQ2cpy.c` utility by Patrick-Gilles Maillot.

## How It Works

The tool connects to the X32 and sends a series of OSC (Open Sound Control) messages to read and write the parameters of the GEQs loaded into the FX slots. Before performing any action, it verifies that the target FX slot actually contains a GEQ effect.

## Command-Line Arguments

| Argument   | Short Flag | Long Flag   | Default Value  | Description                                                                 |
| ---------- | ---------- | ----------- | -------------- | --------------------------------------------------------------------------- |
| IP Address | `-i`       | `--ip`      | `192.168.0.64` | The IP address of the X32/M32 console.                                        |
| From Slot  | `-f`       | `--from`    | `1`            | The source FX slot number (1-8).                                            |
| To Slot    | `-t`       | `--to`      | `1`            | The destination FX slot number (1-8). Only used with the `copy-to` direction. |
| Direction  | `-d`       | `--direction`| `ato-b`        | The copy or reset operation to perform. See directions below.               |
| Master     | `-m`       | `--master`  | `true`         | A boolean (`true` or `false`) indicating whether to include the master level controls in the operation. |
| Verbose    | `-v`       | `--verbose` | `false`        | If specified, enables verbose output.                                       |

## Directions

The `--direction` argument accepts the following values:

| Value     | Description                                                          |
| --------- | -------------------------------------------------------------------- |
| `ato-b`   | Copies the settings from side A to side B of the GEQ in the source slot. |
| `bto-a`   | Copies the settings from side B to side A of the GEQ in the source slot. |
| `reset`   | Resets all bands on both sides of the GEQ in the source slot.        |
| `copy-to` | Copies the entire GEQ settings from the source slot to the destination slot. |

## Example Usage

### Copy GEQ Settings from Side A to B

To copy the GEQ settings from side A to side B in FX slot 1:
```bash
x32_geq2_cpy --ip 192.168.1.64 --from 1 --direction ato-b
```

### Reset a GEQ

To reset all the bands of the GEQ in FX slot 4, including the master levels:
```bash
x32_geq2_cpy --ip 192.168.1.64 --from 4 --direction reset
```

### Copy a GEQ to a Different Slot

To copy the entire GEQ from FX slot 2 to FX slot 3, without copying the master levels:
```bash
x32_geq2_cpy --ip 192.168.1.64 --from 2 --to 3 --direction copy-to --master false
```
