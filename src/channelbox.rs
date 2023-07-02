/* channelbox.rs
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

use crate::pwchannelobject::PwChannelObject;

mod imp {

    use super::*;
    use std::cell::RefCell;
    use gtk::{prelude::*, subclass::prelude::*};
    use glib::{ParamSpec, Value, Properties};
    
    #[derive(Debug, Default, gtk::CompositeTemplate, Properties)]
    #[template(resource = "/com/saivert/pwvucontrol/gtk/channelbox.ui")]
    #[properties(wrapper_type = super::PwChannelBox)]
    pub struct PwChannelBox {
        #[property(get, set, construct_only)]
        row_data: RefCell<Option<PwChannelObject>>,

        // Template widgets
        #[template_child]
        pub label: TemplateChild<gtk::Label>,
        #[template_child]
        pub scale: TemplateChild<gtk::Scale>,

    }

    #[glib::object_subclass]
    impl ObjectSubclass for PwChannelBox {
        const NAME: &'static str = "PwChannelBox";
        type Type = super::PwChannelBox;
        type ParentType = gtk::ListBoxRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PwChannelBox {
        fn properties() -> &'static [ParamSpec] {
            Self::derived_properties()
        }
        fn set_property(&self, id: usize, value: &Value, pspec: &ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }
        fn property(&self, id: usize, pspec: &ParamSpec) -> Value {
            self.derived_property(id, pspec)
        }

        fn constructed(&self) {
            self.parent_constructed();

            let item = self.row_data.borrow();
            let item = item.as_ref().cloned().unwrap();

            item.bind_property("volume", &self.scale.adjustment(), "value")
                .sync_create()
                .bidirectional()
                .transform_to::<f32, f64, _>(|_, y|Some(y.cbrt() as f64))
                .transform_from::<f64, f32, _>(|_, y|Some((y*y*y) as f32))
                .build();

            item.bind_property("name", &self.label.get(), "label")
                .sync_create()
                .build();
        }


    }
    impl WidgetImpl for PwChannelBox {}
    impl ListBoxRowImpl for PwChannelBox {}

    #[gtk::template_callbacks]
    impl PwChannelBox {}
}

glib::wrapper! {
    pub struct PwChannelBox(ObjectSubclass<imp::PwChannelBox>)
        @extends gtk::Widget, gtk::ListBoxRow,
        @implements gtk::Actionable;
}

impl PwChannelBox {
    pub(crate) fn new(channelobj: &PwChannelObject) -> Self {
        glib::Object::builder()
            .property("row-data", channelobj)
            .build()
    }
}
