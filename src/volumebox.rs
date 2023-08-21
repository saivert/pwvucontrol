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

use crate::{application::PwvucontrolApplication, pwnodeobject::PwNodeObject};

use glib::{self, clone, ControlFlow, Properties};
use gtk::{gio, prelude::*, subclass::prelude::*};

use std::cell::RefCell;

use wireplumber as wp;

mod imp {

    use std::cell::Cell;

    use glib::SignalHandlerId;
    use once_cell::sync::OnceCell;
    use wp::pw::MetadataExt;

    use super::*;
    use crate::{
        channelbox::PwChannelBox, levelprovider::LevelbarProvider,
        pwchannelobject::PwChannelObject, NodeType,
    };

    #[derive(Default, gtk::CompositeTemplate, Properties)]
    #[template(resource = "/com/saivert/pwvucontrol/gtk/volumebox.ui")]
    #[properties(wrapper_type = super::PwVolumeBox)]
    pub struct PwVolumeBox {
        #[property(get, set, construct_only)]
        pub(super) row_data: RefCell<Option<PwNodeObject>>,

        #[property(get, set, construct_only)]
        channelmodel: OnceCell<gio::ListStore>,

        pub(super) outputdevice_dropdown_block_signal: Cell<bool>,
        metadata_changed_event: Cell<Option<SignalHandlerId>>,
        levelbarprovider: OnceCell<LevelbarProvider>,
        timeoutid: Cell<Option<glib::SourceId>>,
        pub(super) level: Cell<f32>,

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

