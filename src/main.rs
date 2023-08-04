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

mod config {
    #![allow(dead_code)]

    include!(concat!(env!("CODEGEN_BUILD_DIR"), "/config.rs"));
}

mod application;
mod channelbox;
mod levelprovider;
mod pwchannelobject;
mod pwnodemodel;
mod pwnodeobject;
mod volumebox;
mod window;

use self::application::PwvucontrolApplication;
use self::window::PwvucontrolWindow;

use self::config::{GETTEXT_PACKAGE, LOCALEDIR, RESOURCES_FILE};
use gettextrs::{bind_textdomain_codeset, bindtextdomain, gettext, textdomain};
use gtk::gio;
use gtk::prelude::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, glib::Enum)]
#[enum_type(name = "NodeType")]
pub enum NodeType {
    #[default]
    Undefined,
    Input,
    Output,
    Sink,
    Source,
}

fn main() -> gtk::glib::ExitCode {
    // init_glib_logger();
    // Set up gettext translations
    bindtextdomain(
        GETTEXT_PACKAGE,
        std::env::var_os("PWVUCONTROL_LOCALEDIR").unwrap_or(LOCALEDIR.into()),
    )
    .expect("Unable to bind the text domain");
    bind_textdomain_codeset(GETTEXT_PACKAGE, "UTF-8")
        .expect("Unable to set the text domain encoding");
    textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    gtk::init().unwrap_or_else(|_| panic!("Failed to initialize GTK."));
    gtk::glib::set_application_name("Pwvucontrol");

    let resources = gio::Resource::load("../data/resources/resources.gresource")
        .or(gio::Resource::load(RESOURCES_FILE))
        .expect(&gettext("Could not load resources"));

    // Load resources
    gio::resources_register(&resources);

    let app = PwvucontrolApplication::new();

    app.run()
}
