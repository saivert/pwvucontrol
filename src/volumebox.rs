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

use crate::pwnodeobject::PwNodeObject;

use glib::{self, clone, ParamSpec, Properties, Value};
use gtk::{gio, prelude::*, subclass::prelude::*};

use std::cell::RefCell;

use wireplumber as wp;

mod imp {

    use super::*;
    use crate::{
        application::PwvucontrolApplication, channelbox::PwChannelBox,
        pwchannelobject::PwChannelObject, window::PwvucontrolWindow, NodeType,
    };

    #[derive(Default, gtk::CompositeTemplate, Properties)]
    #[template(resource = "/com/saivert/pwvucontrol/gtk/volumebox.ui")]
    #[properties(wrapper_type = super::PwVolumeBox)]
    pub struct PwVolumeBox {
        #[property(get, set, construct_only)]
        row_data: RefCell<Option<PwNodeObject>>,

        channelmodel: gio::ListStore,

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
        #[template_child]
        pub channellock: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub outputdevice_dropdown: TemplateChild<gtk::DropDown>,
        #[template_child]
        pub mainvolumescale: TemplateChild<gtk::Scale>,
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
                .transform_to::<f32, f64, _>(|_, y| Some(y.cbrt() as f64))
                .transform_from::<f64, f32, _>(|_, y| Some((y * y * y) as f32))
                .build();

            item.bind_property("formatstr", &self.format.get(), "label")
                .sync_create()
                .build();

            item.bind_property("channellock", &self.channellock.get(), "active")
                .sync_create()
                .bidirectional()
                .build();

            item.bind_property("mainvolume", &self.mainvolumescale.adjustment(), "value")
                .sync_create()
                .bidirectional()
                .transform_to::<f32, f64, _>(|_, y| Some(y.cbrt() as f64))
                .transform_from::<f64, f32, _>(|_, y| Some((y * y * y) as f32))
                .build();

            if matches!(item.nodetype(), /* NodeType::Input | */ NodeType::Output) { //TODO: Implement support for Audio/Source switching for NodeType::Input
                let factory = gtk::SignalListItemFactory::new();
                factory.connect_setup(|_, item| {
                    let label = gtk::Label::new(None);
                    label.set_ellipsize(gtk::pango::EllipsizeMode::End);

                    item.property_expression("item")
                        .chain_property::<PwNodeObject>("name")
                        .bind(&label, "label", gtk::Widget::NONE);

                    item.set_child(Some(&label));
                });

                self.outputdevice_dropdown.set_factory(Some(&factory));


                let listfactory = gtk::SignalListItemFactory::new();
                listfactory.connect_setup(|_, item| {
                    let label = gtk::Label::new(None);
                    label.set_xalign(0.0);

                    item.property_expression("item")
                        .chain_property::<PwNodeObject>("name")
                        .bind(&label, "label", gtk::Widget::NONE);

                    item.set_child(Some(&label));
                });

                self.outputdevice_dropdown.set_list_factory(Some(&listfactory));


                let win = PwvucontrolWindow::default();
                let model = &win.imp().nodemodel;

                let filter = gtk::CustomFilter::new(|x| {
                    if let Some(o) = x.downcast_ref::<PwNodeObject>() {
                        return o.nodetype() == crate::NodeType::Sink;
                    }
                    false
                });
                let ref filterlistmodel =
                    gtk::FilterListModel::new(Some(model.clone()), Some(filter));

                self.outputdevice_dropdown.set_enable_search(true);
                self.outputdevice_dropdown.set_expression(
                    Some(gtk::PropertyExpression::new(PwNodeObject::static_type(), gtk::Expression::NONE, "name"))
                );

                self.outputdevice_dropdown.set_model(Some(filterlistmodel));

                fn find_position_with_boundid_match(model: &impl IsA<gio::ListModel>, id: u32) -> Option<u32> {
                    model.iter::<glib::Object>().enumerate().find_map(|(x, y)|{
                        if let Ok(d) = y {
                            if let Some(o) = d.downcast_ref::<PwNodeObject>() {
                                if o.boundid() == id {
                                    return Some(x as u32);
                                }
                            }
                        }
                        None
                    })
                } 

                if let Some(deftarget) = item.default_target() {
                    let pos = find_position_with_boundid_match(filterlistmodel, deftarget.boundid());
                    self.outputdevice_dropdown.set_selected(pos.unwrap_or(gtk::ffi::GTK_INVALID_LIST_POSITION));
                } else {
                    let app = PwvucontrolApplication::default();
                    let core = app.imp().wp_core.get().expect("Core");
                    let defaultnodesapi = wp::plugin::Plugin::find(&core, "default-nodes-api").expect("Get mixer-api");
                    let id: u32 = defaultnodesapi.emit_by_name("get-default-node", &[&"Audio/Sink"]);
                    if id != u32::MAX {
                        let pos = find_position_with_boundid_match(filterlistmodel, id);
                        self.outputdevice_dropdown.set_selected(pos.unwrap_or(gtk::ffi::GTK_INVALID_LIST_POSITION));
                    }
                }



                self.outputdevice_dropdown.connect_notify_local(Some("selected-item"), clone!(@weak item as nodeobj => move |dropdown, _| {
                    if let Some(item) = dropdown.selected_item() {
                        if let Some(item) = item.downcast_ref::<PwNodeObject>() {
                            nodeobj.set_default_target(item);
                        }
                    }
                }));

            } else {
                self.outputdevice_dropdown.hide();
            }

