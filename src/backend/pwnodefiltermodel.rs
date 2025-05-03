// SPDX-License-Identifier: GPL-3.0-or-later

use super::{NodeType, PwNodeObject};
use glib::{closure_local, Properties, SignalHandlerId};
use gtk::{gio, prelude::*, subclass::prelude::*};
use std::cell::{Cell, OnceCell, RefCell};

mod imp {
    use super::*;

    #[derive(Debug, Properties, Default)]
    #[properties(wrapper_type = super::PwNodeFilterModel)]
    pub struct PwNodeFilterModel {
        /// Contains the items that matches the filter predicate.
        pub(super) filtered_model: gtk::FilterListModel,

        #[property(get, set = Self::set_nodetype, builder(NodeType::Undefined))]
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
    impl ObjectImpl for PwNodeFilterModel {}

    impl ListModelImpl for PwNodeFilterModel {
        fn item_type(&self) -> glib::Type {
            PwNodeObject::static_type()
        }
        fn n_items(&self) -> u32 {
            self.filtered_model.n_items()
        }
        fn item(&self, position: u32) -> Option<glib::Object> {
            self.filtered_model.item(position)
        }
    }

    impl PwNodeFilterModel {

        fn set_nodetype(&self, nodetype: NodeType) {
            self.nodetype.set(nodetype);

            let filter = gtk::CustomFilter::new(move |obj| {
                let node: &PwNodeObject = obj.downcast_ref().expect("PwNodeObject");
                if nodetype == NodeType::Undefined {
                    return !node.hidden();
                }
                node.nodetype() == nodetype && !node.hidden()
            });

            self.filtered_model.set_filter(Some(&filter));
        }

        fn set_model(&self, new_model: Option<gio::ListModel>) {
            let removed = self.filtered_model.n_items();
            let widget = self.obj();

            self.disconnect();

            if let Some(new_model) = new_model {
                assert!(self.item_type().is_a(new_model.item_type()));

                let handler = closure_local!(@watch widget => move |_listmodel: &gio::ListModel, position: u32, removed: u32, added: u32| {
                    widget.items_changed(position, removed, added);
                });
                //handler.invoke::<()>(&[&new_model, &0u32, &0u32, &0u32]);
                self.signalid.replace(Some(self.filtered_model.connect_closure("items-changed", true, handler)));

                self.filtered_model.set_model(Some(&new_model));

                self.model.replace(Some(new_model));
            } else {
                widget.items_changed(0, removed, 0);
            }
        }

        fn disconnect(&self) {
            self.filtered_model.set_model(gio::ListModel::NONE);
            if let Some(id) = self.signalid.take() {
                self.filtered_model.disconnect(id);
            }
        }
    }
}

glib::wrapper! {
    pub struct PwNodeFilterModel(ObjectSubclass<imp::PwNodeFilterModel>) @implements gio::ListModel;
}

impl PwNodeFilterModel {
    pub(crate) fn new(nodetype: NodeType, model: Option<impl IsA<gio::ListModel>>) -> Self {
        glib::Object::builder().property("model", &model).property("nodetype", nodetype).build()
    }

    pub fn get_node_pos_from_id(&self, id: u32) -> Option<u32> {
        let pos: Option<usize> = self.iter::<PwNodeObject>().position(|item| item.map_or(false, |item| item.boundid() == id));
        pos.map(|x| x as u32)
    }
}

impl Default for PwNodeFilterModel {
    fn default() -> Self {
        Self::new(NodeType::Undefined, None::<gio::ListModel>)
    }
}
