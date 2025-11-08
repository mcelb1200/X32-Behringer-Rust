# X32 Custom Layer

This tool is a Rust rewrite of the `X32CustomLayer.c` utility by Patrick-Gilles Maillot. It provides a command-line interface for creating, saving, restoring, and resetting custom channel layers on a Behringer X32 or Midas M32 digital mixing console.

## Usage

```
x32_custom_layer <COMMAND>
```

### Commands

- `set`: Sets a custom layer on the X32 mixer.
- `save`: Saves the current custom layer to a file.
- `restore`: Restores a custom layer from a file.
- `reset`: Resets channels to their default settings.
- `list`: Lists the current channel sources.

---

### `set`

This command allows you to reassign channel strips to create a custom layer. You can specify one or more assignments in the format `DEST=SRC`, where `DEST` is the destination channel strip and `SRC` is the source channel strip.

**Usage:**
```
x32_custom_layer set --ip <IP_ADDRESS> <ASSIGNMENTS>...
```

**Example:**
```
x32_custom_layer set --ip 192.168.1.60 1=5 2=10 3=15
```
This command will set channel strip 1 to the settings of channel 5, channel strip 2 to the settings of channel 10, and channel strip 3 to the settings of channel 15.

---

### `save`

This command saves the current state of all channel strips (1-32) and aux inputs (1-8) to a snippet file. This file can be used later to restore the mixer's state.

**Usage:**
```
x32_custom_layer save --ip <IP_ADDRESS> --file <FILE_PATH>
```

**Example:**
```
x32_custom_layer save --ip 192.168.1.60 --file my_layer.snp
```

---

### `restore`

This command restores the state of all channel strips and aux inputs from a previously saved snippet file.

**Usage:**
```
x32_custom_layer restore --ip <IP_ADDRESS> --file <FILE_PATH>
```

**Example:**
```
x32_custom_layer restore --ip 192.168.1.60 --file my_layer.snp
```

---

### `reset`

This command resets one or more channel strips to their default settings. You can specify a single channel, a comma-separated list of channels, or a range of channels.

**Usage:**
```
x32_custom_layer reset --ip <IP_ADDRESS> --channels <CHANNELS>
```

**Examples:**
- Reset a single channel:
  ```
  x32_custom_layer reset --ip 192.168.1.60 --channels 5
  ```
- Reset multiple channels:
  ```
  x32_custom_layer reset --ip 192.168.1.60 --channels 1,3,5
  ```
- Reset a range of channels:
  ```
  x32_custom_layer reset --ip 192.168.1.60 --channels 1-8
  ```
- Reset a combination of single channels and ranges:
  ```
  x32_custom_layer reset --ip 192.168.1.60 --channels 1,3,5-8
  ```

---

### `list`

This command lists the current input source for each channel strip and aux input.

**Usage:**
```
x32_custom_layer list --ip <IP_ADDRESS>
```

**Example:**
```
x32_custom_layer list --ip 192.168.1.60
```
This will print a table showing the source for each channel, similar to the following:

```
  Channel	Source		Channel		Source
     01		In01		17		    In17
     02		In02		18		    In18
...

  AuxIN		Source		AuxIN		Source
     01		Aux1		05		    Aux5
     02		Aux2		06		    Aux6
...
```
