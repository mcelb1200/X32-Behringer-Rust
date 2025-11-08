# X32TCP

`x32_tcp` is a command-line utility that acts as a TCP to UDP bridge for the Behringer X32 and Midas M32 digital mixers. It allows you to send OSC commands to the mixer using a simple text-based TCP protocol.

This is a Rust rewrite of the original C program `X32TCP.c` by Patrick-Gilles Maillot.

## Usage

```
x32_tcp [OPTIONS]
```

### Options

| Flag | Name      | Description                  | Default     |
|------|-----------|------------------------------|-------------|
| `-b` | `--backlog` | Server max connections backlog | 10          |
| `-i` | `--ip`      | X32 console IP address       | 127.0.0.1   |
| `-p` | `--port`    | Server port                  | 10041       |
| `-d` | `--debug`   | Debug mode                   | false       |
| `-v` | `--verbose` | Verbose mode                 | false       |

## Protocol

Connect to the `x32_tcp` server on the specified port. Send OSC commands as plain text strings, with each command terminated by a newline character.

The format for the command string is the OSC path followed by a space, then the type tag string, and then a space-separated list of arguments. String arguments with spaces should be enclosed in double quotes.

### Example

```
/ch/01/mix/fader ,f 0.75
```

The server will send back the X32's response as a formatted string. If there is no response from the mixer within the timeout period, the server will send back the string "no data".

To close the connection, send the command `exit`.
