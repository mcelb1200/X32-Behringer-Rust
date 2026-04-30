# x32_automix

`x32_automix` is a command-line utility that provides automixing functionality for Behringer X32 and Midas M32 consoles. It monitors the input levels of a specified range of channels and automatically adjusts their faders based on who is speaking, ensuring a clear and consistent mix for panel discussions, conferences, and other multi-microphone events.

## How It Works

The tool works by subscribing to the mixer's channel meter data. When a channel's input level exceeds a user-defined sensitivity threshold, its fader is automatically raised. If the channel's level drops below the threshold for a specified duration, the fader is lowered.

Additionally, the tool supports a **Number of Mics (NOM)** feature, which automatically lowers the overall mix level by 3 dB every time the number of open microphones doubles. This helps to prevent feedback and maintain a consistent output level.

## Command-Line Arguments

| Argument              | Short Flag | Long Flag          | Default Value | Description                                                                 |
| --------------------- | ---------- | ------------------ | ------------- | --------------------------------------------------------------------------- |
| IP Address            | `-i`       | `--ip`             | (none)        | **Required.** The IP address of the X32/M32 console.                           |
| Down Delay            |            | `--down-delay`     | 5             | The time in seconds that a channel must be quiet before its fader is lowered. |
| Meter Rate            |            | `--meter-rate-ms`  | 50            | The rate in milliseconds at which the console sends meter updates.          |
| Sensitivity           | `-s`       | `--sensitivity`    | 0.005         | The input level threshold required to activate a channel's fader.           |
| Start Channel         |            | `--start-channel`  | 1             | The first channel in the range to be automixed.                             |
| Stop Channel          |            | `--stop-channel`   | 32            | The last channel in the range to be automixed.                              |
| Use Bus               | `-b`       | `--use-bus`        | (false)       | If specified, automixing will be applied to a mix bus instead of the main L/R mix. |
| Bus Number            |            | `--bus-number`     | 1             | The bus number to use if `--use-bus` is specified.                             |
| NOM (Number of Mics)  |            | `--nom`            | (false)       | If specified, enables the NOM feature to adjust the overall mix level.      |

## Example Usage

To run the automixer for channels 1 through 8 on a console at IP address `192.168.1.64`, with a sensitivity of `0.01` and the NOM feature enabled, you would use the following command:

```bash
x32_automix -i 192.168.1.64 --start-channel 1 --stop-channel 8 -s 0.01 --nom
```

To automix channels 17-24 to bus 3, you would use:

```bash
x32_automix -i 192.168.1.64 --start-channel 17 --stop-channel 24 -b --bus-number 3
```
