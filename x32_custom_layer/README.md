# x32_custom_layer

`x32_custom_layer` is a powerful command-line utility for managing custom channel layers on Behringer X32 and Midas M32 consoles. It provides a suite of tools for creating, saving, restoring, and resetting channel layouts, making it easy to reconfigure the mixer for different events or workflows. This tool is a Rust rewrite of the original `X32CustomLayer.c` utility by Patrick-Gilles Maillot.

## How It Works

The tool communicates with the X32 over the network using OSC (Open Sound Control) messages to manipulate the channel strip assignments and settings. It can save the entire state of the mixer's input channels to a `.snp` (snippet) file, which can be reloaded later to restore that exact configuration.

## Commands

### `set`

Remaps one or more source channels to different destination channel strips, creating a custom fader layer.

**Usage:**
```bash
x32_custom_layer set --ip <IP_ADDRESS> <ASSIGNMENTS>...
```

**Arguments:**

| Argument      | Description                                                                                             |
| ------------- | ------------------------------------------------------------------------------------------------------- |
| `--ip`        | **Required.** The IP address of the X32/M32 console.                                                       |
| `ASSIGNMENTS` | A space-separated list of assignments in the format `DEST=SRC` (e.g., `1=5 2=10 3=15`). `DEST` is the destination channel strip you want to control, and `SRC` is the source channel that will be controlled. |

**Example:**
```bash
x32_custom_layer set --ip 192.168.1.64 1=17 2=18 3=19 4=20
```

---

### `save`

Saves the current configuration of all input channels (1-32) and auxiliary inputs (1-8) to a snippet (`.snp`) file.

**Usage:**
```bash
x32_custom_layer save --ip <IP_ADDRESS> --file <FILE_PATH>
```

**Arguments:**

| Argument | Description                                      |
| -------- | ------------------------------------------------ |
| `--ip`   | **Required.** The IP address of the X32/M32 console. |
| `--file` | **Required.** The path to the snippet file to be created. |

**Example:**
```bash
x32_custom_layer save --ip 192.168.1.64 --file my_drum_kit.snp
```

---

### `restore`

Restores a previously saved channel configuration from a snippet file.

**Usage:**
```bash
x32_custom_layer restore --ip <IP_ADDRESS> --file <FILE_PATH>
```

**Arguments:**

| Argument | Description                                      |
| -------- | ------------------------------------------------ |
| `--ip`   | **Required.** The IP address of the X32/M32 console. |
| `--file` | **Required.** The path to the snippet file to be loaded. |

**Example:**
```bash
x32_custom_layer restore --ip 192.168.1.64 --file my_drum_kit.snp
```

---

### `reset`

Resets one or more channel strips to their default factory settings.

**Usage:**
```bash
x32_custom_layer reset --ip <IP_ADDRESS> --channels <CHANNELS>
```

**Arguments:**

| Argument     | Description                                                                                             |
| -------------- | ------------------------------------------------------------------------------------------------------- |
| `--ip`         | **Required.** The IP address of the X32/M32 console.                                                       |
| `--channels`   | **Required.** A comma-separated list and/or hyphenated range of channels to reset (e.g., `1,3,5-8`).      |

**Example:**
```bash
x32_custom_layer reset --ip 192.168.1.64 --channels 1-8,17,24
```

---

### `list`

Displays the current input source for each channel strip and auxiliary input.

**Usage:**
```bash
x32_custom_layer list --ip <IP_ADDRESS>
```

**Arguments:**

| Argument | Description                                      |
| -------- | ------------------------------------------------ |
| `--ip`   | **Required.** The IP address of the X32/M32 console. |

**Example Output:**
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
