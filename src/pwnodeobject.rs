use glib::Object;
use gtk::glib;


mod imp {
    use std::cell::{Cell, RefCell};
    use std::collections::HashMap;
    use glib::SignalHandlerId;
    use glib::subclass::Signal;
    use gtk::subclass::prelude::*;

    use gtk::{
        glib::{self, ParamSpec, Properties, Value},
        prelude::*,
    };
    use once_cell::sync::Lazy;
    
    // Object holding the state
    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::PwNodeObject)]
    pub struct PwNodeObject {
        #[property(get, set)]
        name: RefCell<Option<String>>,
        #[property(get, set)]
        description: RefCell<Option<String>>,
        #[property(get, set)]
        formatstr: RefCell<Option<String>>,
        #[property(get, set)]
        serial: Cell<u32>,
        #[property(get, set)]
        volume: Cell<f32>,
        #[property(get, set)]
        mute: Cell<bool>,
        #[property(get = Self::channel_volumes, set = Self::set_channel_volumes, type = glib::ValueArray)]
        channel_volumes: RefCell<Vec<f32>>,
        #[property(get, set, builder(crate::NodeType::Undefined))]
        node_type: Cell<crate::NodeType>,

        signalblockers: RefCell<HashMap<String, SignalHandlerId>>,
        format: Cell<Option<pipewire::spa::sys::spa_audio_info_raw>>
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

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder("channelvolume")
                    .param_types([u32::static_type(), f32::static_type()])
                    .build()]
            });

            SIGNALS.as_ref()
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

        pub fn channel_volumes_vec(&self) -> Vec<f32> {
            self.channel_volumes.borrow().clone()
        }

        pub fn set_channel_volumes_vec(&self, values: &Vec<f32>) {
            *(self.channel_volumes.borrow_mut()) = values.clone();
            self.obj().notify_channel_volumes();
        }

        pub fn set_channel_volumes_vec_noevent(&self, values: &Vec<f32>) {
            *(self.channel_volumes.borrow_mut()) = values.clone();
            let obj = self.obj();
            // If a signal blocker is registered then use it
            if let Some(sigid) = self.signalblockers.borrow().get("channel-volumes") {
                obj.block_signal(sigid);
                obj.notify_channel_volumes();
                obj.unblock_signal(sigid);
                return;
            }
            // Otherwise just let the property change notify happen
            obj.notify_channel_volumes();
        }

        pub fn set_channel_volume(&self, index: u32, volume: f32) {
            if let Some (value) = self.channel_volumes.borrow_mut().get_mut(index as usize) {
                *value = volume;
            }
            self.obj().emit_by_name::<()>("channelvolume", &[&index, &volume]);
        }

        pub fn set_property_change_handler_with_blocker<F: Fn(&super::PwNodeObject, &glib::ParamSpec) + 'static>(
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
            obj.set_volume(volume);
        }

        pub fn set_mute_noevent(&self, mute: bool) {
            let obj = self.obj();
            if let Some(sigid) = self.signalblockers.borrow().get("mute") {
                obj.block_signal(sigid);
                obj.set_mute(mute);
                obj.unblock_signal(sigid);
                return;
            }
            obj.set_mute(mute);
        }

        pub fn set_format(&self, format: pipewire::spa::sys::spa_audio_info_raw) {
            self.format.set(Some(format));

            self.obj().notify_channel_volumes();
        }

        pub fn set_format_noevent(&self, format: pipewire::spa::sys::spa_audio_info_raw) {
            self.format.set(Some(format));

            let obj = self.obj();
            // Reuse channel-volumes event here because channel-volumes may also change if format changes
            if let Some(sigid) = self.signalblockers.borrow().get("channel-volumes") {
                obj.block_signal(sigid);
                obj.notify_channel_volumes();
                obj.unblock_signal(sigid);
                return;
            }
            obj.notify_channel_volumes();
        }

        pub fn format(&self) -> Option<pipewire::spa::sys::spa_audio_info_raw> {
            self.format.get()
        }


    }
}

glib::wrapper! {
    pub struct PwNodeObject(ObjectSubclass<imp::PwNodeObject>);
}

impl PwNodeObject {
    pub fn new(serial: u32, name: &str, nodetype: crate::NodeType) -> Self {
        Object::builder()
            .property("serial", serial)
            .property("name", name)
            .property("node-type", nodetype)
            .build()
    }
}


#[test]
fn test_nodetype() {
    let object = PwNodeObject::new(0, "test", crate::NodeType::Input);

    assert_eq!(object.node_type(), crate::NodeType::Input);
}

#[test]
fn test_channel_volume_get() {
    use glib::{ValueArray, Value};
    use gtk::subclass::prelude::*;

    let object = PwNodeObject::new(0, "test", crate::NodeType::Input);
    let mut value = ValueArray::new(2);
    value.append(&Value::from(0.5f32));
    value.append(&Value::from(0.6f32));
    object.set_channel_volumes(value);

    let vec = object.imp().channel_volumes_vec();

    assert_eq!(vec.len(), 2);

    assert_eq!(vec[0], 0.5);
    assert_eq!(vec[1], 0.6);
}