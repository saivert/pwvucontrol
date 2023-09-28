// SPDX-License-Identifier: GPL-3.0-or-later

use crate::pwnodeobject::PwNodeObject;

use std::cell::{Cell, RefCell};

use gtk::{
    glib::{self, Properties, Value},
    prelude::*,
    subclass::prelude::*
};

use wireplumber as wp;

mod imp {

    use super::*;

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
        pub(super) volume: Cell<f32>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for PwChannelObject {
        const NAME: &'static str = "PwChannelObject";
        type Type = super::PwChannelObject;
    }

    #[glib::derived_properties]
    impl ObjectImpl for PwChannelObject {}

    impl PwChannelObject {
        fn set_volume(&self, value: &Value) {
            wp::log::debug!(
                "Got set_volume on channel object {} = {:?}",
                self.obj().name(),
                value.get::<f32>()
            );
            let index = self.index.get();
            let volume = value.get::<f32>().expect("f32 for set_volume");
            self.volume.set(volume);

            if let Some(nodeobj) = self.row_data.borrow().as_ref() {
                if nodeobj.channellock() {
                    nodeobj.set_channel_volumes_vec(&vec![volume; nodeobj.channel_volumes_vec().len()]);
                } else {
                    nodeobj.set_channel_volume(index, volume);
                }
            }
        }
    }
}

glib::wrapper! {
    pub struct PwChannelObject(ObjectSubclass<imp::PwChannelObject>);
}

impl PwChannelObject {
    pub(crate) fn new(index: u32, volume: f32, row_data: &PwNodeObject) -> Self {
        let t_audiochannel =
            wp::spa::SpaIdTable::from_name("Spa:Enum:AudioChannel").expect("audio channel type");
        let channel = row_data.format().unwrap().positions[index as usize];
        let channelname = t_audiochannel
            .values()
            .find(|x| x.number() == channel)
            .and_then(|x| x.short_name())
            .unwrap();

        glib::Object::builder()
            .property("index", index)
            .property("volume", volume)
            .property("name", channelname)
            .property("row-data", row_data)
            .build()
    }

    pub fn set_volume_no_send(&self, volume: f32) {
        let imp = self.imp();

        imp.volume.set(volume);
        self.notify_volume();
    }

}
