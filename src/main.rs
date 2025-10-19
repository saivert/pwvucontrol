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

    include!(concat!(env!("OUT_DIR"), "/config.rs"));
}
mod macros;

log_topic! {
   static TOPIC = "pwvucontrol";
}

mod application;
mod backend;
mod ui;

use std::{
    ffi::{OsStr, OsString},
    path::PathBuf,
};

use self::application::PwvucontrolApplication;

use self::config::{GETTEXT_PACKAGE, LOCALEDIR, RESOURCES_FILE};
use gettextrs::{bind_textdomain_codeset, bindtextdomain, gettext, textdomain};
use gtk::gio;
use wireplumber::log_topic;

fn path_override_from_env<K>(var: K, default: K, append: Option<K>) -> OsString
where
    K: AsRef<OsStr> + Default,
{
    let append = append.unwrap_or_default();
    if let Some(var) = std::env::var_os(var) {
        let path = PathBuf::from(&var);
        return path.join(append.as_ref()).into_os_string();
    }

    [default, append].iter().map(|x| x.as_ref()).collect::<PathBuf>().into_os_string()
}

fn main() -> gtk::glib::ExitCode {
    // Set up gettext translations
    bindtextdomain(GETTEXT_PACKAGE, path_override_from_env("PWVUCONTROL_LOCALEDIR", LOCALEDIR, None)).expect("Unable to bind the text domain");
    bind_textdomain_codeset(GETTEXT_PACKAGE, "UTF-8").expect("Unable to set the text domain encoding");
    textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    gtk::init().unwrap_or_else(|_| panic!("Failed to initialize GTK."));
    gtk::glib::set_application_name("Pwvucontrol");

    let resources = gio::Resource::load(path_override_from_env("PWVUCONTROL_RESOURCEDIR", "../data/resources", Some("resources.gresource")))
        .or(gio::Resource::load(RESOURCES_FILE))
        .unwrap_or_else(|_| panic!("{}", gettext("Could not load resources")));

    // Load resources
    gio::resources_register(&resources);

    PwvucontrolApplication::run()
}
