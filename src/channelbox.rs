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

use gtk::{glib, prelude::*, subclass::prelude::*};

use glib::Properties;
use std::cell::RefCell;

use crate::pwnodeobject::PwNodeObject;

mod imp {

    use std::cell::Cell;

    use super::*;
    use glib::{clone, ParamSpec, Value};

    #[derive(Debug, Default, gtk::CompositeTemplate, Properties)]
    #[template(resource = "/com/saivert/pwvucontrol/gtk/channelbox.ui")]
    #[properties(wrapper_type = super::PwChannelBox)]
    pub struct PwChannelBox {
        #[property(get, set, construct_only)]
        row_data: RefCell<Option<PwNodeObject>>,
        #[property(get, set)]
        channelname: RefCell<String>,
        #[property(get, set)]
        channelindex: Cell<u32>,
        #[property(get, set)]
        volume: Cell<f32>,

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

            item.connect_channel_volumes_notify(clone!(@weak self as widget => move |nodeobj| {
                let values = nodeobj.imp().channel_volumes_vec();
                let index = widget.channelindex.get();
                let channelname = crate::format::get_channel_name_for_position(index, nodeobj.imp().format());
                if let Some(volume) = values.get(index as usize) {
                    widget.obj().set_volume(volume.cbrt());
                    widget.obj().set_channelname(channelname);
                } else {
                    log::error!("channel volumes array out of bounds");
                }
            }));

            let adjustment = self.scale.adjustment();

            adjustment.connect_value_changed(clone!(@weak self as widget, @weak item => move |x| {
                let index = widget.channelindex.get();
                item.imp().set_channel_volume(index, x.value().powi(3) as f32);
            }));
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
    pub fn new(channelindex: u32, volume: f32, row_data: &PwNodeObject) -> Self {
        let channelname = crate::format::get_channel_name_for_position(channelindex, row_data.imp().format());

        glib::Object::builder()
            .property("channelindex", channelindex)
            .property("volume", volume)
            .property("row-data", row_data)
            .property("channelname", channelname)
            .build()
    }
}
