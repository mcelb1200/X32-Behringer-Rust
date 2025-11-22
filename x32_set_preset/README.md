# x32_set_preset

`x32_set_preset` is a command-line utility for loading channel, effect, and routing presets to a Behringer X32 or Midas M32 mixer. It replaces the original Windows-only GUI application `X32SetPreset`.

## Features

-   **Support for Multiple Preset Types:** Handles Channel (`.chn`), Effect (`.efx`), and Routing (`.rou`) preset files.
-   **Intelligent Mapping:** Automatically maps generic preset data (e.g., `/config/...`) to specific target slots (e.g., Channel 1, FX Slot 3, Aux 5).
-   **Safety Flags:** Allows excluding specific sections of a channel preset (Headamp, Config, Gate, EQ, Dynamics, Sends) using command-line flags, mimicking the "Safes" checkboxes of the original tool.

## Usage

```bash
x32_set_preset [OPTIONS] <FILE>
```

### Arguments

| Option | Short | Description | Default |
| :--- | :--- | :--- | :--- |
| `--ip` | `-i` | The IP address of the X32 mixer. | `192.168.0.64` |
| `--target` | `-t` | The target slot for the preset (e.g., `ch01`, `fx3`, `aux05`, `bus01`, `mtx02`, `main_st`, `main_m`). Required for Channel and Effect presets. | (None) |
| `--safe-headamp` | | Skip Headamp settings (Channel presets only). | `false` |
| `--safe-config` | | Skip Config settings (Channel/Routing presets). | `false` |
| `--safe-gate` | | Skip Gate settings (Channel presets only). | `false` |
| `--safe-dyn` | | Skip Dynamics settings (Channel presets only). | `false` |
| `--safe-eq` | | Skip EQ settings (Channel presets only). | `false` |
| `--safe-send` | | Skip Mix/Send settings (Channel presets only). | `false` |
| `--master-safe` | | Mute Master faders before loading (Safety feature). | `false` |
| `--verbose` | `-v` | Enable verbose output. | `false` |

### Examples

**Load a channel preset to Channel 5, skipping EQ and Headamp settings:**

```bash
x32_set_preset my_kick.chn -t ch05 --safe-eq --safe-headamp
```

**Load an effect preset to FX Slot 2:**

```bash
x32_set_preset nice_reverb.efx -t fx2
```

**Load a routing preset:**

```bash
x32_set_preset studio_setup.rou
```
