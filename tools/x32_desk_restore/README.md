# x32_desk_restore

`x32_desk_restore` is a command-line utility for restoring the state of a Behringer X32 or Midas M32 console from a file. It can be used to load scenes, snippets, or any file that contains a list of OSC commands. This tool is a Rust rewrite of the original `X32DeskRestore.c` utility by Patrick-Gilles Maillot.

## How It Works

The tool reads a specified file line by line, parsing each line as an OSC command and sending it to the mixer. This allows you to recreate a previously saved mixer state, making it an essential tool for live sound engineers who need to manage multiple shows or setups.

## File Format

The source file should be a plain text file containing one OSC command per line. The format for each line should be the OSC path, followed by a comma, the OSC type tag string, and then the arguments.

Lines starting with `#` are treated as comments and are ignored.

### Example `.scn` File:

```
# Set the name of channel 1 to "Kick"
/ch/01/config/name,s "Kick"

# Set the fader level of channel 1 to 0.75
/ch/01/mix/fader,f 0.75

# Mute channel 2
/ch/02/mix/on,i 0
```

## Command-Line Arguments

| Argument      | Short Flag | Long Flag | Default Value   | Description                                            |
| ------------- | ---------- | --------- | --------------- | ------------------------------------------------------ |
| Source File   |            |           | (none)          | **Required.** The path to the file to be loaded.         |
| IP Address    | `-i`       | `--ip`    | `192.168.1.64`  | The IP address of the X32/M32 console.                 |

## Example Usage

To restore the mixer's state from a file named `my_show.scn` on a console at IP address `192.168.1.32`:

```bash
x32_desk_restore --ip 192.168.1.32 my_show.scn
```
