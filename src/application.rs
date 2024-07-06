// SPDX-License-Identifier: GPL-3.0-or-later

use glib::ExitCode;
use gtk::{gio, prelude::*, subclass::prelude::*};
use adw::{subclass::prelude::*, prelude::*};
use std::cell::OnceCell;
use crate::{
    config::{APP_ID, VERSION},
    backend::PwvucontrolManager,
    ui::PwvucontrolWindow,
};

mod imp {
    use super::*;

    #[derive(glib::Properties)]
    #[properties(wrapper_type = super::PwvucontrolApplication)]
    pub struct PwvucontrolApplication {
        pub window: OnceCell<PwvucontrolWindow>,
        #[property(get)]
        pub manager: PwvucontrolManager,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PwvucontrolApplication {
        const NAME: &'static str = "PwvucontrolApplication";
        type Type = super::PwvucontrolApplication;
        type ParentType = adw::Application;

        fn new() -> PwvucontrolApplication {
            PwvucontrolApplication {
                window: OnceCell::default(),
                manager: PwvucontrolManager::new(),
            }
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for PwvucontrolApplication {
        fn constructed(&self) {
            self.parent_constructed();
            
            let obj = self.obj();
            obj.setup_gactions();
            obj.set_accels_for_action("app.quit", &["<primary>q"]);
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
    pub fn run() -> ExitCode {
        let app: Self = glib::Object::builder()
        .property("application-id", APP_ID)
        .property("flags", gio::ApplicationFlags::empty())
        .property("resource-base-path", "/com/saivert/pwvucontrol")
        .build();

        ApplicationExtManual::run(&app)
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
        let about = adw::AboutDialog::builder()
            .application_name("pwvucontrol")
            .application_icon("com.saivert.pwvucontrol")
            .developer_name("Nicolai Syvertsen")
            .version(VERSION)
            .developers(vec!["Nicolai Syvertsen"])
            .copyright("© 2023 Nicolai Syvertsen")
            .build();

        about.present(&window);
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
