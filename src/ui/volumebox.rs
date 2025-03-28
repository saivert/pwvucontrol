// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{
    backend::{NodeType, PwChannelObject, PwNodeObject, PwvucontrolManager},
    ui::{LevelbarProvider, PwChannelBox, PwPeakMeter, PwVolumeScale},
};
use glib::{clone, ControlFlow, SignalHandlerId};
use gtk::{prelude::*, subclass::prelude::*};
use std::cell::{Cell, RefCell};

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/com/saivert/pwvucontrol/gtk/volumebox.ui")]
    #[properties(wrapper_type = super::PwVolumeBox)]
    pub struct PwVolumeBox {
        #[property(get, set = Self::set_node_object)]
        pub(super) node_object: RefCell<Option<PwNodeObject>>,

        metadata_changed_event: Cell<Option<SignalHandlerId>>,
        levelbarprovider: Cell<Option<LevelbarProvider>>,
        timeoutid: Cell<Option<gtk::TickCallbackId>>,
        pub(super) level: Cell<f32>,

        // Template widgets
        #[template_child]
        pub icon: TemplateChild<gtk::Image>,
        #[template_child]
        pub title_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub subtitle_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub volume_scale: TemplateChild<PwVolumeScale>,
        #[template_child]
        pub peak_meter: TemplateChild<PwPeakMeter>,
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
        pub mainvolumescale: TemplateChild<PwVolumeScale>,
        #[template_child]
        pub monitorvolumescale: TemplateChild<PwVolumeScale>,
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
            PwPeakMeter::ensure_type();
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for PwVolumeBox {
        fn constructed(&self) {
            self.parent_constructed();

            self.revealer.connect_child_revealed_notify(clone!(@weak self as widget => move |_| {
                widget.obj().grab_focus();
            }));

            let window = crate::ui::PwvucontrolWindow::default();
            window.imp().settings.bind("use-peakmeter-led", &self.peak_meter.get(), "use-led").get_only().build();
        }

        fn dispose(&self) {
            if let Some(sid) = self.metadata_changed_event.take() {
                let manager = PwvucontrolManager::default();
                if let Some(metadata) = manager.metadata() {
                    metadata.disconnect(sid);
                };
            };
        }
    }

    impl WidgetImpl for PwVolumeBox {
        fn unmap(&self) {
            if let Some(t) = self.timeoutid.take() {
                t.remove();
            }
            self.levelbarprovider.take();
            self.parent_unmap();
        }

        fn map(&self) {
            self.parent_map();

            // Monitoring ourselves cause an infinite loop.
            let item = self.node_object.borrow();
            let item = item.as_ref().unwrap();
            if item.name() != "pwvucontrol-peak-detect" {
                self.setuplevelbar();
            } else {
                self.peak_meter.set_visible(false);
            }
        }
    }
    impl ListBoxRowImpl for PwVolumeBox {}

    impl BuildableImpl for PwVolumeBox {
        fn add_child(&self, builder: &gtk::Builder, child: &glib::Object, type_: Option<&str>) {
            if type_ == Some("extra") {
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

    impl PwVolumeBox {
        fn set_node_object(&self, node_object: PwNodeObject) {
            self.node_object.set(Some(node_object));
            let item = self.node_object.borrow();
            let item = item.as_ref().unwrap();

            // Flatpak blocks access to application icons, so hide the icon when run in the sandbox.
            if cfg!(feature = "sandboxed") && matches!(item.nodetype(), NodeType::StreamInput | NodeType::StreamOutput) {
                self.icon.set_visible(false);
            } else {
                self.icon.set_icon_name(Some(&item.iconname()));
            }

            item.bind_property("name", &self.title_label.get(), "label").sync_create().build();

            item.bind_property("description", &self.subtitle_label.get(), "label").sync_create().build();

            item.bind_property("mute", &self.mutebtn.get(), "active").sync_create().bidirectional().build();

            item.bind_property("volume", &self.volume_scale.get(), "volume").sync_create().bidirectional().build();

            item.bind_property("monitorvolume", &self.monitorvolumescale.get(), "volume").sync_create().bidirectional().build();

            item.bind_property("formatstr", &self.format.get(), "label").sync_create().build();

            item.bind_property("channellock", &self.channellock.get(), "active").sync_create().bidirectional().build();

            item.bind_property("mainvolume", &self.mainvolumescale.get(), "volume").sync_create().bidirectional().build();

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
        }

        fn setuplevelbar(&self) {
            let item = self.node_object.borrow();
            let item = item.as_ref().unwrap();

            if let Ok(provider) = LevelbarProvider::new(&self.obj(), item.boundid()) {
                self.levelbarprovider.set(Some(provider));

                let callbackid = self.obj().add_tick_callback(|widget, _fc| {
                    widget.imp().peak_meter.set_level(widget.imp().level.get());
                    ControlFlow::Continue
                });
                self.timeoutid.set(Some(callbackid));
            }
        }
    }
}

glib::wrapper! {
    pub struct PwVolumeBox(ObjectSubclass<imp::PwVolumeBox>)
        @extends gtk::Widget,
        @implements gtk::Actionable, gtk::Buildable;
}

impl PwVolumeBox {
    pub(crate) fn set_level(&self, level: f32) {
        self.imp().level.set(level);
    }
}
