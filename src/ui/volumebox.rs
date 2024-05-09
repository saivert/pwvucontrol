// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{backend::PwChannelObject, backend::PwNodeObject, backend::PwvucontrolManager, ui::LevelbarProvider, ui::PwChannelBox};

use glib::{clone, closure_local, ControlFlow, SignalHandlerId};
use gtk::{prelude::*, subclass::prelude::*};
use once_cell::sync::OnceCell;
use std::cell::{Cell, RefCell};
use wireplumber as wp;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/com/saivert/pwvucontrol/gtk/volumebox.ui")]
    #[properties(wrapper_type = super::PwVolumeBox)]
    pub struct PwVolumeBox {
        #[property(get, set, construct_only)]
        pub(super) node_object: RefCell<Option<PwNodeObject>>,

        metadata_changed_event: Cell<Option<SignalHandlerId>>,
        levelbarprovider: OnceCell<LevelbarProvider>,
        timeoutid: Cell<Option<glib::SourceId>>,
        pub(super) level: Cell<f32>,
        pub(super) default_node: Cell<u32>,
        pub(super) default_node_changed_handler: RefCell<Option<Box<dyn Fn()>>>,

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
        pub mainvolumescale: TemplateChild<gtk::Scale>,
        #[template_child]
        pub monitorvolumescale: TemplateChild<gtk::Scale>,
        #[template_child]
        pub container: TemplateChild<gtk::Box>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PwVolumeBox {
        const NAME: &'static str = "PwVolumeBox";
        type Type = super::PwVolumeBox;
        type ParentType = gtk::ListBoxRow;
        type Interfaces = (gtk::Buildable,);

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

            let item = self.node_object.borrow();
            let item = item.as_ref().cloned().unwrap();

            self.icon.set_icon_name(Some(&item.iconname()));

            item.bind_property("name", &self.title_label.get(), "label").sync_create().build();

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

            #[rustfmt::skip]
            item.bind_property("monitorvolume", &self.monitorvolumescale.adjustment(), "value")
                .sync_create()
                .bidirectional()
                .transform_to(linear_to_cubic)
                .transform_from(cubic_to_linear)
                .build();

            self.volume_scale.set_format_value_func(|_scale, value| {
                format!(
                    "{:>16}",
                    format!("{:.0}% ({:.2} dB)", value * 100.0, (value * value * value).log10() * 20.0)
                )
            });

            item.bind_property("formatstr", &self.format.get(), "label").sync_create().build();

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

            let manager = PwvucontrolManager::default();

            let defaultnodesapi = manager.default_nodes_api();
            let widget = self.obj();
            let defaultnodesapi_closure = closure_local!(@watch widget => move |defaultnodesapi: wp::plugin::Plugin| {
                let id: u32 = defaultnodesapi.emit_by_name("get-default-node", &[&"Audio/Sink"]);
                wp::info!("default-nodes-api changed: new id {id}");
                widget.imp().default_node.set(id);

                let handler = widget.imp().default_node_changed_handler.borrow();
                if let Some(cb) = handler.as_ref() {
                    cb();
                }
            });
            defaultnodesapi_closure.invoke::<()>(&[&defaultnodesapi]);
            defaultnodesapi.connect_closure("changed", false, defaultnodesapi_closure);

            self.channel_listbox.bind_model(
                Some(&item.channelmodel()),
                clone!(@weak self as widget => @default-panic, move |item| {
                    PwChannelBox::new(
                        item.clone().downcast_ref::<PwChannelObject>()
                        .expect("RowData is of wrong type")
                    )
                    .upcast::<gtk::Widget>()
                }),
            );

            self.revealer.connect_child_revealed_notify(clone!(@weak self as widget => move |_| {
                widget.obj().grab_focus();
            }));

            self.level_bar.set_min_value(0.0);
            self.level_bar.set_max_value(1.0);

            self.level_bar.add_offset_value(gtk::LEVEL_BAR_OFFSET_LOW, 0.0);
            self.level_bar.add_offset_value(gtk::LEVEL_BAR_OFFSET_HIGH, 0.0);
            self.level_bar.add_offset_value(gtk::LEVEL_BAR_OFFSET_FULL, 1.0);

            if let Ok(provider) = LevelbarProvider::new(&self.obj(), item.boundid()) {
                self.levelbarprovider.set(provider).expect("Provider not set already");

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
                let manager = PwvucontrolManager::default();
                if let Some(metadata) = manager.imp().metadata.borrow().as_ref() {
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

    impl BuildableImpl for PwVolumeBox {
        fn add_child(&self, builder: &gtk::Builder, child: &glib::Object, type_: Option<&str>) {
            if type_.unwrap_or_default() == "extra" {
                if let Some(widget) = child.downcast_ref::<gtk::Widget>() {
                    if let Some(container) = self.container.try_get() {
                        widget.unparent();
                        container.append(widget);
                        container.set_child_visible(true);
                    }
                }
            } else {
                self.parent_add_child(builder, child, type_);
            }
        }
    }

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
        @implements gtk::Actionable, gtk::Buildable;
}

impl PwVolumeBox {
    pub(crate) fn new(channel_object: &impl glib::IsA<PwNodeObject>) -> Self {
        glib::Object::builder()
            .property("node-object", channel_object)
            //.property("channelmodel", gio::ListStore::new::<PwChannelObject>())
            .build()
    }

    pub(crate) fn set_level(&self, level: f32) {
        self.imp().level.set(level);
    }

    pub fn set_default_node_change_handler(&self, c: impl Fn() + 'static) {
        let imp = self.imp();

        let mut handler = imp.default_node_changed_handler.borrow_mut();
        handler.replace(Box::new(c));
    }
}
pub trait PwVolumeBoxImpl: ListBoxRowImpl {}

pub trait PwVolumeBoxExt: IsA<PwVolumeBox> {
    fn default_node(&self) -> u32 {
        self.upcast_ref::<PwVolumeBox>().imp().default_node.get()
    }

    fn node_object(&self) -> Option<PwNodeObject> {
        self.upcast_ref::<PwVolumeBox>().node_object()
    }

    fn set_default_node_change_handler(&self, c: impl Fn() + 'static) {
        self.upcast_ref::<PwVolumeBox>().set_default_node_change_handler(c);
    }
}

impl<O: IsA<PwVolumeBox>> PwVolumeBoxExt for O {}

unsafe impl<T: PwVolumeBoxImpl> IsSubclassable<T> for PwVolumeBox {
    fn class_init(class: &mut glib::Class<Self>) {
        Self::parent_class_init::<T>(class.upcast_ref_mut());
    }
}