    #[glib::derived_properties]
    impl ObjectImpl for PwVolumeBox {
        fn constructed(&self) {
            fn linear_to_cubic(_binding: &glib::Binding, i: f32) -> Option<f64> {
                Some(i.cbrt() as f64)
            }

            fn cubic_to_linear(_binding: &glib::Binding, i: f64) -> Option<f32> {
                Some((i * i * i) as f32)
            }

            self.parent_constructed();

            let item = self.row_data.borrow();
            let item = item.as_ref().cloned().unwrap();

            self.icon.set_icon_name(Some(&item.iconname()));

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
                .transform_to(linear_to_cubic)
                .transform_from(cubic_to_linear)
                .build();

            self.volume_scale.set_format_value_func(|_scale, value| {
                format!(
                    "{:>16}",
                    format!(
                        "{:.0}% ({:.2} dB)",
                        value * 100.0,
                        (value * value * value).log10() * 20.0
                    )
                )
            });

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
                .transform_to(linear_to_cubic)
                .transform_from(cubic_to_linear)
                .build();

            if matches!(
                item.nodetype(),
                /* NodeType::Input | */ NodeType::Output
            ) {
                //TODO: Implement support for Audio/Source switching for NodeType::Input
                let factory = gtk::SignalListItemFactory::new();
                factory.connect_setup(|_, item| {
                    let item: &gtk::ListItem = item.downcast_ref().expect("ListItem");
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
                    let item: &gtk::ListItem = item.downcast_ref().expect("ListItem");
                    let label = gtk::Label::new(None);
                    label.set_xalign(0.0);

                    item.property_expression("item")
                        .chain_property::<PwNodeObject>("name")
                        .bind(&label, "label", gtk::Widget::NONE);

                    item.set_child(Some(&label));
                });

                self.outputdevice_dropdown
                    .set_list_factory(Some(&listfactory));

                let app = PwvucontrolApplication::default();
                let model = &app.imp().nodemodel;

                let filter = gtk::CustomFilter::new(|x| {
                    if let Some(o) = x.downcast_ref::<PwNodeObject>() {
                        return o.nodetype() == crate::NodeType::Sink;
                    }
                    false
                });
                let filterlistmodel = &gtk::FilterListModel::new(Some(model.clone()), Some(filter));

                self.outputdevice_dropdown.set_enable_search(true);
                self.outputdevice_dropdown
                    .set_expression(Some(gtk::PropertyExpression::new(
                        PwNodeObject::static_type(),
                        gtk::Expression::NONE,
                        "name",
                    )));

                self.outputdevice_dropdown.set_model(Some(filterlistmodel));

                self.obj().update_output_device_dropdown();

                // filterlistmodel.connect_items_changed(clone!(@weak self as widget => move |_,_,_,_| {
                //     widget.obj().update_output_device_dropdown();
                // }));

                let app = PwvucontrolApplication::default();
                if let Some(metadata) = app.imp().metadata.borrow().as_ref() {
                    let boundid = item.boundid();
                    let sid = metadata.connect_changed(
                        clone!(@weak self as widget => @default-panic, move |_obj,id,key,_type_,value| {
                            if id == boundid && key == "target.object" {
                                wp::log::info!("metadata changed handler id:{boundid} value:{value}!");
                                widget.obj().update_output_device_dropdown();
                            }
                        })
                    );
                    self.metadata_changed_event.set(Some(sid));
                }

                self.outputdevice_dropdown.connect_notify_local(
                    Some("selected-item"),
                    clone!(@weak self as widget, @weak item as nodeobj => move |dropdown, _| {
                        wp::log::info!("selected-item");
                        if widget.outputdevice_dropdown_block_signal.get() {
                            return;
                        }
                        if let Some(item) = dropdown.selected_item() {
                            if let Some(item) = item.downcast_ref::<PwNodeObject>() {
                                nodeobj.set_default_target(item);
                            }
                        }
                    }),
                );
            } else {
                self.outputdevice_dropdown.set_visible(false);
            }
            let channelmodel = self.obj().channelmodel();

            self.channel_listbox.bind_model(
                Some(&channelmodel),
                clone!(@weak self as widget => @default-panic, move |item| {
                    PwChannelBox::new(
                        item.clone().downcast_ref::<PwChannelObject>()
                        .expect("RowData is of wrong type")
                    )
                    .upcast::<gtk::Widget>()
                }),
            );

            item.connect_local("format", false, 
            clone!(@weak channelmodel, @weak item as nodeobj => @default-panic, move |_| {
                let values = nodeobj.channel_volumes_vec();
                let oldlen = channelmodel.n_items();

                wp::log::debug!("format signal, values.len = {}, oldlen = {}", values.len(), oldlen);

                if values.len() as u32 != oldlen {
                    channelmodel.remove_all();
                    for (i,v) in values.iter().enumerate() {
                        channelmodel.append(&PwChannelObject::new(i as u32, *v, &nodeobj));
                    }

                    return None;
                }
                None
            }));

            item.connect_channel_volumes_notify(clone!(@weak channelmodel => move |nodeobj| {
                let values = nodeobj.channel_volumes_vec();
                for (i,v) in values.iter().enumerate() {
                    if let Some(item) = channelmodel.item(i as u32) {
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

            self.level_bar.set_min_value(0.0);
            self.level_bar.set_max_value(1.0);

            self.level_bar
                .add_offset_value(gtk::LEVEL_BAR_OFFSET_LOW, 0.0);
            self.level_bar
                .add_offset_value(gtk::LEVEL_BAR_OFFSET_HIGH, 0.0);
            self.level_bar
                .add_offset_value(gtk::LEVEL_BAR_OFFSET_FULL, 1.0);

            if let Ok(provider) = LevelbarProvider::new(&self.obj(), item.boundid()) {
                self.levelbarprovider
                    .set(provider)
                    .expect("Provider not set already");

                self.timeoutid.set(Some(glib::timeout_add_local(
                    std::time::Duration::from_millis(25),
                    clone!(@weak self as obj => @default-panic, move || {
                        obj.level_bar.set_value(obj.level.get() as f64);
                        ControlFlow::Continue
                    }),
                )));
            }
        }

        fn dispose(&self) {
            if let Some(sid) = self.metadata_changed_event.take() {
                let app = PwvucontrolApplication::default();
                if let Some(metadata) = app.imp().metadata.borrow().as_ref() {
                    metadata.disconnect(sid);
                };
            };
            if let Some(t) = self.timeoutid.take() {
                t.remove();
            }
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
            .property("row-data", row_data)
            .property(
                "channelmodel",
                gio::ListStore::new::<crate::pwchannelobject::PwChannelObject>(),
            )
            .build()
    }

    pub(crate) fn set_level(&self, level: f32) {
        self.imp().level.set(level);
    }

    pub(crate) fn update_output_device_dropdown(&self) {
        fn find_position_with_boundid_match(
            model: &impl IsA<gio::ListModel>,
            id: u32,
        ) -> Option<u32> {
            model.iter::<glib::Object>().enumerate().find_map(|(x, y)| {
                if let Ok(d) = y {
                    if let Some(o) = d.downcast_ref::<PwNodeObject>() {
                        dbg!(o.boundid());
                        if o.boundid() == id {
                            return Some(x as u32);
                        }
                    }
                }
                None
            })
        }

        let imp = self.imp();

        let item = imp.row_data.borrow();
        let item = item.as_ref().cloned().unwrap();
        let filterlistmodel = imp.outputdevice_dropdown.model().expect("model");

        if let Some(deftarget) = item.default_target() {
            if let Some(pos) =
                find_position_with_boundid_match(&filterlistmodel, deftarget.boundid())
            {
                wp::log::info!(
                    "switching to preferred target {} {}",
                    deftarget.boundid(),
                    deftarget.serial()
                );
                imp.outputdevice_dropdown_block_signal.set(true);
                imp.outputdevice_dropdown.set_selected(pos);
                imp.outputdevice_dropdown_block_signal.set(false);
            }
        } else {
            let app = PwvucontrolApplication::default();
            let core = app.imp().wp_core.get().expect("Core");
            let defaultnodesapi =
                wp::plugin::Plugin::find(core, "default-nodes-api").expect("Get mixer-api");
            let id: u32 = defaultnodesapi.emit_by_name("get-default-node", &[&"Audio/Sink"]);
            if id != u32::MAX {
                if let Some(pos) = find_position_with_boundid_match(&filterlistmodel, id) {
                    wp::log::info!("switching to default target");
                    imp.outputdevice_dropdown_block_signal.set(true);
                    imp.outputdevice_dropdown.set_selected(pos);
                    imp.outputdevice_dropdown_block_signal.set(false);
                }
            }
        }
    }
}
