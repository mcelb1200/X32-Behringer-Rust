# x32_tap

`x32_tap` is a command-line utility for setting the delay time of an effects unit on the Behringer X32 and Midas M32 mixers. It allows you to "tap" the tempo using your keyboard to synchronize the delay with the music. This tool replaces the original `X32Tap.c` (CLI) and `X32TapW.c` (GUI/Auto) utilities.

## Features

-   **Manual Tap:** Press `Enter` repeatedly to set the delay time based on the interval between presses.
-   **Auto Tap:** (Coming Soon) Automatically detect tempo from a signal source on the mixer.
-   **FX Slot Selection:** Select which FX slot (1-4) to control.
-   **Verification:** Automatically verifies that the selected slot contains a delay effect.

## Usage

```bash
x32_tap [OPTIONS]
```

### Arguments

| Option | Short | Description | Default |
| :--- | :--- | :--- | :--- |
| `--ip` | `-i` | The IP address of the X32 mixer. | `192.168.0.64` |
| `--slot` | `-s` | The FX slot number (1-4) containing the delay effect. | `1` |
| `--auto` | `-a` | Enable auto-tap mode (Not yet implemented). | `false` |

### Example

To control a delay effect in slot 2 on a mixer at 192.168.1.50:

```bash
x32_tap -i 192.168.1.50 -s 2
```

Once running, follow the on-screen instructions:
1.  Press `Enter` to tap the tempo.
2.  Press `q` then `Enter` to quit.
