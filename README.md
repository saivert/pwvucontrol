# pwvucontrol

## Announcement Regarding `wireplumber-rs` Dependency and the Road Ahead

As many of you know, the `wireplumber-rs` library (a Rust wrapper around WirePlumber) is no longer maintained. It was a hobby project and never officially supported by the PipeWire project, which has made it a long-standing issue.

Upgrading to GNOME Flatpak runtime version 49 also requires switching to the `org.freedesktop.Sdk.Extension.llvm20` SDK extension. Unfortunately, this breaks the build for WirePlumber 0.4.9 due to compiler errors. The only viable path is upgrading to WirePlumber 0.5.0, which is incompatible with `wireplumber-rs` 0.4. This effectively ends the road for `wireplumber-rs`.

I’m not in a position to fork and maintain `wireplumber-rs` myself, as I lack the expertise in `gobject-introspection` and writing robust Rust bindings for WirePlumber.

### Possible Paths Forward

1. **Rewrite `pwvucontrol` to use `pipewire-rs` directly**

   This means re-implementing the tracking of PipeWire objects and communication with the PipeWire server—functionality previously handled by WirePlumber. I can reuse some of Helvum’s code for this.

2. **Switch to using PulseAudio APIs**

   PulseAudio remains supported for backward compatibility and is considered a higher-level API by the PipeWire project. There are solid Rust wrappers available. I could still use PipeWire APIs where needed, but this would mean mixing two protocols e.g., reconciling a PulseAudio sink with a PipeWire node for direct API calls.
   While `pwvucontrol` currently doesn't rely heavily on PipeWire-only features, future plans include support for session manager properties (e.g., `pw-metadata -n sm-settings`) and node properties (e.g., `pw-cli enum-params <sink id> PropInfo`), which would require the PipeWire API.

3. **Rewrite it in another language**

    I'm not going to do that.

### Final Thoughts

This project is, above all, a personal Rust learning exercise. Like most open source projects, development happens when I have both the time and interest. I'm not committed to maintaining it indefinitely, unless it gains more contributors and traction.

That’s all for now.


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
