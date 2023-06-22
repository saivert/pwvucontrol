use glib::{subclass::prelude::*, Object, ObjectExt};
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
        pub(super) channel_volumes: RefCell<Vec<f32>>,
        #[property(get, set, builder(crate::NodeType::Undefined))]
        node_type: Cell<crate::NodeType>,

        pub(super) signalblockers: RefCell<HashMap<String, SignalHandlerId>>,
        pub(super) format: Cell<Option<pipewire::spa::sys::spa_audio_info_raw>>,
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
                    .build(),
                    Signal::builder("format")
                    .build()
                    ]
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

    pub fn channel_volumes_vec(&self) -> Vec<f32> {
        self.imp().channel_volumes.borrow().clone()
    }

    pub fn set_channel_volumes_vec(&self, values: &Vec<f32>) {
        *(self.imp().channel_volumes.borrow_mut()) = values.clone();
        self.notify_channel_volumes();
    }


    pub fn set_channel_volumes_vec_noevent(&self, values: &Vec<f32>) {
        *(self.imp().channel_volumes.borrow_mut()) = values.clone();

        // If a signal blocker is registered then use it
        if let Some(sigid) = self.imp().signalblockers.borrow().get("channel-volumes") {
            self.block_signal(sigid);
            self.notify_channel_volumes();
            self.unblock_signal(sigid);
            return;
        }
        // Otherwise just let the property change notify happen
        self.notify_channel_volumes();
    }

    pub fn set_channel_volume(&self, index: u32, volume: f32) {
        if let Some (value) = self.imp().channel_volumes.borrow_mut().get_mut(index as usize) {
            *value = volume;
        }
        self.emit_by_name::<()>("channelvolume", &[&index, &volume]);
    }

    pub fn set_property_change_handler_with_blocker<F: Fn(&PwNodeObject, &glib::ParamSpec) + 'static>(
        &self,
        name: &str,
        handler: F,
    ) {
        let sigid = self.connect_notify_local(Some(name), handler);
        self.imp().signalblockers.borrow_mut().insert(name.to_string(), sigid);
    }

    pub fn set_volume_noevent(&self, volume: f32) {
        if let Some(sigid) = self.imp().signalblockers.borrow().get("volume") {
            self.block_signal(sigid);
            self.set_volume(volume);
            self.unblock_signal(sigid);
            return;
        }
        self.set_volume(volume);
    }

    pub fn set_mute_noevent(&self, mute: bool) {
        if let Some(sigid) = self.imp().signalblockers.borrow().get("mute") {
            self.block_signal(sigid);
            self.set_mute(mute);
            self.unblock_signal(sigid);
            return;
        }
        self.set_mute(mute);
    }

    pub fn set_format(&self, format: pipewire::spa::sys::spa_audio_info_raw) {
        self.imp().format.set(Some(format));

        // self.notify_channel_volumes();
        self.emit_by_name::<()>("format", &[]);
    }

    pub fn set_format_noevent(&self, format: pipewire::spa::sys::spa_audio_info_raw) {
        self.imp().format.set(Some(format));

        // Reuse channel-volumes event here because channel-volumes may also change if format changes
        if let Some(sigid) = self.imp().signalblockers.borrow().get("channel-volumes") {
            self.block_signal(sigid);
            self.emit_by_name::<()>("format", &[]);
            self.unblock_signal(sigid);
            return;
        }
        self.emit_by_name::<()>("format", &[]);
    }

    pub fn format(&self) -> Option<pipewire::spa::sys::spa_audio_info_raw> {
        self.imp().format.get()
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

    let object = PwNodeObject::new(0, "test", crate::NodeType::Input);
    let mut value = ValueArray::new(2);
    value.append(&Value::from(0.5f32));
    value.append(&Value::from(0.6f32));
    object.set_channel_volumes(value);

    let vec = object.channel_volumes_vec();

    assert_eq!(vec.len(), 2);

    assert_eq!(vec[0], 0.5);
    assert_eq!(vec[1], 0.6);
}