// SPDX-License-Identifier: GPL-3.0-or-later

use crate::backend::pwnodeobject::PwNodeObject;
use gtk::{gio, glib, prelude::*, subclass::prelude::*};
use std::cell::RefCell;
use im_rc::Vector;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct PwNodeModel(pub(super) RefCell<Vector<PwNodeObject>>);
    
    #[glib::object_subclass]
    impl ObjectSubclass for PwNodeModel {
        const NAME: &'static str = "PwNodeModel";
        type Type = super::PwNodeModel;
        type Interfaces = (gio::ListModel,);
    }
    
    impl ObjectImpl for PwNodeModel {}
    
    impl ListModelImpl for PwNodeModel {
        fn item_type(&self) -> glib::Type {
            PwNodeObject::static_type()
        }
        fn n_items(&self) -> u32 {
            self.0.borrow().len() as u32
        }
        fn item(&self, position: u32) -> Option<glib::Object> {
            self.0
                .borrow()
                .get(position as usize)
                .map(|o| o.clone().upcast::<glib::Object>())
        }
    }
}

glib::wrapper! {
    pub struct PwNodeModel(ObjectSubclass<imp::PwNodeModel>) @implements gio::ListModel;
}

impl PwNodeModel {
    pub(crate) fn new() -> PwNodeModel {
        glib::Object::new()
    }

    pub(crate) fn append(&self, obj: &PwNodeObject) {
        let imp = self.imp();
        let index = {
            // Borrow the data only once and ensure the borrow guard is dropped
            // before we emit the items_changed signal because the view
            // could call get_item / get_n_item from the signal handler to update its state
            let mut data = imp.0.borrow_mut();
            data.push_back(obj.clone());
            data.len() - 1
        };
        // Emits a signal that 1 item was added, 0 removed at the position index
        self.items_changed(index as u32, 0, 1);
    }

    pub(crate) fn remove(&self, id: u32) {
        let imp = self.imp();
        let mut vector = imp.0.borrow_mut();
        for (i,v) in (0..).zip(vector.iter()) {
            if id == v.boundid() {
                vector.remove(i as usize);
                drop(vector);
                // Emits a signal that 1 item was removed, 0 added at the position index
                self.items_changed(i, 1, 0);
                break;
            }
        }
        
    }

    pub fn get_node(&self, id: u32) -> Result<PwNodeObject, ()> {
        let imp = self.imp();
        let vector = imp.0.borrow();
        if let Some(v) = vector.iter().find(|p|id == p.boundid()) {
            return Ok(v.clone());
        }
        Err(())
    }

    pub fn get_node_pos_from_id(&self, id: u32) -> Option<u32> {
        let imp = self.imp();
        let vector = imp.0.borrow();
        for (i, v) in vector.iter().enumerate() {
            if v.boundid() == id {
                return Some(i as u32);
            }
        }
        None
    }

}

impl Default for PwNodeModel {
    fn default() -> Self {
        Self::new()
    }
}