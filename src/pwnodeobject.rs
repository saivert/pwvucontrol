use glib::Object;
use gtk::glib;


mod imp {
    use std::cell::{Cell, RefCell};
    use gtk::subclass::prelude::*;

    use gtk::{
        glib::{self, ParamSpec, Properties, Value},
        prelude::*,
    };
    
    // Object holding the state
    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::PwNodeObject)]
    pub struct PwNodeObject {
        #[property(get, set)]
        name: RefCell<Option<String>>,
        #[property(get, set)]
        serial: Cell<u32>,
        #[property(get, set)]
        volume: Cell<f32>,
        #[property(get, set)]
        mute: Cell<bool>,
        #[property(get = Self::channel_volumes, set = Self::set_channel_volumes, type = glib::ValueArray)]
        channel_volumes: RefCell<Vec<f32>>,
    }
    
    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for PwNodeObject {
        const NAME: &'static str = "PwNodeObject";
        type Type = super::PwNodeObject;
    }

    // Trait shared by all GObjects
    impl ObjectImpl for PwNodeObject {
        fn properties() -> &'static [ParamSpec] {
            Self::derived_properties()
        }
    
        fn set_property(&self, id: usize, value: &Value, pspec: &ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }
    
        fn property(&self, id: usize, pspec: &ParamSpec) -> Value {
            self.derived_property(id, pspec)
        }

    }

    impl PwNodeObject {
        pub fn channel_volumes(&self) -> glib::ValueArray {
            let mut values = glib::ValueArray::new(self.channel_volumes.borrow().len() as u32);
            let channel_volumes = self.channel_volumes.borrow();
            channel_volumes.iter().for_each(|volume| {
                values.append(&Value::from(volume));

            });

            values
        }
    
        pub fn set_channel_volumes(&self, values: glib::ValueArray) {
            let mut channel_volumes = self.channel_volumes.borrow_mut();
            values.iter().for_each(|value| {
                if let Ok(volume) = value.get() {
                    channel_volumes.push(volume);
                }
            });
        }

        pub fn set_channel_volumes_vec(&self, values: &Vec<f32>) {
            *(self.channel_volumes.borrow_mut()) = values.clone();
           
        }

    }
}

glib::wrapper! {
    pub struct PwNodeObject(ObjectSubclass<imp::PwNodeObject>);
}

impl PwNodeObject {
    pub fn new(serial: u32, name: &str) -> Self {
        Object::builder()
            .property("serial", serial)
            .property("name", name)
            .build()
    }
}
