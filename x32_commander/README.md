# x32_commander

`x32_commander` is a command-line utility that listens for specific OSC messages from an X32/M32 console and, in response, triggers other OSC or MIDI commands. It acts as a bridge, allowing you to create custom workflows and interactions that are not natively supported by the mixer.

**Note:** This is a partial rewrite of the original C-based `X32Commander` utility. Currently, only OSC-to-OSC command translation is fully supported. MIDI command functionality is not yet implemented.

## How It Works

The tool reads a user-defined command file that specifies a list of trigger-action pairs. It then connects to the X32 and listens for incoming OSC messages. When a message matches a trigger, `x32_commander` sends the corresponding action command. The action can be sent back to the same X32 or to a different OSC-enabled device, such as a lighting controller or a digital audio workstation (DAW).

## Command File Format

The command file is a plain text file (`.txt`) that defines the trigger-action mappings. Each line in the file represents a single command and must follow this format:

```
<type>~~~<incoming_osc_address> | <outgoing_command>
```

-   `<type>`: A single character indicating the type of the incoming and outgoing commands.
    -   `O`: OSC to OSC
    -   `M`: OSC to MIDI (currently not supported)
-   `~~~`: A separator between the type and the incoming OSC address.
-   `<incoming_osc_address>`: The OSC address that will trigger the action (e.g., `/ch/01/mix/fader`).
-   `|`: A separator between the trigger and the action.
-   `<outgoing_command>`: The OSC or MIDI command to be sent when the trigger is received.

Lines starting with `#` are treated as comments and are ignored.

### Example Command File

```
# When the fader of channel 1 is moved, set the fader of channel 2 to 0.5
O~~~/ch/01/mix/fader | /ch/02/mix/fader ,f 0.5

# When the mute button of channel 3 is pressed, set the mute of channel 4
O~~~/ch/03/mix/on | /ch/04/mix/on ,i 1
```

## Command-Line Arguments

| Argument         | Short Flag | Long Flag  | Default Value      | Description                                                    |
| ---------------- | ---------- | ---------- | ------------------ | -------------------------------------------------------------- |
| Command File     | `-f`       | `--file`   | `X32Commander.txt` | The path to the command file that defines the trigger-action mappings. |
| IP Address       | `-i`       | `--ip`     | (none)             | **Required.** The IP address of the X32/M32 console.              |
| OSC Output       | `-o`       | `--output` | (none)             | An optional IP address and port for sending the outgoing OSC commands (e.g., `127.0.0.1:9000`). If not provided, commands are sent back to the X32. |

## Example Usage

To run the commander with a command file named `my_commands.txt` on a console at IP address `192.168.1.64`, and send the outgoing commands to a device at `192.168.1.100` on port `8000`, you would use the following command:

```bash
x32_commander -i 192.168.1.64 -f my_commands.txt -o 192.168.1.100:8000
```
