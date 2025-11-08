# x32_emulator

`x32_emulator` is a command-line utility that simulates a Behringer X32 or Midas M32 console. It listens for incoming OSC messages and responds in the same way a physical mixer would, making it an invaluable tool for offline development and testing of X32 control applications.

## How It Works

The emulator runs as a server that binds to a specified IP address and port, listening for UDP packets containing OSC messages. It maintains an internal state of the mixer's parameters, which can be modified and queried by OSC clients. This allows you to test your applications without needing to connect to a real X32 console.

## Command-Line Arguments

| Argument   | Short Flag | Long Flag | Default Value | Description                                  |
| ---------- | ---------- | --------- | ------------- | -------------------------------------------- |
| IP Address | `-i`       | `--ip`    | `0.0.0.0`     | The IP address to bind the server to.        |
| Port       | `-p`       | `--port`  | `10023`       | The port number to bind the server to.       |

## Example Usage

To start the emulator on your local machine, listening on the default X32 port (10023), simply run the following command:

```bash
x32_emulator
```

You can then connect your X32 control application to `127.0.0.1:10023` to interact with the emulator.

To run the emulator on a specific IP address and port, you can use the `--ip` and `--port` flags:

```bash
x32_emulator --ip 192.168.1.100 --port 10024
```

## Use in Testing

The `x32_emulator` crate is also designed to be used as a library within your own Rust applications, making it easy to write integration tests for your X32 tools. You can start the emulator server in a separate thread, run your tests against it, and then shut it down when you're done.
