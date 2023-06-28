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
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
};

use adw::subclass::prelude::*;

use wireplumber as wp;

use crate::config::VERSION;
use crate::PwvucontrolWindow;

mod imp {
    use std::{str::FromStr, cell::Cell};


    use crate::pwnodeobject::PwNodeObject;

    use super::*;
    use once_cell::unsync::OnceCell;
    use wp::{pw::ProxyExt, plugin::*};

    #[derive(Default)]
    pub struct PwvucontrolApplication {
        pub(super) window: OnceCell<PwvucontrolWindow>,
        pub wp_core: OnceCell<wp::core::Core>,
        pub wp_object_manager: OnceCell<wp::registry::ObjectManager>,
        pub count: Cell<u32>,

    }

    #[glib::object_subclass]
    impl ObjectSubclass for PwvucontrolApplication {
        const NAME: &'static str = "PwvucontrolApplication";
        type Type = super::PwvucontrolApplication;
        type ParentType = adw::Application;
    }

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
            self.setup_wp_connection();
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

    impl PwvucontrolApplication {
        fn setup_wp_connection(&self) {
            wp::core::Core::init();

            wireplumber::Log::set_default_level("3");

            let props = wp::pw::Properties::new_string("media.category=Manager");

            let wp_core = wp::core::Core::new(Some(&glib::MainContext::default()), Some(props));
            let wp_om = wp::registry::ObjectManager::new();

            wp_core.connect();

            wp_core.load_component("libwireplumber-module-mixer-api", "module", None).expect("loadig mixer-api plugin");
            wp_core.load_component("libwireplumber-module-default-nodes-api", "module", None).expect("loadig mixer-api plugin");

            let plugin_names = vec!["mixer-api", "default-nodes-api"];

            glib::MainContext::default().spawn_local(clone!(@weak self as app, @weak wp_core as core, @weak wp_om as om => async move {
                for plugin_name in plugin_names {
                    if let Some(plugin) = Plugin::find(&core, plugin_name) {
                        let result = plugin.activate_future(PluginFeatures::ENABLED)
                        .await;
                        if result.is_err() {
                            wp::log::critical!("Cannot activate plugin {plugin_name}");
                        } else {
                            wp::log::info!("Activated plugin {plugin_name}");
                            let count = app.count.get() + 1;
                            app.count.set(count);
                            dbg!(count);
                            if count == 2 {
                                core.install_object_manager(&om);
                            }
                        }
                    } else {
                        wp::log::critical!("Cannot find plugin {plugin_name}");
                        app.obj().quit();
                    }
                }
            }));

            wp_om.add_interest_full(
                {
                    let interest = wp::registry::ObjectInterest::new_type(
                        wp::pw::Node::static_type(),
                    );
                    let variant = glib::Variant::from_str("('Stream/Output/Audio', 'Stream/Input/Audio', 'Audio/Device', 'Audio/Sink')").expect("variant");
                    interest.add_constraint(
                        wp::registry::ConstraintType::PwGlobalProperty,
                        "media.class",
                        wp::registry::ConstraintVerb::InList,
                        Some(&variant));
    
                    interest
                }
            );

            wp_om.request_object_features(
                wp::pw::Node::static_type(),
                wp::core::ObjectFeatures::ALL,
            );

            wp_om.request_object_features(
                wp::pw::GlobalProxy::static_type(),
                wp::core::ObjectFeatures::ALL,
            );

            wp_om.connect_object_added(
                clone!(@weak self as imp, @weak wp_core as core => move |_, object| {
                    if let Some(node) = object.dynamic_cast_ref::<wp::pw::Node>() {
                        wp::log::info!("added: {:?}", node.name());
                        let pwobj = PwNodeObject::new(node);
                        let window = imp.window.get().unwrap();
                        let model = &window.imp().nodemodel;
                        model.append(&pwobj);
                    } else {
                        unreachable!("Object must be one of the above, but is {:?} instead", object.type_());
                    }
                }),
            );

            wp_om.connect_object_removed(clone!(@weak self as imp => move |_, object| {
                if let Some(node) = object.dynamic_cast_ref::<wp::pw::Node>() {
                    wp::log::info!("removed: {:?} id: {}", node.name(), node.bound_id());
                    let window = imp.window.get().unwrap();
                    let model = &window.imp().nodemodel;
                    model.remove(node.bound_id());

                } else {
                    unreachable!("Object must be one of the above");
                }
            }));


            self.wp_core
                .set(wp_core)
                .expect("wp_core should only be set once during application activation");
            self.wp_object_manager
                .set(wp_om)
                .expect("wp_object_manager should only be set once during application activation");
   
        }

    }
}

glib::wrapper! {
    pub struct PwvucontrolApplication(ObjectSubclass<imp::PwvucontrolApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl PwvucontrolApplication {
    pub(super) fn new() -> Self {
        glib::Object::builder()
            .property("application-id", "com.saivert.pwvucontrol")
            .property("flags", &gio::ApplicationFlags::empty())
            .property("resource-base-path", &"/com/saivert/pwvucontrol")
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
}

impl Default for PwvucontrolApplication {
    fn default() -> Self {
        gio::Application::default()
            .expect("Could not get default GApplication")
            .downcast()
            .unwrap()
    }
}
