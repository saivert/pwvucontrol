# pwvucontrol

## Flatpak

[Get them from releases page](https://github.com/saivert/pwvucontrol/releases)

There is a flatpak-builder manifest in the build-aux directory. I'm planning on getting it on flathub but I need help crossing the finish line here.

## General info

This is an attempt are making a volume control applet for Pipewire.

Current implemented features as of 2023-12-23:

- Volume control
- Mute
- Media name display
- Peak level meter
- Output device (Sink) drop down for playback streams
- Default output device
- Card profile selection

## What it looks like

![Screenshot](../assets/screenshot.png)

## Building

Use meson to build.

    meson setup builddir
    meson compile -C builddir
    meson install -C builddir


## Help needed
I need a nice icon in SVG!

Flatpak is on the GitHub releases page. I need help getting it on Flathub.

Help with making code robust.

UI ideas and mockups welcome!
