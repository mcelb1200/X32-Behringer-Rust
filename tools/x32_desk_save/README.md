# x32_desk_save

`x32_desk_save` is a command-line utility for saving the state of a Behringer X32 or Midas M32 console to a file. It can save the entire scene, just the routing, or a custom set of parameters defined in a pattern file. This tool is a Rust rewrite of the original `X32DeskSave.c` utility by Patrick-Gilles Maillot.

## How It Works

The tool connects to the X32 and sends a series of OSC (Open Sound Control) messages to query the values of various parameters. The responses are then formatted and saved to a local file. This file can be used with `x32_desk_restore` to load the saved state back onto the mixer.

## Command-Line Arguments

| Argument          | Short Flag | Long Flag         | Default Value  | Description                                                                                             |
| ----------------- | ---------- | ----------------- | -------------- | ------------------------------------------------------------------------------------------------------- |
| Destination File  |            |                   | (none)         | **Required.** The path to the file where the data will be saved.                                           |
| IP Address        | `-i`       | `--ip`            | `192.168.1.64` | The IP address of the X32/M32 console.                                                                    |
| Pattern File      | `-p`       | `--pattern-file`  | (none)         | An optional path to a file containing a list of OSC commands to query.                                    |

## Save Modes

You must specify one of the following save modes:

| Mode        | Short Flag | Long Flag     | Description                                                                 |
| ----------- | ---------- | ------------- | --------------------------------------------------------------------------- |
| Desk Save   | `-d`       | `--desk-save` | Saves the mixer's preferences and status to a `.xds` file.                  |
| Scene       | `-s`       | `--scene`     | Saves the mixer's channel and configuration settings to a `.scn` file.      |
| Routing     | `-r`       | `--routing`   | Saves the mixer's input/output routing configuration to a `.rou` file.      |

## Example Usage

### Save a Full Scene

To save the mixer's current scene to a file named `my_live_show.scn`:

```bash
x32_desk_save --ip 192.168.1.32 --scene my_live_show.scn
```

### Save Only the Routing

To save only the routing configuration to a file named `routing_preset.rou`:

```bash
x32_desk_save --ip 192.168.1.32 --routing routing_preset.rou
```

### Save a Custom Set of Parameters

To save a custom set of parameters defined in `my_pattern.txt` to a file named `custom_settings.txt`:

```bash
x32_desk_save --ip 192.168.1.32 --pattern-file my_pattern.txt custom_settings.txt
```

#### Example `my_pattern.txt`:

```
# Get the name and fader level for the first 8 channels
/ch/01/config/name
/ch/01/mix/fader
/ch/02/config/name
/ch/02/mix/fader
/ch/03/config/name
/ch/03/mix/fader
/ch/04/config/name
/ch/04/mix/fader
/ch/05/config/name
/ch/05/mix/fader
/ch/06/config/name
/ch/06/mix/fader
/ch/07/config/name
/ch/07/mix/fader
/ch/08/config/name
/ch/08/mix/fader
```
