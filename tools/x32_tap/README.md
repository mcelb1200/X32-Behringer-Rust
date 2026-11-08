# x32_tap

`x32_tap` is a command-line utility for setting the delay time of an effects unit across the Behringer X32, Midas M32, and Wing family of digital mixers (`X32`, `Wing`, `XR18`, `XR16`, `XR12`). It allows you to "tap" the tempo using your keyboard to synchronize the delay with the music. This tool replaces the original `X32Tap.c` (CLI) and `X32TapW.c` (GUI/Auto) utilities.

## Features

-   **MixerModel Family Support:** Compatible with `X32`, `Wing`, `XR18`, `XR16`, and `XR12` consoles with model-aware target port resolution (10023 for X32, 2222 for Wing, 10024 for XR series).
-   **Manual Tap:** Press `Enter` repeatedly to set the delay time based on the interval between presses.
-   **Auto Tap:** Automatically detect tempo from a signal source on the mixer by monitoring a channel's gate meter level.
-   **Model-Aware Limits:** Validates FX slots (1-16 on Wing, 1-8 on X32, 1-4 on XR) and channel bounds (up to 40 on Wing, 32 on X32, 16 on XR18/16, 12 on XR12).
-   **Verification:** Automatically verifies that the selected slot contains a delay effect.

## Usage

```bash
x32_tap [OPTIONS]
```

### Arguments

| Option | Short | Description | Default |
| :--- | :--- | :--- | :--- |
| `--model` | `-M` | Digital mixer model (`X32`, `Wing`, `XR18`, `XR16`, `XR12`). | `X32` |
| `--ip` | `-i` | The IP address of the mixer. | `192.168.0.64` |
| `--slot` | `-s` | The FX slot number containing the delay effect. | `1` |
| `--auto` | `-a` | Enable auto-tap mode by monitoring a channel's meter level. | `false` |
| `--channel` | `-c` | In auto mode, the channel number to monitor. | `1` |
| `--threshold` | `-t` | In auto mode, the gate meter threshold level (e.g. 0.5) to trigger a tap. | `0.5` |

### Example

To control a delay effect in slot 2 on a Wing mixer at 192.168.1.50:

```bash
x32_tap -M Wing -i 192.168.1.50 -s 2
```

Once running, follow the on-screen instructions:
1.  Press `Enter` to tap the tempo.
2.  Press `q` then `Enter` to quit.
