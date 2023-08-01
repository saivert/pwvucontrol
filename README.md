# pwvucontrol

[Download flatpak bundle](https://saivert.com/files/pwvucontrol.flatpak) (built from git Aug. 1 2023)

## General info

This is an attempt are making a volume control applet for Pipewire.

Current implemented features as of 2023-07-30:

- Volume control
- Mute
- Media name display
- Peak level meter
- Output device (Sink) drop down for playback streams


## wireplumber-rs branch

This is the wireplumber-rs branch which means it is a complete rewrite using wireplumber-rs bindings against libwireplumber to interact with pipewire instead of the pipewire-rs bindings. We have a different architecture where PwNodeObject is responsible for listening to events from the WpNode object and sending param updates. We no longer run our own thread loop for handling the pipewire connection as libwireplumber does that for us.

![Screenshot](../assets/screenshot.png)

# Help needed
I need a nice icon!

Help with making it run as a flatpak. Update Aug 1 2023: It runs now but I need help getting it on Flathub.

Help with making code robust.

UI ideas and mockups welcome!
