/* application.rs
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

 use gtk::{
    gio,
    glib,
    prelude::*,
    subclass::prelude::*,
};

use adw::subclass::prelude::*;

use crate::{config::{VERSION, APP_ID}, manager::PwvucontrolManager};
use crate::PwvucontrolWindow;

mod imp {
    use std::cell::RefCell;

    use super::*;
    use once_cell::unsync::OnceCell;


    pub struct PwvucontrolApplication {
        pub(super) window: OnceCell<PwvucontrolWindow>,
        pub(super) manager: RefCell<Option<PwvucontrolManager>>,

    }

    #[glib::object_subclass]
    impl ObjectSubclass for PwvucontrolApplication {
        const NAME: &'static str = "PwvucontrolApplication";
        type Type = super::PwvucontrolApplication;
        type ParentType = adw::Application;

        fn new() -> PwvucontrolApplication {
            PwvucontrolApplication {
                window: OnceCell::default(),
                manager: RefCell::default(),
            }
        }
    }

    impl ObjectImpl for PwvucontrolApplication {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_gactions();
            obj.set_accels_for_action("app.quit", &["<primary>q"]);

            self.manager.replace(Some(crate::manager::PwvucontrolManager::new(self.obj().as_ref())));

        }
    }

    impl ApplicationImpl for PwvucontrolApplication {
        // We connect to the activate callback to create a window when the application
        // has been launched. Additionally, this callback notifies us when the user
        // tries to launch a "second instance" of the application. When they try
        // to do that, we'll just present any existing window.
        fn activate(&self) {
            let window = self
            .window
            .get()
            .expect("Should always be initialized in gio_application_startup");

            // Ask the window manager/compositor to present the window
            window.present();
        }

        fn startup(&self) {
            self.parent_startup();
    
            let window = PwvucontrolWindow::new(&self.obj());
            self.window
                .set(window)
                .expect("Failed to initialize application window");
        }
    }

    impl GtkApplicationImpl for PwvucontrolApplication {}
    impl AdwApplicationImpl for PwvucontrolApplication {}

    impl PwvucontrolApplication {}
}

glib::wrapper! {
    pub struct PwvucontrolApplication(ObjectSubclass<imp::PwvucontrolApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl PwvucontrolApplication {
    pub(super) fn new() -> Self {
        glib::Object::builder()
            .property("application-id", APP_ID)
            .property("flags", gio::ApplicationFlags::empty())
            .property("resource-base-path", "/com/saivert/pwvucontrol")
            .build()
    }

    fn setup_gactions(&self) {
        let quit_action = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| app.quit())
            .build();
        let about_action = gio::ActionEntry::builder("about")
            .activate(move |app: &Self, _, _| app.show_about())
            .build();
        self.add_action_entries([quit_action, about_action])
    }

    fn show_about(&self) {
        let window = self.active_window().unwrap();
        let about = adw::AboutWindow::builder()
            .transient_for(&window)
            .application_name("pwvucontrol")
            .application_icon("com.saivert.pwvucontrol")
            .developer_name("Nicolai Syvertsen")
            .version(VERSION)
            .developers(vec!["Nicolai Syvertsen"])
            .copyright("© 2023 Nicolai Syvertsen")
            .build();

        about.present();
    }

    pub fn manager(&self) -> Option<PwvucontrolManager> {
        self.imp().manager.borrow().as_ref().cloned()
    }
}

impl Default for PwvucontrolApplication {
    fn default() -> Self {
        gio::Application::default()
        .expect("Could not get default GApplication")
        .downcast()
        .unwrap()
}
}
