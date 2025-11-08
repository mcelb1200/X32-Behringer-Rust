# x32_set_scene

`x32_set_scene` is a command-line utility for sending a series of OSC commands from standard input to a Behringer X32 or Midas M32 console. It is designed to be a simple and flexible way to load scenes, snippets, or any custom sequence of commands. This tool is a Rust rewrite of the original `X32SetScene.c` utility by Patrick-Gilles Maillot.

## How It Works

The tool connects to the X32 and reads OSC command strings from standard input, one line at a time. Each line is parsed as an OSC message and sent to the mixer. This makes it easy to pipe the output of other tools (like `x32_get_scene`) or the contents of a scene file directly to the mixer.

## Command-Line Arguments

| Argument   | Short Flag | Long Flag | Default Value  | Description                                            |
| ---------- | ---------- | --------- | -------------- | ------------------------------------------------------ |
| IP Address | `-i`       | `--ip`    | `192.168.0.64` | The IP address of the X32/M32 console.                 |
| Delay      | `-d`       | `--delay` | `1`            | The delay in milliseconds between sending each command. |

## Example Usage

To load a scene from a file named `my_show.scn` onto a console at IP address `192.168.1.32`, you can use the following command:

```bash
cat my_show.scn | x32_set_scene --ip 192.168.1.32
```

You can also use the tool to send individual commands from the command line:

```bash
echo "/ch/01/mix/fader,f 0.75" | x32_set_scene --ip 192.168.1.32
```
