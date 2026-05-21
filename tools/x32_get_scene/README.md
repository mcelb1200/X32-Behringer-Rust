# x32_get_scene

`x32_get_scene` is a command-line utility for retrieving scene data from a Behringer X32 or Midas M32 console. It queries the mixer for the values of specified OSC paths and prints them to the console in a format that can be saved as a scene file. This tool is a Rust rewrite of the original `X32GetScene.c` utility by Patrick-Gilles Maillot.

## How It Works

The tool connects to the X32 and reads a list of OSC paths from standard input. For each path, it sends a `/node` query to the mixer and prints the response. The output is formatted with a scene header, making it easy to create `.scn` files that can be loaded back onto the console using `x32_desk_restore`.

## Command-Line Arguments

| Argument   | Short Flag | Long Flag    | Default Value  | Description                                            |
| ---------- | ---------- | ------------ | -------------- | ------------------------------------------------------ |
| IP Address | `-i`       | `--ip`       | `192.168.0.64` | The IP address of the X32/M32 console.                 |
| Scene Name | `-s`       | `--scene-name` | (none)         | The name of the scene. If not provided, you will be prompted to enter it. |
| Note       | `-n`       | `--note`     | (none)         | A descriptive note for the scene. If not provided, you will be prompted to enter it. |

## Example Usage

To use the tool, you will need a file containing the OSC paths you want to query. For example, you could create a file named `my_pattern.txt` with the following content:

```
/ch/01/config/name
/ch/01/mix/fader
/ch/02/config/name
/ch/02/mix/fader
```

You can then run the tool and pipe the contents of this file to its standard input:

```bash
cat my_pattern.txt | x32_get_scene --ip 192.168.1.64 --scene-name "My Band" --note "Live setup" > my_band.scn
```

This will create a new file named `my_band.scn` with the following content:

```
#2.7# "My Band" "Live setup" %000000000 1 X32GetScene V1.5 (c)2014 Patrick-Gilles Maillot

/ch/01/config/name "Lead Vox"
/ch/01/mix/fader 0.75
/ch/02/config/name "Guitar"
/ch/02/mix/fader 0.65
```
