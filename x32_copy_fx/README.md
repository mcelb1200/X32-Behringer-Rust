# X32 Copy FX

`x32_copy_fx` is a command-line utility for copying and resetting FX parameters on Behringer X32 and X-Air digital mixers. It is a Rust rewrite of the original `X32CopyFX.c` tool by Patrick-Gilles Maillot.

## Usage

```
x32_copy_fx [OPTIONS] --ip <IP> --from <FROM> <COMMAND>
```

### Options

* `-i`, `--ip <IP>`: IP address of the X32/X-Air console.
* `-d`, `--defaults-file <DEFAULTS_FILE>`: Path to a custom FX defaults file.
* `-f`, `--from <FROM>`: Source FX slot (1-8).
* `-t`, `--to <TO>`: Destination FX slot (1-8).

### Commands

* `reset`: Resets the source FX slot to its default values.
* `copy`: Copies all parameters from a source FX slot to a destination slot.
    * `--master`: Include master parameters in the copy.
* `a-to-b`: Copies parameters from side A to side B within a stereo effect.
    * `--master`: Include master parameters in the copy.
* `b-to-a`: Copies parameters from side B to side A within a stereo effect.
    * `--master`: Include master parameters in the copy.

## Examples

### Copy FX from slot 1 to slot 2

```
x32_copy_fx --ip 192.168.1.10 --from 1 --to 2 copy
```

### Copy FX from slot 1 to slot 2, including master parameters

```
x32_copy_fx --ip 192.168.1.10 --from 1 --to 2 copy --master
```

### Reset FX slot 3 to its default values

```
x32_copy_fx --ip 192.168.1.10 --from 3 reset
```

### Reset FX slot 3 using a custom defaults file

```
x32_copy_fx --ip 192.168.1.10 --from 3 --defaults-file /path/to/my_defaults.txt reset
```

### Copy from side A to side B of FX slot 5

```
x32_copy_fx --ip 192.168.1.10 --from 5 a-to-b
```

## Custom Defaults File Format

The custom defaults file should contain pairs of lines. The first line of a pair is the FX name (e.g., `HALL`), and the second line is a space-separated string of default parameter values.

Example:

```
HALL
20 1.57 60 5k74 25 0.0 83 7k2 0.95 25 50 30
AMBI
4 0.84 60 5k06 30 0.0 71 7k9 20 50
```

## Attribution

This tool is a rewrite of the original `X32CopyFX.c` by Patrick-Gilles Maillot.
