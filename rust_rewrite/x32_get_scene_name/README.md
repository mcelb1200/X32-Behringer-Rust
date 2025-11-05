# x32_get_scene_name

`x32_get_scene_name` is a command-line utility that connects to a Behringer X32 digital mixer, listens for scene change events, and prints the name of the new scene to standard output. It is a Rust implementation of the original `GetSceneName.c` tool by Patrick-Gilles Maillot.

## Usage

```
x32_get_scene_name [OPTIONS]
```

### Options

*   `-i`, `--ip <IP>` - X32 console IP address [default: `192.168.1.62`].
*   `-v`, `--verbose <VERBOSE>` - Prints welcome and connection status messages (0 or 1) [default: 1].
*   `-o`, `--onetime <ONETIME>` - Exits at first occurrence (0 or 1) [default: 1].
*   `-h`, `--help` - Print help information.
*   `-V`, `--version` - Print version information.

## Examples

### Get Scene Name Once

To connect to an X32 at `192.168.1.32`, get the next scene name, and then exit, run the following command:

```
x32_get_scene_name -i 192.168.1.32 -o 1
```

### Continuously Monitor for Scene Changes

To continuously monitor for scene changes and print the name of each new scene, run the following command:

```
x32_get_scene_name -i 192.168.1.32 -o 0
```
