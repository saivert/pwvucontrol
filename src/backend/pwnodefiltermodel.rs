// SPDX-License-Identifier: GPL-3.0-or-later

use glib::{closure_local, subclass::prelude::*, Properties, SignalHandlerId};
use gtk::{gio, prelude::*, subclass::prelude::*};
use std::cell::{Cell, RefCell, OnceCell};
use super::{NodeType, PwNodeObject};

mod imp {
    use super::*;

    #[derive(Debug, Properties, Default)]
    #[properties(wrapper_type = super::PwNodeFilterModel)]
    pub struct PwNodeFilterModel {
        /// Contains the items that matches the filter predicate.
        pub(super) filtered_model: OnceCell<gtk::FilterListModel>,

        #[property(get, set, construct_only, builder(NodeType::Undefined))]
        pub(super) nodetype: Cell<NodeType>,

        /// The model we are filtering.
        #[property(get, set = Self::set_model, nullable)]
        pub(super) model: RefCell<Option<gio::ListModel>>,

        pub(crate) signalid: RefCell<Option<SignalHandlerId>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PwNodeFilterModel {
        const NAME: &'static str = "PwNodeFilterModel";
        type Type = super::PwNodeFilterModel;
        type Interfaces = (gio::ListModel,);
    }

    #[glib::derived_properties]
    impl ObjectImpl for PwNodeFilterModel {
        fn constructed(&self) {
            self.parent_constructed();

            let nodetype = self.nodetype.get();

            let filter = gtk::CustomFilter::new(move |obj| {
                let node: &PwNodeObject = obj.downcast_ref().expect("PwNodeObject");
                node.nodetype() == nodetype && !node.hidden()
            });

            self.filtered_model.set(gtk::FilterListModel::new(None::<gio::ListModel>, Some(filter))).expect("filtered model not set");

        }
    }

    impl ListModelImpl for PwNodeFilterModel {
        fn item_type(&self) -> glib::Type {
            PwNodeObject::static_type()
        }
        fn n_items(&self) -> u32 {
            self.filtered_model.get().expect("Filtered model").n_items()
        }
        fn item(&self, position: u32) -> Option<glib::Object> {
            self.filtered_model.get().expect("Filtered model").item(position)
        }
    }

    impl PwNodeFilterModel {
        fn set_model(&self, new_model: Option<gio::ListModel>) {
            let filtered_model = self.filtered_model.get().expect("Filtered model");
            let removed = filtered_model.n_items();
            let widget = self.obj();

            self.disconnect();

            if let Some(new_model) = new_model {

                assert!(self.item_type().is_a(new_model.item_type()));

                let handler = closure_local!(@watch widget => move |_listmodel: &gio::ListModel, position: u32, removed: u32, added: u32| {
                    widget.items_changed(position, removed, added);
                });
                //handler.invoke::<()>(&[&new_model, &0u32, &0u32, &0u32]);
                self.signalid.replace(Some(filtered_model.connect_closure("items-changed", true, handler)));

                filtered_model.set_model(Some(&new_model));

                self.model.replace(Some(new_model));
            } else {
                widget.items_changed(0, removed, 0);
            }
        }

        fn disconnect(&self) {
            let filtered_model = self.filtered_model.get().expect("Filtered model");
            filtered_model.set_model(gio::ListModel::NONE);
            if let Some(id) = self.signalid.take() {
                filtered_model.disconnect(id);
            }
        }
    }
}

glib::wrapper! {
    pub struct PwNodeFilterModel(ObjectSubclass<imp::PwNodeFilterModel>) @implements gio::ListModel;
}

impl PwNodeFilterModel {
    pub(crate) fn new(nodetype: NodeType, model: Option<impl glib::IsA<gio::ListModel>>) -> Self
    {
        glib::Object::builder()
        .property("model", &model)
        .property("nodetype", nodetype)
        .build()
    }

    pub fn get_node_pos_from_id(&self, id: u32) -> Option<u32> {
        let pos: Option<usize> = self.iter::<PwNodeObject>().position(|item| {
            item.map_or(false, |item| item.boundid() == id)
        });
        pos.map(|x| x as u32)
    }
}
