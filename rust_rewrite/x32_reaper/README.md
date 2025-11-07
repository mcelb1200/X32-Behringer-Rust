# X32 Reaper

`x32_reaper` is a tool that bridges the Behringer X32/Midas M32 series of digital mixers with the Reaper DAW. It allows for synchronization of faders, mutes, pans, and other controls between the mixer and the DAW.

This application is a Rust rewrite of the original C program `X32Reaper.c` by Patrick-Gilles Maillot.

## Features

*   **Synchronization:** Real-time synchronization of faders, mutes, pans, scribble strips, EQ settings, and mix bus sends.
*   **Channel Banking:** Control a large number of Reaper tracks using the X32's channel banks.
*   **DCA Control:** Map X32 DCAs to control groups of Reaper tracks.
*   **Transport Control:** Use the X32's transport controls to control Reaper's playback, including scrubbing and moving by measure/beat.
*   **Marker Control:** Set Reaper markers from the X32's user-assignable buttons.
*   **Configuration:** A simple JSON configuration file allows for easy customization of the mapping between the X32 and Reaper.

## Usage

1.  **Configuration:**
    *   Create a `config.json` file in the same directory as the `x32_reaper` executable.
    *   The application will generate a default `config.json` file on the first run if one is not found.
    *   Modify the `config.json` file to match your network settings and desired mappings.

2.  **Run the application:**
    ```
    cargo run -p x32_reaper
    ```

3.  **Setup Reaper:**
    *   In Reaper, go to `Options > Preferences > Control/OSC/web`.
    *   Add a new `OSC` control surface.
    *   Set the `Mode` to `Send + receive`.
    *   Set the `Local listen port` to the `reaper_receive_port` specified in your `config.json` file.
    *   Set the `Send to host` to the IP address of the machine running `x32_reaper` and the `reaper_send_port` from your `config.json`.
    *   In the OSC pattern configuration file, add patterns to send the desired track information (e.g., volume, pan, mute) to `x32_reaper`.

4.  **Setup X32:**
    *   Ensure your X32 is connected to the same network as the machine running `x32_reaper`.
    *   The `x32_ip` in your `config.json` should be the IP address of your X32.
