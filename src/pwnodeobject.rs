use glib::Object;
use gtk::glib;


mod imp {
    use std::cell::{Cell, RefCell};
    use std::collections::HashMap;
    use glib::SignalHandlerId;
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
        description: RefCell<Option<String>>,
        #[property(get, set)]
        serial: Cell<u32>,
        #[property(get, set)]
        volume: Cell<f32>,
        #[property(get, set)]
        mute: Cell<bool>,
        #[property(get = Self::channel_volumes, set = Self::set_channel_volumes, type = glib::ValueArray)]
        channel_volumes: RefCell<Vec<f32>>,
        signalblockers: RefCell<HashMap<String, SignalHandlerId>>,
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
            self.obj().notify_channel_volumes();           
        }

        pub fn set_channel_volumes_vec_noevent(&self, values: &Vec<f32>) {
            *(self.channel_volumes.borrow_mut()) = values.clone();
            let obj = self.obj();
            if let Some(sigid) = self.signalblockers.borrow().get("channel-volumes") {
                obj.block_signal(sigid);
                obj.notify_channel_volumes();
                obj.unblock_signal(sigid);
            }
        }

        pub fn set_property_change_handler<F: Fn(&super::PwNodeObject, &glib::ParamSpec) + 'static>(
            &self,
            name: &str,
            handler: F,
        ) {
            let sigid = self.obj().connect_notify_local(Some(name), handler);
            self.signalblockers.borrow_mut().insert(name.to_string(), sigid);
        }

        pub fn set_volume_noevent(&self, volume: f32) {
            let obj = self.obj();
            if let Some(sigid) = self.signalblockers.borrow().get("volume") {
                obj.block_signal(sigid);
                obj.set_volume(volume);
                obj.unblock_signal(sigid);
                return;
            }
            log::error!("Missing signal handler id for volume event");
        }

        pub fn set_mute_noevent(&self, mute: bool) {
            let obj = self.obj();
            if let Some(sigid) = self.signalblockers.borrow().get("mute") {
                obj.block_signal(sigid);
                obj.set_mute(mute);
                obj.unblock_signal(sigid);
                return;
            }
            log::error!("Missing signal handler id for volume event");
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
