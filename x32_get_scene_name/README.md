# x32_get_scene_name

`x32_get_scene_name` is a command-line utility that retrieves the name of the currently active scene on a Behringer X32 or Midas M32 console. It can be run in a one-time mode to get the current scene name and then exit, or in a continuous monitoring mode to print the name of each new scene as it is loaded. This tool is a Rust rewrite of the original `GetSceneName.c` utility by Patrick-Gilles Maillot.

## How It Works

The tool connects to the X32 and subscribes to notifications for scene changes. When a scene is loaded on the console, the mixer sends an OSC message containing the scene number and name. `x32_get_scene_name` captures this message and prints the name to standard output.

## Command-Line Arguments

| Argument   | Short Flag | Long Flag   | Default Value  | Description                                                                 |
| ---------- | ---------- | ----------- | -------------- | --------------------------------------------------------------------------- |
| IP Address | `-i`       | `--ip`      | `192.168.1.62` | The IP address of the X32/M32 console.                                        |
| Verbose    | `-v`       | `--verbose` | `1`            | Set to `1` to print welcome and connection status messages, or `0` to run silently. |
| One-time   | `-o`       | `--onetime` | `1`            | Set to `1` to exit after the first scene name is received, or `0` to run continuously. |

## Example Usage

### Get the Current Scene Name and Exit

To connect to an X32 at `192.168.1.32`, get the next scene name that is loaded, and then exit:

```bash
x32_get_scene_name --ip 192.168.1.32 --onetime 1
```

### Continuously Monitor for Scene Changes

To connect to an X32 at `192.168.1.32` and continuously monitor for scene changes, printing the name of each new scene as it is loaded:

```bash
x32_get_scene_name --ip 192.168.1.32 --onetime 0
```
