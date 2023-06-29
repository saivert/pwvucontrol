# pwvucontrol

## General info

This is an attempt are making a volume control applet for Pipewire.

Current implemented features as of 2023-06-08:

- Volume control (using pipewire's main volume parameter, not channel volumes)
- Mute
- Media name display
- Proper volume control of device nodes (using current route lookup)

Not implemented yet:

- Default sink dropdown (only mock UI)

## wireplumber-rs branch

This is the wireplumber-rs branch which means it is a complete rewrite using wireplumber-rs bindings against libwireplumber to interact with pipewire instead of the pipewire-rs bindings. We have a different architecture where PwNodeObject is responsible for listening to events from the WpNode object and sending param updates. We no longer run our own thread loop for handling the pipewire connection as libwireplumber does that for us.

![Screenshot](../assets/screenshot.png)

# Help needed
I need a nice icon!

Help with making it run as a flatpak.

Help with making code robust.

UI ideas and mockups welcome!
