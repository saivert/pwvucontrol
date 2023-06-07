use gtk::subclass::prelude::*;

use crate::pwnodeobject::PwNodeObject;
use gtk::{gio, glib, prelude::*};

mod imp {
    use gtk::{gio::{subclass::prelude::*, self}, glib, prelude::*};
    
    use std::cell::RefCell;
    
    // Use `im-rc::Vector` here as it has much better insert/delete performance than a plain `Vec`.
    use im_rc::Vector;

    use crate::pwnodeobject::PwNodeObject;

    
    #[derive(Debug, Default)]
    pub struct PwNodeModel(pub(super) RefCell<Vector<PwNodeObject>>);
    
    /// Basic declaration of our type for the GObject type system
    #[glib::object_subclass]
    impl ObjectSubclass for PwNodeModel {
        const NAME: &'static str = "Model";
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

// Public part of the Model type.
glib::wrapper! {
    pub struct PwNodeModel(ObjectSubclass<imp::PwNodeModel>) @implements gio::ListModel;
}

// Constructor for new instances. This simply calls glib::Object::new()
impl PwNodeModel {
    pub fn new() -> PwNodeModel {
        glib::Object::new()
    }

    pub fn append(&self, obj: &PwNodeObject) {
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

    pub fn remove(&self, id: u32) {
        let imp = self.imp();
        let mut vector = imp.0.borrow_mut();
        for (i,v) in (0..).zip(vector.iter()) {
            if id == v.serial() {
                vector.remove(i as usize);
                // Emits a signal that 1 item was removed, 0 added at the position index
                self.items_changed(i, 1, 0);
                break;
            }
        }
        
    }

    pub fn update_node<F>(&self, id: u32, f: F) -> Result<(), ()>
    where F: FnOnce(&PwNodeObject) {
        let imp = self.imp();
        let vector = imp.0.borrow();
        if let Some(v) = vector.iter().find(|p|id == p.serial()) {
            f(v);
            return Ok(());
        }
        Err(())
        
    }

    pub fn get_node(&self, id: u32) -> Result<PwNodeObject, ()> {
        let imp = self.imp();
        let vector = imp.0.borrow();
        if let Some(v) = vector.iter().find(|p|id == p.serial()) {
            return Ok(v.clone());
        }
        Err(())
    }
}

impl Default for PwNodeModel {
    fn default() -> Self {
        Self::new()
    }
}