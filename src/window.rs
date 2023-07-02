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

mod imp {
    use super::*;

    use crate::{volumebox::PwVolumeBox, pwnodemodel::PwNodeModel, pwnodeobject::PwNodeObject};

    #[derive(Debug, Default, gtk::CompositeTemplate)]
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

            let filter = gtk::CustomFilter::new(|x| {
                if let Some(o) = x.downcast_ref::<PwNodeObject>() {
                    return o.nodetype() == crate::NodeType::Output;
                }
                false
            });
            let ref filterlistmodel = gtk::FilterListModel::new(Some(model.clone()), Some(filter));

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
            let ref filterlistmodel = gtk::FilterListModel::new(Some(model.clone()), Some(filter));

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
            let ref filterlistmodel = gtk::FilterListModel::new(Some(model.clone()), Some(filter));

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

        }
    }
    impl WidgetImpl for PwvucontrolWindow {}
    impl WindowImpl for PwvucontrolWindow {}
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
