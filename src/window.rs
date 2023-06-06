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
    glib,
    prelude::*,
    subclass::prelude::*,
};

use adw::subclass::prelude::*;

use crate::application::PwvucontrolApplication;


mod imp {
    use std::cell::Cell;

    use gtk::glib::clone;

    use crate::{volumebox::PwVolumeBox, pwnodemodel::PwNodeModel, pwnodeobject::PwNodeObject};

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/saivert/pwvucontrol/window.ui")]
    pub struct PwvucontrolWindow {
        pub counter: Cell<u32>,
        // Template widgets
        #[template_child]
        pub header_bar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub stack: TemplateChild<adw::ViewStack>,
        #[template_child]
        pub btn: TemplateChild<gtk::Button>,
        #[template_child]
        pub playbacklist: TemplateChild<gtk::ListBox>,
        pub nodemodel: PwNodeModel,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PwvucontrolWindow {
        const NAME: &'static str = "PwvucontrolWindow";
        type Type = super::PwvucontrolWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            PwVolumeBox::ensure_type();
            
            klass.bind_template();
            klass.bind_template_callbacks();
            // Self::Type::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }

    }


    impl ObjectImpl for PwvucontrolWindow {
        fn constructed(&self) {
            self.parent_constructed();

            let model = &self.nodemodel;
            let window = self;
            self.playbacklist.bind_model(
                Some(model),
                clone!(@weak window => @default-panic, move |item| {
                    PwVolumeBox::new(
                        item.downcast_ref::<PwNodeObject>()
                            .expect("RowData is of wrong type"),
                    )
                    .upcast::<gtk::Widget>()
                }),
            );

        }
    }
    impl WidgetImpl for PwvucontrolWindow {}
    impl WindowImpl for PwvucontrolWindow {}
    impl ApplicationWindowImpl for PwvucontrolWindow {}
    impl AdwApplicationWindowImpl for PwvucontrolWindow {}

    #[gtk::template_callbacks]
    impl PwvucontrolWindow {
        #[template_callback]
        fn ok_button_clicked(&self) {
            let button = &self.btn;
            let counter = self.counter.get() + 1;
            self.counter.set(counter);
            button.set_label(&format!("Hello World! {counter}"));
        }
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

}

