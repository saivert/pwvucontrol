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
    glib,
    prelude::*,
    subclass::prelude::*,
};



use std::cell::RefCell;
use glib::Properties;

use crate::pwnodeobject::PwNodeObject;

mod imp {

    use crate::channelbox::PwChannelBox;

    use super::*;
    use glib::{ParamSpec, Value, clone};

    #[derive(Default, gtk::CompositeTemplate, Properties)]
    #[template(resource = "/com/saivert/pwvucontrol/gtk/volumebox.ui")]
    #[properties(wrapper_type = super::PwVolumeBox)]
    pub struct PwVolumeBox {
        #[property(get, set, construct_only)]
        row_data: RefCell<Option<PwNodeObject>>,

        channel_widgets: RefCell<Vec<PwChannelBox>>,

        // Template widgets
        #[template_child]
        pub icon: TemplateChild<gtk::Image>,
        #[template_child]
        pub title_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub subtitle_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub volume_scale: TemplateChild<gtk::Scale>,
        #[template_child]
        pub level_bar: TemplateChild<gtk::LevelBar>,
        #[template_child]
        pub mutebtn: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub channel_listbox: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub format: TemplateChild<gtk::Label>,
        #[template_child]
        pub revealer: TemplateChild<gtk::Revealer>,
    }


    #[glib::object_subclass]
    impl ObjectSubclass for PwVolumeBox {
        const NAME: &'static str = "PwVolumeBox";
        type Type = super::PwVolumeBox;
        type ParentType = gtk::ListBoxRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }


    impl ObjectImpl for PwVolumeBox {
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

            item.bind_property("name", &self.title_label.get(), "label")
                .sync_create()
                .build();

            item.bind_property("description", &self.subtitle_label.get(), "label")
                .sync_create()
                .build();

            item.bind_property("mute", &self.mutebtn.get(), "active")
                .sync_create()
                .bidirectional()
                .build();

            item.bind_property("volume", &self.volume_scale.adjustment(), "value")
                .sync_create()
                .bidirectional()
                .transform_to::<f32, f64, _>(|_, y|Some(y.cbrt() as f64))
                .transform_from::<f64, f32, _>(|_, y|Some((y*y*y) as f32))
                .build();

            item.bind_property("formatstr", &self.format.get(), "label")
                .sync_create()
                .build();


            self.create_channel_volumes_widgets();

            item.connect_channel_volumes_notify(clone!(@weak self as widget => move |nodeobj| {
                let values = nodeobj.channel_volumes();
                let channel_widgets_len = {
                    let channel_widgets = widget.channel_widgets.borrow();
                    channel_widgets.len()
                };

                if let Some(f) = nodeobj.format() {
                    nodeobj.set_formatstr(format!("{}ch {}Hz {}", f.channels, f.rate, crate::format::format_to_string(f.format)));
                }

                if values.len() != channel_widgets_len {
                    widget.create_channel_volumes_widgets();
                    return;
                }
            }));

            self.revealer.connect_child_revealed_notify(clone!(@weak self as widget => move |_| {
                widget.obj().grab_focus();
            }));

            self.level_bar.add_offset_value(gtk::LEVEL_BAR_OFFSET_LOW, 0.0);
            self.level_bar.add_offset_value(gtk::LEVEL_BAR_OFFSET_HIGH, 0.0);
            self.level_bar.add_offset_value(gtk::LEVEL_BAR_OFFSET_FULL, 0.0);
        }

    
    }
    impl WidgetImpl for PwVolumeBox {}
    impl ListBoxRowImpl for PwVolumeBox {}

    #[gtk::template_callbacks]
    impl PwVolumeBox {
        fn clear_channel_volumes_listbox(&self) {
            let list_box: gtk::ListBox = self.channel_listbox.get();

            while let Some(row) = list_box.last_child() {
                list_box.remove(&row);
            }

            self.channel_widgets.borrow_mut().clear();
        }

        fn create_channel_volumes_widgets(&self) {
            self.clear_channel_volumes_listbox();
            let item = self.row_data.borrow();
            let item = item.as_ref().cloned().unwrap();

            let valuesvec = item.channel_volumes_vec();
            for (i,volume) in (0..).zip(valuesvec.iter()) {
                let mut list = self.channel_widgets.borrow_mut();
                let channelbox = PwChannelBox::new(i as u32, *volume, &item);
                list.push(channelbox);
                self.channel_listbox.append(list.last().unwrap());
            }
        }
    }

}

glib::wrapper! {
    pub struct PwVolumeBox(ObjectSubclass<imp::PwVolumeBox>)
        @extends gtk::Widget, gtk::ListBoxRow,
        @implements gtk::Actionable;
}

impl PwVolumeBox {
    pub fn new(row_data: &PwNodeObject) -> Self {
        glib::Object::builder()
            .property("row-data", &row_data)
            .build()
    }

}
