// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{
    backend::{NodeType, PwNodeObject, PwvucontrolManager},
    pwvucontrol_info,
    ui::{PwRouteDropDown, PwVolumeBox, PwvucontrolWindow},
};

use crate::pwvucontrol_warning;
use glib::clone;
use glib::closure_local;
use gtk::{prelude::*, subclass::prelude::*};
use std::cell::Cell;
use std::cell::RefCell;
use wireplumber as wp;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/com/saivert/pwvucontrol/gtk/sinkbox.ui")]
    #[properties(wrapper_type = super::PwSinkBox)]
    pub struct PwSinkBox {
        pub(super) block_default_node_toggle_signal: Cell<bool>,

        #[property(get, set, construct_only)]
        pub(super) node_object: RefCell<Option<PwNodeObject>>,

        #[template_child]
        pub volumebox: TemplateChild<PwVolumeBox>,

        #[template_child]
        pub default_sink_toggle: TemplateChild<gtk::ToggleButton>,

        #[template_child]
        pub route_dropdown: TemplateChild<PwRouteDropDown>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PwSinkBox {
        const NAME: &'static str = "PwSinkBox";
        type Type = super::PwSinkBox;
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
    impl ObjectImpl for PwSinkBox {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            let item = obj.node_object().expect("nodeobj");

            self.volumebox.set_node_object(&item);

            if matches!(item.nodetype(), NodeType::Sink) {
                self.volumebox.imp().volume_scale.connect_volume_notify(|_| {
                    let window = PwvucontrolWindow::default();
                    window.play_beep();
                });
            }
            let manager = PwvucontrolManager::default();
            let defaultnodesapi = manager.default_nodes_api();
            let widget = self.obj();
            let defaultnodesapi_closure = closure_local!(@watch widget => move |_: wp::plugin::Plugin| widget.imp().default_node_changed());
            defaultnodesapi.connect_closure("changed", false, defaultnodesapi_closure);
            self.default_node_changed();

            // Only set nodeobject once it has a device associated.
            if let Some(node) = obj.node_object() {
                node.connect_device_notify(clone!(@weak self as widget => move |nodeobject| {
                    widget.route_dropdown.set_nodeobject(Some(nodeobject));
                    widget.default_node_changed();
                }));
            }

            pwvucontrol_info!("sinkbox set_nodeobject {}", self.obj().node_object().expect("Node object").name());
        }
    }
    impl WidgetImpl for PwSinkBox {}
    impl ListBoxRowImpl for PwSinkBox {}

    #[gtk::template_callbacks]
    impl PwSinkBox {
        #[template_callback]
        fn default_sink_toggle_toggled(&self, _togglebutton: &gtk::ToggleButton) {
            if self.block_default_node_toggle_signal.get() {
                return;
            }

            let node = self.volumebox.node_object().expect("nodeobj");
            let node_name: String = if _togglebutton.is_active() { node.node_property("node.name").unwrap_or_default() } else { "".to_string() };

            let manager = PwvucontrolManager::default();

            let core = manager.imp().wp_core.get().expect("Core");
            let defaultnodesapi = wp::plugin::Plugin::find(core, "default-nodes-api").expect("Get mixer-api");

            let type_name = match node.nodetype() {
                NodeType::Sink => "Audio/Sink",
                NodeType::Source => match node.is_virtual() {
                    true => "Audio/Source/Virtual",
                    false => "Audio/Source",
                },
                _ => unreachable!(),
            };

            let result: bool = defaultnodesapi.emit_by_name("set-default-configured-node-name", &[&type_name, &node_name]);
            wp::info!("set-default-configured-node-name result: {result:?}");
        }

        fn default_node_changed(&self) {
            let manager = PwvucontrolManager::default();
            let defaultnodesapi = manager.default_nodes_api();

            let node = self.obj().node_object().expect("nodeobj");
            let Some(media_class) = node.node_property::<String>("media.class") else {
                pwvucontrol_warning!("{} is missing media.class property", node.name());
                return;
            };
            let id: u32 = defaultnodesapi.emit_by_name("get-default-node", &[&media_class]);

            self.block_default_node_toggle_signal.set(true);
            self.default_sink_toggle.set_active(node.boundid() == id);
            self.block_default_node_toggle_signal.set(false);
        }
    }
}

glib::wrapper! {
    pub struct PwSinkBox(ObjectSubclass<imp::PwSinkBox>)
        @extends gtk::Widget, gtk::ListBoxRow,
        @implements gtk::Actionable;
}

impl PwSinkBox {
    pub(crate) fn new(node_object: &impl IsA<PwNodeObject>) -> Self {
        glib::Object::builder().property("node-object", node_object).build()
    }
}
