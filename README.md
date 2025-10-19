# pwvucontrol

## Experimental wireplumber-rs wp-5.0 branch

This is an experiment to see if using the in-progress wp-0.5 branch of wireplumber-rs is goin going to work.

## Flatpak


The recommended way of installing pwvucontrol is through Flatpak. If you don't have
Flatpak installed, you can get it from [the Flatpak website](https://flatpak.org/setup).

You can install stable builds of pwvucontrol from [Flathub](https://flathub.org)
by using this command:

    flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo
    flatpak install flathub com.saivert.pwvucontrol

<a href="https://flathub.org/apps/com.saivert.pwvucontrol"><img src="https://flathub.org/assets/badges/flathub-badge-en.png" width="200"/></a>


## General info

This is an attempt at making a volume control applet for PipeWire.

Current implemented features as of 2024-05-04:

- Volume control
- Mute
- Media name display
- Peak level meter
- Output device (Sink) drop down for playback streams
- Default output device
- Card profile selection
- Port selection for sinks and sources

## What it looks like

![Screenshot](../assets/screenshot.png)

## Building

Use meson to build.

    meson setup builddir
    meson compile -C builddir
    meson install -C builddir


## Help needed
Help with making code robust.

UI ideas and mockups welcome!
