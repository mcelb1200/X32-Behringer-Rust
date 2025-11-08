# x32_geq2_cpy

`x32_geq2_cpy` is a command-line utility for copying and resetting Graphic EQ (GEQ) settings within and between FX slots on a Behringer X32 or Midas M32 digital mixing console. It is a Rust rewrite of the original `X32GEQ2cpy.c` program by Patrick-Gilles Maillot.

## Functionality

This tool allows you to:
- Copy the settings of a dual-channel GEQ from side A to side B.
- Copy the settings of a dual-channel GEQ from side B to side A.
- Reset all the bands of a GEQ to their default value (0.5).
- Copy the entire settings of one GEQ in one FX slot to another.
- Choose whether or not to include the master level controls in the copy/reset operation.

## Usage

```
x32_geq2_cpy [OPTIONS]
```

### Arguments

| Argument | Description | Default |
|---|---|---|
| `-i`, `--ip <IP>` | IP address of the X32 console. | `192.168.0.64` |
| `-f`, `--from <SLOT>` | Source FX slot number (1-8). | `1` |
| `-t`, `--to <SLOT>` | Destination FX slot number (1-8). Only used for `copy-to`. | `1` |
| `-d`, `--direction <DIR>`| The direction of the copy or reset operation. | `ato-b` |
| `-m`, `--master <BOOL>` | Include the master level in the copy/reset. | `true` |
| `-v`, `--verbose` | Enable verbose output. | `false` |
| `-D`, `--debug` | Enable debug output. | `false` |
| `-h`, `--help` | Print help information. | |
| `-V`, `--version` | Print version information. | |

### Directions

The `-d` or `--direction` argument accepts the following values:
- `ato-b`: Copies the settings from side A to side B of the GEQ in the source slot.
- `bto-a`: Copies the settings from side B to side A of the GEQ in the source slot.
- `reset`: Resets all bands on both sides of the GEQ in the source slot.
- `copy-to`: Copies the entire GEQ settings from the source slot to the destination slot.

### Examples

**1. Copy GEQ Side A to B**

Copy the GEQ settings from side A to side B in FX slot 1.

```bash
cargo run --bin x32_geq2_cpy -- --ip 192.168.1.100 --from 1 --direction ato-b
```

**2. Reset GEQ in Slot 4**

Reset all the bands of the GEQ in FX slot 4, including the master levels.

```bash
cargo run --bin x32_geq2_cpy -- --ip 192.168.1.100 --from 4 --direction reset
```

**3. Copy GEQ from Slot 1 to Slot 2**

Copy the entire GEQ from FX slot 1 to FX slot 2, without copying the master levels.

```bash
cargo run --bin x32_geq2_cpy -- --ip 192.168.1.100 --from 1 --to 2 --direction copy-to --master false
```
