# x32_desk_save

`x32_desk_save` is a command-line utility for saving preferences, scenes, and routing data from a Behringer X32 digital mixer to a file. It is a Rust implementation of the original `X32DeskSave.c` tool by Patrick-Gilles Maillot.

## Usage

```
x32_desk_save [OPTIONS] <DESTINATION_FILE>
```

### Arguments

*   `<DESTINATION_FILE>` - The destination file path to save the retrieved data.

### Options

*   `-i`, `--ip <IP>` - X32 console IP address [default: `192.168.1.64`].
*   `-p`, `--pattern-file <PATTERN_FILE>` - File path to a pattern file containing OSC commands to be retrieved from the X32.
*   `-d`, `--desk-save` - Save a DeskSave file, containing the mixer's preferences and status.
*   `-s`, `--scene` - Save a Scene file, containing the mixer's channel and configuration settings.
*   `-r`, `--routing` - Save a Routing file, containing the mixer's input/output routing configuration.
*   `-h`, `--help` - Print help information.
*   `-V`, `--version` - Print version information.

## Examples

### Save a DeskSave File

To save the mixer's preferences and status to a file named `my_desksave.xds`, run the following command:

```
x32_desk_save -i 192.168.1.32 -d my_desksave.xds
```

### Save a Scene File

To save the mixer's current scene to a file named `my_scene.scn`, run the following command:

```
x32_desk_save -i 192.168.1.32 -s my_scene.scn
```

### Save a Routing File

To save the mixer's routing configuration to a file named `my_routing.rou`, run the following command:

```
x32_desk_save -i 192.168.1.32 -r my_routing.rou
```

### Save Data Using a Pattern File

To save a custom set of OSC commands from a pattern file named `my_pattern.txt`, run the following command:

```
x32_desk_save -i 192.168.1.32 -p my_pattern.txt my_custom_data.txt
```

The `my_pattern.txt` file should contain one OSC command per line. Lines starting with `#` are treated as comments and are ignored.
