# x32_tcp

`x32_tcp` is a command-line utility that creates a TCP to UDP bridge for Behringer X32 and Midas M32 consoles. It allows you to communicate with the mixer using a simple, text-based TCP protocol, which can be more convenient for scripting and integration with other applications than working directly with UDP. This tool is a Rust rewrite of the original `X32TCP.c` utility by Patrick-Gilles Maillot.

## How It Works

The tool runs a TCP server that listens for incoming connections. When a client connects, it can send OSC commands as plain text strings. `x32_tcp` then translates these strings into standard OSC messages, sends them to the X32 via UDP, and forwards any response from the mixer back to the TCP client.

## Command-Line Arguments

| Argument   | Short Flag | Long Flag   | Default Value | Description                                  |
| ---------- | ---------- | ----------- | ------------- | -------------------------------------------- |
| IP Address | `-i`       | `--ip`      | `127.0.0.1`   | The IP address of the X32/M32 console.       |
| Port       | `-p`       | `--port`    | `10041`       | The TCP port for the server to listen on.    |
| Backlog    | `-b`       | `--backlog` | `10`          | The maximum number of pending connections.   |
| Debug      | `-d`       | `--debug`   | `false`       | If specified, enables debug output.          |
| Verbose    | `-v`       | `--verbose` | `false`       | If specified, enables verbose output.        |

## Protocol

1.  **Connect:** Use a TCP client (like `telnet` or `netcat`) to connect to the `x32_tcp` server on the specified port.
2.  **Send Commands:** Send OSC commands as plain text strings, with each command terminated by a newline character (`\n`). The format for the command string is the OSC path, a comma, the OSC type tag string, and then the arguments, all separated by spaces (e.g., `/ch/01/mix/fader,f 0.75`).
3.  **Receive Responses:** The server will send back the X32's response as a formatted string. If there is no response from the mixer, the server will send back the string "no data".
4.  **Disconnect:** To close the connection, send the command `exit`.

## Example Usage

### Start the Server

To start the server, connecting to an X32 at `192.168.1.64` and listening for TCP connections on port `12345`:

```bash
x32_tcp --ip 192.168.1.64 --port 12345
```

### Interact with the Server using `netcat`

In a separate terminal, you can use `netcat` (or `nc`) to connect to the server and send commands:

```bash
netcat localhost 12345
/ch/01/mix/fader,f 0.5
/ch/01/mix/fader
exit
```

**Explanation:**

1.  The first command sets the fader of channel 1 to `0.5`.
2.  The second command queries the fader level of channel 1. The server will respond with the new value (e.g., `/ch/01/mix/fader,f 0.5`).
3.  The `exit` command closes the connection.
