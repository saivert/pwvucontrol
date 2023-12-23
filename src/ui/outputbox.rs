// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{
    backend::PwvucontrolManager,
    backend::PwNodeObject,
    ui::PwVolumeBox,
    ui::PwVolumeBoxImpl,
    ui::PwOutputDropDown,
};
use glib::{closure_local, clone};
use gtk::{prelude::*, subclass::prelude::*};
use wireplumber as wp;

mod imp {
    use crate::backend::PwvucontrolManager;

    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/saivert/pwvucontrol/gtk/outputbox.ui")]
    pub struct PwOutputBox {
        #[template_child]
        pub output_dropdown: TemplateChild<PwOutputDropDown>,
    }
    
    #[glib::object_subclass]
    impl ObjectSubclass for PwOutputBox {
        const NAME: &'static str = "PwOutputBox";
        type Type = super::PwOutputBox;
        type ParentType = PwVolumeBox;
    
        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }
    
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    
    impl ObjectImpl for PwOutputBox {
        fn constructed(&self) {

            let manager = PwvucontrolManager::default();

            let obj = self.obj();
            let parent: &PwVolumeBox = obj.upcast_ref();
            let item = parent.node_object().expect("nodeobj");

            parent.add_default_node_change_handler(clone!(@weak self as widget => move || {
                widget.obj().update_output_device_dropdown();
            }));

            self.parent_constructed();


            if let Some(metadata) = manager.imp().metadata.borrow().as_ref() {
                let boundid = item.boundid();
                let widget = self.obj();
                let changed_closure = closure_local!(@watch widget =>
                    move |_obj: &wp::pw::Metadata, id: u32, key: Option<String>, _type: Option<String>, _value: Option<String>| {
                    let key = key.unwrap_or_default();
                    if id == boundid && key.contains("target.") {
                        wp::log::info!("metadata changed handler id: {boundid} {key:?} {_value:?}!");
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
    impl WidgetImpl for PwOutputBox {}
    impl ListBoxRowImpl for PwOutputBox {}
    impl PwVolumeBoxImpl for PwOutputBox {}
    
    impl PwOutputBox {

    }
}

glib::wrapper! {
    pub struct PwOutputBox(ObjectSubclass<imp::PwOutputBox>)
        @extends gtk::Widget, gtk::ListBoxRow, PwVolumeBox,
        @implements gtk::Actionable;
}

impl PwOutputBox {
    pub(crate) fn new(node_object: &impl glib::IsA<PwNodeObject>) -> Self {
        glib::Object::builder()
        .property("node-object", node_object)
        .build()
    }

    pub(crate) fn update_output_device_dropdown(&self) {
        let manager = PwvucontrolManager::default();

        let sinkmodel = &manager.imp().sinkmodel;

        let imp = self.imp();
        let parent: &PwVolumeBox = self.upcast_ref();

        let output_dropdown = imp.output_dropdown.get();

        let id = parent.imp().default_node.get();

        let string = if let Ok(node) = sinkmodel.get_node(id) {
            format!("Default ({})", node.name().unwrap())
        } else {
            "Default".to_string()
        };
        output_dropdown.set_default_text(&string);

        let item = parent.node_object().expect("nodeobj");

        if let Some(deftarget) = item.default_target() {
            // let model: gio::ListModel = imp
            //     .outputdevice_dropdown
            //     .model()
            //     .expect("Model from dropdown")
            //     .downcast()
            //     .unwrap();
            // let pos = model.iter::<glib::Object>().enumerate().find_map(|o| {
            //     if let Ok(Ok(node)) = o.1.map(|x| x.downcast::<PwNodeObject>()) {
            //         if node.boundid() == deftarget.boundid() {
            //             return Some(o.0);
            //         }
            //     }
            //     None
            // });

            if let Some(pos) = sinkmodel.get_node_pos_from_id(deftarget.boundid()) {
                wp::log::info!(
                    "switching to preferred target pos={pos} boundid={} serial={}",
                    deftarget.boundid(),
                    deftarget.serial()
                );
                output_dropdown.set_selected_no_send(pos+1);
            }
        } else {
            output_dropdown.set_selected_no_send(0);

            // let id = self.imp().default_node.get();
            // wp::log::info!("default_node is {id}");
            // if id != u32::MAX {
            //     if let Some(pos) = sinkmodel.get_node_pos_from_id(id) {
            //         wp::log::info!("switching to default target");
            //         if true
            //         /* imp.outputdevice_dropdown.selected() != pos */
            //         {
            //             wp::log::info!("actually switching to default target");
            //             imp.outputdevice_dropdown_block_signal.set(true);
            //             imp.outputdevice_dropdown.set_selected(pos);
            //             imp.outputdevice_dropdown_block_signal.set(false);
            //         }
            //     }
            // }
        }
    }

}
