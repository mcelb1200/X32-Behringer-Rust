# X32Jog4Xlive

`x32_jog4xlive` is a command-line tool that provides jog wheel functionality for the Behringer X32's X-Live! card. It maps two of the user-assignable encoders to control playback position and jog sensitivity.

This is a Rust rewrite of the original C program by Patrick-Gilles Maillot.

## Usage

```
x32_jog4xlive --ip <X32_IP_ADDRESS>
```

## Functionality

- **Encoder 1:** Acts as a jog wheel, allowing you to move forward and backward in the current track.
- **Encoder 3:** Controls the sensitivity of the jog wheel (the `delta_time`).
