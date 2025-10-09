// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{
    backend::{PwNodeObject, PwvucontrolManager},
    macros::*,
    ui::{PwStreamDropDown, PwVolumeBox},
};
use glib::{clone, closure_local};
use gtk::{prelude::*, subclass::prelude::*};
use std::cell::OnceCell;
use wireplumber as wp;

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/com/saivert/pwvucontrol/gtk/streambox.ui")]
    #[properties(wrapper_type = super::PwStreamBox)]
    pub struct PwStreamBox {
        #[property(get, set, construct_only)]
        pub(super) node_object: OnceCell<PwNodeObject>,

        #[template_child]
        pub volumebox: TemplateChild<PwVolumeBox>,

        #[template_child]
        pub output_dropdown: TemplateChild<PwStreamDropDown>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PwStreamBox {
        const NAME: &'static str = "PwStreamBox";
        type Type = super::PwStreamBox;
        type ParentType = gtk::ListBoxRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for PwStreamBox {
        fn constructed(&self) {
            let manager = PwvucontrolManager::default();

            let obj = self.obj();
            let item = obj.node_object();

            self.volumebox.set_node_object(&item);

            let defaultnodesapi = manager.default_nodes_api();
            let widget = self.obj();
            let defaultnodesapi_closure = closure_local!(@watch widget => move |_: wp::plugin::Plugin| widget.update_output_device_dropdown());
            defaultnodesapi.connect_closure("changed", false, defaultnodesapi_closure);
            widget.update_output_device_dropdown();

            self.parent_constructed();

            if let Some(metadata) = manager.metadata() {
                let boundid = item.boundid();
                let widget = self.obj();
                let changed_closure = closure_local!(@watch widget =>
                    move |_obj: &wp::pw::Metadata, id: u32, key: Option<String>, _type: Option<String>, _value: Option<String>| {
                    let key = key.unwrap_or_default();
                    if id == boundid && key.contains("target.") {
                        pwvucontrol_info!("metadata changed handler id: {boundid} {key:?} {_value:?}!");
                        widget.update_output_device_dropdown();
                    }
                });
                metadata.connect_closure("changed", false, changed_closure);
            }

            // Create our custom output dropdown widget and add it to the layout
            self.output_dropdown.set_nodeobj(Some(&item));

            glib::idle_add_local_once(clone!(@weak self as widget => move || {
                widget.obj().update_output_device_dropdown();
            }));
        }
    }
    impl WidgetImpl for PwStreamBox {}
    impl ListBoxRowImpl for PwStreamBox {}

    impl PwStreamBox {}
}

glib::wrapper! {
    pub struct PwStreamBox(ObjectSubclass<imp::PwStreamBox>)
        @extends gtk::Widget, gtk::ListBoxRow,
        @implements gtk::Actionable;
}

impl PwStreamBox {
    pub(crate) fn new(node_object: &impl IsA<PwNodeObject>) -> Self {
        glib::Object::builder().property("node-object", node_object).build()
    }

    fn update_output_device_dropdown(&self) {
        let manager = PwvucontrolManager::default();

        let item = self.node_object();

        let stream_model = match item.nodetype() {
            crate::backend::NodeType::StreamInput => manager.source_model(),
            crate::backend::NodeType::StreamOutput => manager.sink_model(),
            _ => panic!("Invalid node type"),
        };

        let imp = self.imp();

        let output_dropdown = imp.output_dropdown.get();

        let default_node = match item.nodetype() {
            crate::backend::NodeType::StreamInput => manager.default_configured_source_node(),
            crate::backend::NodeType::StreamOutput => manager.default_configured_sink_node(),
            _ => panic!("Invalid node type"),
        };

        // The following is just so this string gets picked up by xgettext, since it doesn't handle rust macros yet.
        #[cfg(debug_assertions)]
        gettextrs::gettext("Default ({})");

        let string = if let Some(node) = default_node {
            formatx::formatx!(gettextrs::gettext("Default ({})"), node.name()).unwrap()
        } else {
            gettextrs::gettext("Default")
        };
        output_dropdown.set_default_text(&string);

        if let Some(deftarget) = item.default_target() {
            if let Some(pos) = stream_model.get_node_pos_from_id(deftarget.boundid()) {
                pwvucontrol_info!("switching to preferred target pos={pos} boundid={} serial={}", deftarget.boundid(), deftarget.serial());
                output_dropdown.set_selected_no_send(pos + 1);
            }
        } else {
            output_dropdown.set_selected_no_send(0);
        }
    }
}
