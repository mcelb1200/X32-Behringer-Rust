# x32_tap

`x32_tap` is a command-line tool for setting the tempo of delay effects on a Behringer X32/M32 mixer. It is a Rust rewrite of the original C tools `X32Tap.c` and `X32TapW.c` by Patrick-Gilles Maillot.

## Features

- **Manual Mode:** Manually tap the tempo by pressing the Enter key.
- **Auto Mode:** Automatically detect the tempo from a channel's audio signal.

## Usage

### Manual Mode

To use manual mode, specify the IP address of the mixer and the FX slot number:

```
x32_tap manual --ip <IP_ADDRESS> --slot <SLOT>
```

Then, press the Enter key to tap the tempo.

### Auto Mode

To use auto mode, specify the IP address of the mixer, the FX slot number, the channel to monitor, and a sensitivity threshold:

```
x32_tap auto --ip <IP_ADDRESS> --slot <SLOT> --channel <CHANNEL> -e <SENSITIVITY>
```

The tool will then monitor the specified channel and automatically set the tempo when the signal level exceeds the sensitivity threshold.
