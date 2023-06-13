/* main.rs
 *
 * Copyright 2023 Nicolai Syvertsen
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

mod application;
mod config;
mod window;
mod volumebox;
mod channelbox;
mod pwnodeobject;
mod pwnodemodel;
mod pipewire_connection;
mod format;

use std::collections::HashMap;

use self::application::PwvucontrolApplication;
use self::window::PwvucontrolWindow;

use config::{GETTEXT_PACKAGE, LOCALEDIR, PKGDATADIR};
use gettextrs::{bind_textdomain_codeset, bindtextdomain, textdomain};
use gtk::gio;
use gtk::prelude::*;
use pipewire::spa::Direction;

/// Messages sent by the GTK thread to notify the pipewire thread.
#[derive(Debug, Clone)]
enum GtkMessage {
    /// Toggle a link between the two specified ports.
    ToggleLink { port_from: u32, port_to: u32 },
    /// Sets the volume of the node
    SetVolume{id: u32, channel_volumes: Option<Vec<f32>>, volume: Option<f32>, mute: Option<bool>},
    /// Quit the event loop and let the thread finish.
    Terminate,
}

#[derive(Debug, Clone)]
enum ParamType {
    Volume(f32),
    ChannelVolumes(Vec<f32>),
    Mute(bool)
}

/// Messages sent by the pipewire thread to notify the GTK thread.
#[derive(Debug, Clone)]
enum PipewireMessage {
    NodeAdded {
        id: u32,
        name: String,
        node_type: Option<NodeType>,
    },
    NodeParam {
        id: u32,
        param: ParamType
    },
    NodeFormat {
        id: u32,
        channels: u32,
        rate: u32,
        format: u32,
    },
    NodeProps {
        id: u32,
        props: HashMap<String, String>,
    },
    PortAdded {
        id: u32,
        node_id: u32,
        name: String,
        direction: Direction,
        media_type: Option<MediaType>,
    },
    LinkAdded {
        id: u32,
        node_from: u32,
        port_from: u32,
        node_to: u32,
        port_to: u32,
        active: bool,
    },
    LinkStateChanged {
        id: u32,
        active: bool,
    },
    NodeRemoved {
        id: u32,
    },
    PortRemoved {
        id: u32,
        node_id: u32,
    },
    LinkRemoved {
        id: u32,
    },
}

#[derive(Debug, Clone)]
pub enum NodeType {
    Input,
    Output,
    Sink,
    Source
}

#[derive(Debug, Copy, Clone)]
pub enum MediaType {
    Audio,
    Video,
    Midi,
}

#[derive(Debug, Clone)]
pub struct PipewireLink {
    pub node_from: u32,
    pub port_from: u32,
    pub node_to: u32,
    pub port_to: u32,
}


static GLIB_LOGGER: glib::GlibLogger = glib::GlibLogger::new(
    glib::GlibLoggerFormat::Structured,
    glib::GlibLoggerDomain::CrateTarget,
);

fn init_glib_logger() {
    log::set_logger(&GLIB_LOGGER).expect("Failed to set logger");

    // Glib does not have a "Trace" log level, so only print messages "Debug" or higher priority.
    log::set_max_level(log::LevelFilter::Debug);
}


fn main() -> gtk::glib::ExitCode {
    init_glib_logger();
    // Set up gettext translations
    bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).expect("Unable to bind the text domain");
    bind_textdomain_codeset(GETTEXT_PACKAGE, "UTF-8")
        .expect("Unable to set the text domain encoding");
    textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    gtk::glib::set_application_name("Pwvucontrol");

    let resources = gio::Resource::load(PKGDATADIR.to_owned() + "/pwvucontrol.gresource")
        .or(gio::Resource::load("pwvucontrol.gresource")).expect("Could not load resources");

    // Load resources
    gio::resources_register(&resources);

    // Aquire main context so that we can attach the gtk channel later.
    let ctx = glib::MainContext::default();
    let _guard = ctx.acquire().unwrap();

    // Start the pipewire thread with channels in both directions.
    let (gtk_sender, gtk_receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
    let (pw_sender, pw_receiver) = pipewire::channel::channel();
    let pw_thread =
        std::thread::spawn(move || pipewire_connection::thread_main(gtk_sender, pw_receiver));

    let app = PwvucontrolApplication::new(gtk_receiver, pw_sender.clone());

    let exitcode = app.run();

    pw_sender
        .send(GtkMessage::Terminate)
        .expect("Failed to send message");

    pw_thread.join().expect("Pipewire thread panicked");

    exitcode
}
