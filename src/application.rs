// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{
    backend::PwvucontrolManager,
    config::{APP_ID, VERSION},
    ui::PwvucontrolWindow,
};
use adw::subclass::prelude::*;
use glib::{ExitCode, OptionArg, OptionFlags};
use gtk::{gio, prelude::*};
use std::cell::{Cell, OnceCell};

mod imp {
    use super::*;

    #[derive(glib::Properties)]
    #[properties(wrapper_type = super::PwvucontrolApplication)]
    pub struct PwvucontrolApplication {
        pub window: OnceCell<PwvucontrolWindow>,
        #[property(get)]
        pub manager: PwvucontrolManager,

        pub(super) tab: Cell<i32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PwvucontrolApplication {
        const NAME: &'static str = "PwvucontrolApplication";
        type Type = super::PwvucontrolApplication;
        type ParentType = adw::Application;

        fn new() -> PwvucontrolApplication {
            PwvucontrolApplication { window: OnceCell::default(), manager: PwvucontrolManager::new(), tab: Default::default() }
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
            let window = self.window.get().expect("Should always be initialized in gio_application_startup");

            // Ask the window manager/compositor to present the window
            window.present();

            window.select_tab(self.tab.get());
        }

        fn startup(&self) {
            self.parent_startup();

            let window = PwvucontrolWindow::new(&self.obj());
            self.window.set(window).expect("Failed to initialize application window");
        }

        fn command_line(&self, command_line: &gio::ApplicationCommandLine) -> ExitCode {
            let tab_arg = command_line.options_dict().lookup::<i32>("tab");
            if let Ok(Some(tab)) = tab_arg {
                self.tab.set(tab);
            }

            self.activate();

            ExitCode::SUCCESS
        }

        fn handle_local_options(&self, options: &glib::VariantDict) -> ExitCode {
            if options.lookup_value("version", None).is_some() {
                println!("pwvucontrol version {}", VERSION);
                return ExitCode::SUCCESS;
            }

            self.parent_handle_local_options(options)
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
    pub fn run(/*args: crate::Args*/) -> ExitCode {
        let app: Self = glib::Object::builder()
            .property("application-id", APP_ID)
            .property("flags", gio::ApplicationFlags::HANDLES_COMMAND_LINE)
            .property("resource-base-path", "/com/saivert/pwvucontrol")
            .build();

        app.add_main_option("tab", glib::Char('t' as i8), OptionFlags::NONE, OptionArg::Int, "Select tab to open.", Some("number"));
        app.add_main_option("version", glib::Char('v' as i8), OptionFlags::NONE, OptionArg::None, "Show version.", None);

        ApplicationExtManual::run(&app)
    }

    fn setup_gactions(&self) {
        let quit_action = gio::ActionEntryBuilder::new("quit").activate(move |app: &Self, _, _| app.quit()).build();
        let about_action = gio::ActionEntryBuilder::new("about").activate(move |app: &Self, _, _| app.show_about()).build();
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
}

impl Default for PwvucontrolApplication {
    fn default() -> Self {
        gio::Application::default().expect("Could not get default GApplication").downcast().unwrap()
    }
}
