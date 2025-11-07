# X32 Replay

`x32_replay` is a command-line tool for recording and playing back OSC messages to and from a Behringer X32 or Midas M32 digital mixer. It is a Rust rewrite of the original C program by Patrick-Gilles Maillot.

## Features

- Record OSC messages from the mixer to a binary file.
- Play back recorded messages with the same timing as the original performance.
- Insert user comments into the recording.

## Usage

```
x32_replay [OPTIONS]
```

### Options

- `-i, --ip <IP>`: The IP address of the X32 console. [default: 127.0.0.1]
- `-p, --port <PORT>`: The port of the X32 console. [default: 10023]
- `-v, --verbose`: Verbose mode.
- `-f, --file <FILE>`: The file name for recording/playing. [default: X32ReplayFile.bincode]
- `-h, --help`: Print help information.
- `-V, --version`: Print version information.

### Commands

While `x32_replay` is running, you can enter the following commands:

- `record on`: Start or resume recording.
- `record off`: Stop recording.
- `record pause`: Pause recording.
- `play on`: Start playing.
- `play off`: Stop playing.
- `# <comment>`: Insert a comment into the recording.
- `exit`: Exit the program.

## Example

1.  Start the tool, connecting to your X32:

    ```
    x32_replay -i 192.168.1.64
    ```

2.  Start recording:

    ```
    > record on
    ```

3.  While recording, you can insert comments:

    ```
    > # Verse 1
    ```

4.  Stop recording:

    ```
    > record off
    ```

5.  Play back the recording:

    ```
    > play on
    ```
