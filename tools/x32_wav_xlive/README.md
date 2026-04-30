# x32_wav_xlive

`x32_wav_xlive` is a command-line utility for merging multiple mono WAV files into a multi-channel session format that is compatible with the Behringer X-Live! SD card recorder. This allows you to prepare audio tracks in a DAW and then load them onto an X32 for virtual soundchecks or backing track playback. This tool is a Rust rewrite of the original `X32Wav_Xlive.c` utility by Patrick-Gilles Maillot.

## How It Works

The tool takes a directory of mono, 24-bit WAV files (which must be named `ch_1.wav`, `ch_2.wav`, etc.) and creates a new session directory. Inside this new directory, it generates one or more multi-channel, 32-bit WAV files and a `SE_LOG.BIN` metadata file. The `SE_LOG.BIN` file contains information about the session, including the number of channels, sample rate, and any markers you've specified.

## Input File Requirements

-   **Directory:** All input files must be in the same directory.
-   **Naming:** Files must be named sequentially, starting with `ch_1.wav` (or `CH_1.WAV` if using the `--uppercase` flag).
-   **Format:** All files must be mono, 24-bit PCM WAV files.
-   **Sample Rate:** All files must have the same sample rate (either 44100 Hz or 48000 Hz).
-   **Duration:** All files must have the same duration.

## Command-Line Arguments

| Argument       | Short Flag | Long Flag       | Default Value | Description                                                                 |
| -------------- | ---------- | --------------- | ------------- | --------------------------------------------------------------------------- |
| Session Dir    |            |                 | (none)        | **Required.** The path to the directory containing the mono WAV files.      |
| Session Name   |            |                 | (none)        | An optional name for the X-Live! session.                                   |
| Marker File    | `-f`       | `--marker-file` | (none)        | An optional path to a file containing a list of markers (one per line, in seconds). |
| Marker         | `-m`       | `--marker`      | (none)        | A single marker time (in seconds). Can be specified multiple times.         |
| Uppercase      | `-u`       | `--uppercase`   | (false)       | If specified, the tool will look for and create `.WAV` files instead of `.wav`. |
| Silent         | `-S`       | `--silent`      | (false)       | If specified, suppresses all non-error output.                              |

## Example Usage

### Basic Conversion

To convert a directory of mono WAV files located at `./my_audio_tracks` into an X-Live! session named "Live Show":

```bash
x32_wav_xlive ./my_audio_tracks "Live Show"
```

This will create a new session directory inside `./my_audio_tracks` with a name like `A0B1C2D3`, containing the multi-channel WAV file(s) and `SE_LOG.BIN`.

### Conversion with Markers

To perform the same conversion but also add markers at 30 seconds and 1 minute 15 seconds:

```bash
x32_wav_xlive ./my_audio_tracks "Live Show" -m 30 -m 75
```

You can also provide markers in a file. If you have a file named `markers.txt` with the following content:

```
30
75
120
```

You can run the following command:

```bash
x32_wav_xlive ./my_audio_tracks "Live Show" -f markers.txt
```
