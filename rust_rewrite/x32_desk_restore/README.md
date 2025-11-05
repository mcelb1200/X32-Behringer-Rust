# x32_desk_restore

`x32_desk_restore` is a command-line utility for restoring preferences, scenes, and routing data to a Behringer X32 digital mixer from a file. It is a Rust implementation of the original `X32DeskRestore.c` tool by Patrick-Gilles Maillot.

## Usage

```
x32_desk_restore [OPTIONS] <SOURCE_FILE>
```

### Arguments

*   `<SOURCE_FILE>` - The source file path to restore data from.

### Options

*   `-i`, `--ip <IP>` - X32 console IP address [default: `192.168.1.64`].
*   `-h`, `--help` - Print help information.
*   `-V`, `--version` - Print version information.

## Examples

### Restore a DeskSave File

To restore the mixer's preferences and status from a file named `my_desksave.xds`, run the following command:

```
x32_desk_restore -i 192.168.1.32 my_desksave.xds
```

### Restore a Scene File

To restore a scene from a file named `my_scene.scn`, run the following command:

```
x32_desk_restore -i 192.168.1.32 my_scene.scn
```

The file should contain one OSC command per line. Lines starting with `#` are treated as comments and are ignored.
