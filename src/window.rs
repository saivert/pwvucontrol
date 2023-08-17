/* window.rs
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
    prelude::*,
    subclass::prelude::*,
};
use glib::{self, clone};
use adw::subclass::prelude::*;

use crate::application::PwvucontrolApplication;
use crate::config::{APP_ID, PROFILE};

use wireplumber as wp;

pub(crate) enum PwvucontrolWindowView {
    Connected,
    Disconnected
}
mod imp {
    use super::*;

    use crate::{volumebox::PwVolumeBox, pwnodeobject::PwNodeObject};

    #[derive(Debug, gtk::CompositeTemplate)]
    #[template(resource = "/com/saivert/pwvucontrol/gtk/window.ui")]
    pub struct PwvucontrolWindow {
        #[template_child]
        pub header_bar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub stack: TemplateChild<adw::ViewStack>,
        #[template_child]
        pub playbacklist: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub recordlist: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub outputlist: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub viewstack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub reconnectbtn: TemplateChild<gtk::Button>,

        pub settings: gio::Settings,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PwvucontrolWindow {
        const NAME: &'static str = "PwvucontrolWindow";
        type Type = super::PwvucontrolWindow;
        type ParentType = adw::ApplicationWindow;

        fn new() -> Self {
            Self {
                header_bar: TemplateChild::default(),
                stack: TemplateChild::default(),
                playbacklist: TemplateChild::default(),
                recordlist: TemplateChild::default(),
                outputlist: TemplateChild::default(),
                viewstack: TemplateChild::default(),
                reconnectbtn: TemplateChild::default(),
                settings: gio::Settings::new(APP_ID)
            }
        }

        fn class_init(klass: &mut Self::Class) {
            PwVolumeBox::ensure_type();
            
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }

    }


    impl ObjectImpl for PwvucontrolWindow {
        fn constructed(&self) {
            self.parent_constructed();

            // Devel Profile
            if PROFILE == "Devel" {
                self.obj().add_css_class("devel");
            }

            self.obj().setup_scroll_blocker(self.playbacklist.get());
            self.obj().setup_scroll_blocker(self.recordlist.get());
            self.obj().setup_scroll_blocker(self.outputlist.get());

            let app = PwvucontrolApplication::default();

            let model = &app.imp().nodemodel;
            let window = self;

            let filter = gtk::CustomFilter::new(|x| {
                if let Some(o) = x.downcast_ref::<PwNodeObject>() {
                    return o.nodetype() == crate::NodeType::Output;
                }
                false
            });
            let filterlistmodel = &gtk::FilterListModel::new(Some(model.clone()), Some(filter));

            self.playbacklist.bind_model(
                Some(filterlistmodel),
                clone!(@weak window => @default-panic, move |item| {
                    PwVolumeBox::new(
                        item.downcast_ref::<PwNodeObject>()
                            .expect("RowData is of wrong type"),
                    )
                    .upcast::<gtk::Widget>()
                }),
            );

            let filter = gtk::CustomFilter::new(|x| {
                if let Some(o) = x.downcast_ref::<PwNodeObject>() {
                    return o.nodetype() == crate::NodeType::Input;
                }
                false
            });
            let filterlistmodel = &gtk::FilterListModel::new(Some(model.clone()), Some(filter));

            self.recordlist.bind_model(
                Some(filterlistmodel),
                clone!(@weak window => @default-panic, move |item| {
                    PwVolumeBox::new(
                        item.downcast_ref::<PwNodeObject>()
                            .expect("RowData is of wrong type"),
                    )
                    .upcast::<gtk::Widget>()
                }),
            );

            let filter = gtk::CustomFilter::new(|x| {
                if let Some(o) = x.downcast_ref::<PwNodeObject>() {
                    return o.nodetype() == crate::NodeType::Sink;
                }
                false
            });
            let filterlistmodel = &gtk::FilterListModel::new(Some(model.clone()), Some(filter));

            self.outputlist.bind_model(
                Some(filterlistmodel),
                clone!(@weak window => @default-panic, move |item| {
                    PwVolumeBox::new(
                        item.downcast_ref::<PwNodeObject>()
                            .expect("RowData is of wrong type"),
                    )
                    .upcast::<gtk::Widget>()
                }),
            );

            self.reconnectbtn.connect_clicked(|_| {
                let app = PwvucontrolApplication::default();
                if let Some(core) = app.imp().wp_core.get() {
                    core.connect();
                }
            });

            self.obj().load_window_state();

        }
    }
    impl WidgetImpl for PwvucontrolWindow {}
    impl WindowImpl for PwvucontrolWindow {
        // save window state on delete event

        fn close_request(&self) -> glib::Propagation {
            if let Err(err) = self.obj().save_window_size() {
                wp::log::warning!("Failed to save window state, {}", &err);
            }
            self.parent_close_request()
        }
    }
    impl ApplicationWindowImpl for PwvucontrolWindow {}
    impl AdwApplicationWindowImpl for PwvucontrolWindow {}

    #[gtk::template_callbacks]
    impl PwvucontrolWindow {
    }
}

glib::wrapper! {
    pub struct PwvucontrolWindow(ObjectSubclass<imp::PwvucontrolWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl PwvucontrolWindow {
    pub fn new(application: &PwvucontrolApplication) -> Self {
        glib::Object::builder()
        .property("application", application)
        .build()
    }

    pub(crate) fn set_view(&self, view: PwvucontrolWindowView) {
        let imp = self.imp();
        match view {
            PwvucontrolWindowView::Connected => imp.viewstack.set_visible_child_name("connected"),
            PwvucontrolWindowView::Disconnected => imp.viewstack.set_visible_child_name("disconnected"),
        }
        
    }

    fn save_window_size(&self) -> Result<(), glib::BoolError> {
        let settings = &self.imp().settings;

        let size = self.default_size();

        settings.set_int("window-width", size.0)?;
        settings.set_int("window-height", size.1)?;

        settings.set_boolean("is-maximized", self.is_maximized())?;

        Ok(())
    }

    fn load_window_state(&self) {
        let settings = &self.imp().settings;

        let width = settings.int("window-width");
        let height = settings.int("window-height");
        let is_maximized = settings.boolean("is-maximized");

        self.set_default_size(width, height);

        if is_maximized {
            self.maximize();
        }

    }

    /// This prevents child widgets from capturing scroll events
    fn setup_scroll_blocker(&self, listbox: gtk::ListBox) {
            let scrolledwindow = listbox.ancestor(gtk::ScrolledWindow::static_type()).and_then(|x|{
                x.downcast::<gtk::ScrolledWindow>().ok()
            }).expect("downcast to scrolled window");

            let ecs = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
            ecs.set_propagation_phase(gtk::PropagationPhase::Capture);
            ecs.set_propagation_limit(gtk::PropagationLimit::SameNative);

            // Need to actually handle the scroll event in order to block propagation
            ecs.connect_local("scroll", false, clone!(@weak scrolledwindow => @default-return None, move |v| {
                let y: f64 = v.get(2).unwrap().get().unwrap();

                // No way to redirect this event to underlying widget so we need to reimplement the scroll handling
                let adjustment = scrolledwindow.vadjustment();
                
                if (adjustment.upper() - adjustment.page_size()).abs() < f64::EPSILON {
                    return Some(false.to_value());
                }
                
                adjustment.set_value(adjustment.value() + y*adjustment.page_size().powf(2.0 / 3.0));
                
                Some(true.to_value())
            }));
            scrolledwindow.add_controller(ecs);
    }
}

impl Default for PwvucontrolWindow {
    fn default() -> Self {
        PwvucontrolApplication::default()
            .active_window()
            .unwrap()
            .downcast()
            .unwrap()
    }
}