            log::info!("binding model");

            self.channel_listbox.bind_model(
                Some(&self.channelmodel),
                clone!(@weak self as widget => @default-panic, move |item| {
                    PwChannelBox::new(
                        item.clone().downcast_ref::<PwChannelObject>()
                        .expect("RowData is of wrong type")
                    )
                    .upcast::<gtk::Widget>()
                }),
            );

            item.connect_local("format", false, clone!(@weak self as widget, @weak item as nodeobj => @default-panic, move |_| {
                let values = nodeobj.channel_volumes_vec();
                let oldlen = widget.channelmodel.n_items();

                wp::log::info!("channel volumes notify, values.len = {}, oldlen = {}", values.len(), oldlen);

                if values.len() as u32 != oldlen {
                    widget.channelmodel.remove_all();
                    for (i,v) in values.iter().enumerate() {
                        widget.channelmodel.append(&PwChannelObject::new(i as u32, *v, &nodeobj));
                    }

                    return None;
                }
                None
            }));

            item.connect_channel_volumes_notify(clone!(@weak self as widget => move |nodeobj| {
                let values = nodeobj.channel_volumes_vec();
                for (i,v) in values.iter().enumerate() {
                    if let Some(item) = widget.channelmodel.item(i as u32) {
                        let channelobj = item.downcast_ref::<PwChannelObject>()
                            .expect("RowData is of wrong type");
                        channelobj.imp().block_volume_send.set(true);
                        channelobj.set_volume(v);
                        channelobj.imp().block_volume_send.set(false);
                    }
                }
            }));

            self.revealer
                .connect_child_revealed_notify(clone!(@weak self as widget => move |_| {
                    widget.obj().grab_focus();
                }));

            self.level_bar
                .add_offset_value(gtk::LEVEL_BAR_OFFSET_LOW, 0.0);
            self.level_bar
                .add_offset_value(gtk::LEVEL_BAR_OFFSET_HIGH, 0.0);
            self.level_bar
                .add_offset_value(gtk::LEVEL_BAR_OFFSET_FULL, 0.0);
        }
    }
    impl WidgetImpl for PwVolumeBox {}
    impl ListBoxRowImpl for PwVolumeBox {}

    #[gtk::template_callbacks]
    impl PwVolumeBox {
        #[template_callback]
        fn invert_bool(&self, value: bool) -> bool {
            !value
        }
    }
}

glib::wrapper! {
    pub struct PwVolumeBox(ObjectSubclass<imp::PwVolumeBox>)
        @extends gtk::Widget, gtk::ListBoxRow,
        @implements gtk::Actionable;
}

impl PwVolumeBox {
    pub(crate) fn new(row_data: &impl glib::IsA<PwNodeObject>) -> Self {
        glib::Object::builder()
            .property("row-data", &row_data)
            .build()
    }
}
