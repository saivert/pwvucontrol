use gtk::glib;

use crate::pwnodeobject::PwNodeObject;

mod imp {
    use glib::{SignalHandlerId, clone};
    use gtk::subclass::prelude::*;
    use std::cell::{Cell, RefCell};

    use gtk::{
        glib::{self, ParamSpec, Properties, Value},
        prelude::*,
    };

    use crate::pwnodeobject::PwNodeObject;

    // Object holding the state
    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::PwChannelObject)]
    pub struct PwChannelObject {
        #[property(get, set, construct_only)]
        row_data: RefCell<Option<PwNodeObject>>,

        #[property(get, set)]
        name: RefCell<String>,
        #[property(get, set)]
        index: Cell<u32>,
        #[property(get, set = Self::set_volume)]
        volume: Cell<f32>,

        handler: RefCell<Option<SignalHandlerId>>,

        block_volume_send: Cell<bool>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for PwChannelObject {
        const NAME: &'static str = "PwChannelObject";
        type Type = super::PwChannelObject;
    }

    // Trait shared by all GObjects
    impl ObjectImpl for PwChannelObject {
        fn properties() -> &'static [ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &Value, pspec: &ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &ParamSpec) -> Value {
            self.derived_property(id, pspec)
        }

        fn constructed(&self) {
            let item = self.row_data.borrow();
            let item = item.as_ref().cloned().unwrap();

            *self.handler.borrow_mut() = Some(item.connect_channel_volumes_notify(clone!(@weak self as channelobj => @default-panic, move |nodeobj| {
                let values = nodeobj.channel_volumes_vec();
                let index = channelobj.index.get();
                let channelname = crate::format::get_channel_name_for_position(index, nodeobj.format());
                if let Some(pwvolume) = values.get(index as usize) {
                    let volume = *pwvolume;
                    if channelobj.obj().volume() != volume {
                        log::info!("pipewire -> app: setting volume {volume} for index {index}");
                        channelobj.block_volume_send.set(true);
                        channelobj.obj().set_volume(volume);
                        channelobj.block_volume_send.set(false);
                    } else {
                        log::info!("pipewire -> app: volume unchanged");
                    }
                    if channelobj.obj().name() != channelname {
                        log::info!("pipewire -> app: setting channel name '{channelname}' for index {index}");
                        channelobj.obj().set_name(channelname);
                    }
                } else {
                    log::error!("channel volumes array out of bounds");
                }
            })));

        }

        fn dispose(&self) {
            if let Some(signal) = self.handler.take() {
                log::info!("Dispose: Disconnected signal handler");
                self.row_data.borrow_mut().as_ref().cloned().unwrap().disconnect(signal);
            }
        }
    }

    impl PwChannelObject {
        fn set_volume(&self, value: &Value) {
            log::info!("Got set_volume on channel object {:?}", value.get::<f32>());
            let index = self.index.get();
            let volume = value.get::<f32>().expect("f32 for set_volume");
            self.volume.set(volume);
            if self.block_volume_send.get() == false {
                if let Some(nodeobj) = self.row_data.borrow().as_ref() {
                    if nodeobj.channellock() {
                        let vec: Vec<f32> = (0..nodeobj.channel_volumes_vec().len()).map(|_| volume).collect();
                        nodeobj.set_channel_volumes_vec(&vec);
                    } else {
                        nodeobj.set_channel_volume(index, volume);
                    }
                }
            }
        }
    }
}

glib::wrapper! {
    pub struct PwChannelObject(ObjectSubclass<imp::PwChannelObject>);
}

impl PwChannelObject {
    pub fn new(index: u32, volume: f32, row_data: &PwNodeObject) -> Self {
        let channelname = crate::format::get_channel_name_for_position(index, row_data.format());

        glib::Object::builder()
            .property("index", index)
            .property("volume", volume)
            .property("name", channelname)
            .property("row-data", row_data)
            .build()
    }
}
