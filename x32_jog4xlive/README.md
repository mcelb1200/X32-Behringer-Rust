# `x32_jog4xlive`

This application brings two rotary knobs to the X-Live! expansion board on X32.
When the X-Live! board is present in an X32, the FW (3.08 and more) sets Bank C
of User Assign section to a number of parameters. The leftmost rotary knob enables
circulating through sections of the SD card(s), the next knob travels through markers,
the 3rd one is unused, and the last one is used to set the time for adding markers.

We replace knobs 1 and 3 with this program; Here knob 1 is used to act as an audio
jog, and enables moving up and down in a song. Knob #3 is used to set the difference
in time between two consecutive increments of knob #1. This because X32 knobs do not
provide inertia, and are not real jogs of course.

The time difference is displayed in the user interface in the format 00m00s00 (minutes,
seconds, tens of milliseconds) varying from 10ms to 2m41s30

## Credits

*   **Original concept and work on the C library:** Patrick-Gilles Maillot
*   **Additional concepts by:** mcelb1200
*   **Rust implementation by:** mcelb1200
