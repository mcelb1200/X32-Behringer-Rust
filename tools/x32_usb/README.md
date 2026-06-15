# x32_usb

`x32_usb` is a command-line utility that provides a shell-like interface for managing a USB drive connected to a Behringer X32 or Midas M32 console. It allows you to navigate the directory structure, list files, load scenes and presets, and control the playback of WAV files, all from the command line. This tool is a Rust rewrite of the original `X32USB.c` utility by Patrick-Gilles Maillot.

## How It Works

The tool connects to the X32 and sends a series of OSC (Open Sound Control) messages to interact with the USB drive. It provides a set of subcommands that mimic common shell commands like `ls` and `cd`, making it an intuitive tool for anyone familiar with a command-line interface.

## Commands

### `ls`

Lists the files and directories in the current directory on the USB drive.

**Usage:**
```bash
x32_usb --ip <IP_ADDRESS> ls
```

---

### `cd`

Changes the current directory to the specified directory. The target can be either the index of the directory (from the `ls` command) or its name.

**Usage:**
```bash
x32_usb --ip <IP_ADDRESS> cd <TARGET>
```

**Example:**
```bash
x32_usb --ip 192.168.1.64 cd "My Shows"
```

---

### `load`

Loads a scene (`.scn`), snippet (`.snp`), effects (`.efx`), routing (`.rou`), or channel (`.chn`) file. The target can be either the index of the file or its name.

**Usage:**
```bash
x32_usb --ip <IP_ADDRESS> load <TARGET>
```

**Example:**
```bash
x32_usb --ip 192.168.1.64 load "live_show.scn"
```

---

### `umount`

Unmounts the USB drive from the console.

**Usage:**
```bash
x32_usb --ip <IP_ADDRESS> umount
```

---

### `play`

Plays the specified WAV file. The target can be either the index of the file or its name.

**Usage:**
```bash
x32_usb --ip <IP_ADDRESS> play <TARGET>
```

**Example:**
```bash
x32_usb --ip 192.168.1.64 play "walk_in_music.wav"
```

---

### `stop`

Stops the currently playing WAV file.

**Usage:**
```bash
x32_usb --ip <IP_ADDRESS> stop
```

---

### `pause`

Pauses the currently playing WAV file.

**Usage:**
```bash
x32_usb --ip <IP_ADDRESS> pause
```

---

### `resume`

Resumes playback of a paused WAV file.

**Usage:**
```bash
x32_usb --ip <IP_ADDRESS> resume
```
