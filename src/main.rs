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
mod manager;
mod withdefaultlistmodel;
mod output_dropdown;


use std::{ffi::{OsStr, OsString}, path::PathBuf};

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

fn path_override_from_env<K>(var: K, default: K, append: Option<K>) -> OsString
where
    K: AsRef<OsStr> + Default,
{
    let append = append.unwrap_or_default();
    if let Some(var) = std::env::var_os(var) {
        let path = PathBuf::from(&var);
        return path.join(append.as_ref()).into_os_string();
    }

    [default, append]
        .iter()
        .map(|x| x.as_ref())
        .collect::<PathBuf>()
        .into_os_string()
}

fn main() -> gtk::glib::ExitCode {
    // init_glib_logger();
    // Set up gettext translations
    bindtextdomain(
        GETTEXT_PACKAGE,
        path_override_from_env("PWVUCONTROL_LOCALEDIR", LOCALEDIR, None),
    )
    .expect("Unable to bind the text domain");
    bind_textdomain_codeset(GETTEXT_PACKAGE, "UTF-8")
        .expect("Unable to set the text domain encoding");
    textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    gtk::init().unwrap_or_else(|_| panic!("Failed to initialize GTK."));
    gtk::glib::set_application_name("Pwvucontrol");

    let resources = gio::Resource::load(path_override_from_env(
        "PWVUCONTROL_RESOURCEDIR",
        "../data/resources",
        Some("resources.gresource"),
    ))
    .or(gio::Resource::load(RESOURCES_FILE))
    .expect(&gettext("Could not load resources"));

    // Load resources
    gio::resources_register(&resources);

    let css = gtk::CssProvider::new();
    css.load_from_data(
        r#"
    levelbar block.filled {
        filter: blur(2px);
    }
    "#,
    );

    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().unwrap(),
        &css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let app = PwvucontrolApplication::new();

    app.run()
}
