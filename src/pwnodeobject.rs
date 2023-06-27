use std::collections::HashMap;

use glib::{clone, subclass::prelude::*, Object, ObjectExt, ToVariant};
use gtk::glib;

use wireplumber as wp;
use wp::pw::{GlobalProxyExt, PipewireObjectExt, PipewireObjectExt2, ProxyExt};

use crate::{NodeType, application::PwvucontrolApplication};

#[derive(Copy, Clone, Debug)]
pub struct AudioFormat {
    pub channels: i32,
    pub format: u32,
    pub rate: i32,
    pub positions: [u32; 64],
}

mod imp {
    use glib::subclass::Signal;
    use glib::SignalHandlerId;
    use gtk::subclass::prelude::*;
    use std::cell::{Cell, RefCell};
    use std::collections::HashMap;

    use gtk::{
        glib::{self, ParamSpec, Properties, Value},
        prelude::*,
    };
    use once_cell::sync::{Lazy, OnceCell};

    use wireplumber as wp;

    use super::AudioFormat;

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
        nodetype: Cell<crate::NodeType>,

        pub(super) signalblockers: RefCell<HashMap<String, SignalHandlerId>>,
        pub(super) format: Cell<Option<AudioFormat>>,

        #[property(get, set)]
        pub(super) channellock: Cell<bool>,

        pub(super) wpnode: OnceCell<wp::pw::Node>,
        pub(super) mixerapi: OnceCell<wp::plugin::Plugin>,

        pub(super) block: Cell<bool>,
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
            self.derived_set_property(id, value, pspec);
            match pspec.name() {
                "volume" | "mute" => {
                    if self.block.get() == false {
                        self.obj().send_volume();
                    }
                },
                _ => {},
            }
        }

        fn property(&self, id: usize, pspec: &ParamSpec) -> Value {
            self.derived_property(id, pspec)
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(|| vec![Signal::builder("format").build()]);

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
    pub fn new(serial: u32, name: &str, node: &wp::pw::Node) -> Self {
        let nodetype = match node.get_pw_property("media.class").as_deref() {
            Some("Stream/Output/Audio") => NodeType::Output,
            Some("Stream/Input/Audio") => NodeType::Input,
            Some("Audio/Sink") => NodeType::Sink,
            _ => NodeType::Undefined,
        };

        let obj: PwNodeObject = Object::builder()
            .property("serial", serial)
            .property("name", name)
            .property("nodetype", nodetype)
            .build();

        obj.imp()
            .wpnode
            .set(node.clone())
            .expect("Can only set PwNodeObject's wpnode once");

        node.connect_notify_local(
            Some("global-properties"),
            clone!(@weak obj => move  |_, _| {
                obj.label_set_name()
            }),
        );

        node.connect_notify_local(
            Some("properties"),
            clone!(@weak obj => move  |_, _| {
                obj.label_set_description();
            }),
        );

        // Use mixer-api

        let app = PwvucontrolApplication::default();
        let core = app.imp().wp_core.get().expect("Core setup");

        let mixerapi = wp::plugin::Plugin::find(&core, "mixer-api").expect("Get mixer-api");

        // If we need to set cubic scale...
        // let t = glib::Type::from_name("WpMixerApiVolumeScale").unwrap();
        // let v = glib::Value::from_type(t);
        // unsafe {
        //     glib::gobject_ffi::g_value_set_enum(v.as_ptr(), 1);
        // }
        // mixerapi.set_property("scale", v);

        obj.imp().mixerapi.set(mixerapi).expect("mixerapi only set once in PwNodeObject");

        obj.imp().mixerapi.get().unwrap().connect_local("changed", true, clone!(@weak obj => @default-return None, move |x| {
            // let mixerapi: wp::plugin::Plugin = x[0].get().expect("MixerApi in changed event");
            let id: u32 = x[1].get().expect("Id in in changed event");
            wp::log::info!("From mixer-api changed event: {id}");
            if id == obj.serial() {
                obj.update_volume_using_mixerapi();
            }
            None
        }));

        node.connect_params_changed(clone!(@weak obj => move |_node,what| {
            wp::log::info!("params-changed! {what}");
            obj.imp().block.set(true);
            match what {
                // "Props" => obj.update_channel_volumes(),
                "Format" => obj.update_format(),
                _ => {},
            }
            obj.imp().block.set(false);
        }));

        obj.label_set_description();
        obj.update_channel_volumes();
        obj.update_format();
        // obj.update_volume_using_mixerapi();

        obj
    }

    fn label_set_name(&self) {
        let wp_node = self
            .imp()
            .wpnode
            .get()
            .expect("Node widget should always have a wp_node");
        let props = wp_node.global_properties().expect("Node has no properties");
        let name_gstr = props
            .get("node.nick")
            .or_else(|| props.get("node.description"))
            .or_else(|| props.get("node.name"));

        let name = name_gstr
            .as_ref()
            .map(|name| name.as_str())
            .unwrap_or_default();

        self.set_name(name);
    }

    fn label_set_description(&self) {
        let wp_node = self
            .imp()
            .wpnode
            .get()
            .expect("Node widget should always have a wp_node");
        let props = wp_node.properties().expect("Node has no properties");
        let name_gstr = props
            .get("media.name");

        let name = name_gstr
            .as_ref()
            .map(|name| name.as_str())
            .unwrap_or_default();

        self.set_description(name);
    }

    pub fn update_volume_using_mixerapi(&self) {
        let mixerapi = self.imp().mixerapi.get().expect("Mixer api must be set on PwNodeObject");
        let node = self.imp().wpnode.get().expect("WpNode must be set on PwNodeObject");
        let result = mixerapi.emit_by_name::<Option<glib::Variant>>("get-volume", &[&node.bound_id()]);
        if let Some(r) = result {
            let map: HashMap<String, glib::Variant> = r.get().unwrap();
            let t_audiochannel = wp::spa::SpaIdTable::from_name("Spa:Enum:AudioChannel").expect("audio channel type");

            let result: Option<HashMap<String, glib::Variant>> = map.get("channelVolumes").and_then(|x|x.get());
            if let Some(channel_volumes) = result {
                for (index_str, v) in channel_volumes.iter() {
                    let index: u32 = index_str.parse().expect("erroneous index");
                    let map: HashMap<String, glib::Variant> = v.get().unwrap();
                    let volume: Option<f64> = map.get("volume").and_then(|x|x.get());
                    let channelname: String = map.get("channel").and_then(|x|x.get()).unwrap_or_default();
                    let channel = t_audiochannel.find_value_from_short_name(&channelname);

                    if let (Some(c), Some(v)) = (channel, volume) {
                        wp::log::info!("Index: {index}, Number: {} = {}", c.number(), v);
                        self.set_channel_volume(index, v as f32); // TODO: get index via channel map, index of vardict must not be relied upon
                    } else {
                        wp::log::critical!("Got invalid data via mixer-api");
                    }
                }
            } else {
                wp::log::critical!("Cannot get channel volumes via mixer-api");
            }

            self.imp().block.set(true);

            let volume: Option<f64> =  map.get("volume").and_then(|x|x.get());
            if let Some(v) = volume {
                self.set_volume(v as f32);
                wp::log::info!("Setting volume to {v}");
            }

            let mute: Option<bool> = map.get("mute").and_then(|x|x.get());
            if let Some(m) = mute {
                self.set_mute(m);
                wp::log::info!("Setting mute to {m:?}");
            }
            self.imp().block.set(false);

        }
    }

    pub fn update_format(&self) {
        let node = self.imp().wpnode.get().expect("node");

        node.enum_params(Some("Format"), None, gtk::gio::Cancellable::NONE, clone!(@weak self as widget, @weak node => move |res| {
            if let Ok(Some(iter)) = res {
                for a in iter {
                    let pod: wp::spa::SpaPod = a.get().unwrap();
                    if !pod.is_object() {
                        continue;
                    }

                    let keys = wp::spa::SpaIdTable::from_name("Spa:Pod:Object:Param:Format").expect("id table");
                    let channels_key = keys.find_value_from_short_name("channels").expect("channels key");
                    let rate_key = keys.find_value_from_short_name("rate").expect("channels key");
                    let format_key = keys.find_value_from_short_name("format").expect("format key");
                    let position_key = keys.find_value_from_short_name("position").expect("position key");

                    fn get_pod_maybe_choice(pod: wp::spa::SpaPod) -> wp::spa::SpaPod {
                        if pod.is_choice() {
                            pod.choice_child().unwrap()
                        } else {
                            pod
                        }
                    }

                    let choice = pod.find_spa_property(&format_key).expect("Format!");
                    let format = get_pod_maybe_choice(choice).id().expect("Format id");
                    if format == 0 {
                        wp::log::warning!("Format is 0, ignoring...");
                        return;
                    }

                    let choice = pod.find_spa_property(&channels_key).expect("Channels!");
                    let channels = get_pod_maybe_choice(choice).int().expect("Channels int");

                    let choice = pod.find_spa_property(&rate_key).expect("Rate!");
                    let rate = get_pod_maybe_choice(choice).int().expect("Rate int");

                    let positionpod = pod.find_spa_property(&position_key).expect("Position!");
                    let position: [u32; 64] = {
                            let vec: Vec<u32> = positionpod.array_iterator().map(|x: i32| x as u32).collect();
                            let mut a = [0u32;64];
                            for (i,v) in vec.iter().enumerate() {
                                a[i] = *v;
                            }
                            a
                        };

                    wp::log::info!("For id {}, Got rate {rate}, format {format}, channels {channels}", node.bound_id());

                    let t_format = wp::spa::SpaIdTable::from_name("Spa:Enum:AudioFormat").expect("audio format type");
                    let formatname = t_format.values().find(|x| x.number() == format).and_then(|x|x.short_name()).unwrap();

                    widget.set_formatstr(format!("{}ch {}Hz {}", channels, rate, formatname));

                    widget.set_format(AudioFormat { channels, format, rate, positions: position });
                }
            } else {
                wp::log::debug!("enum_params async call didn't return anything useful");
            }
            
        }));
    }

    pub fn update_channel_volumes(&self) {
        let node = self.imp().wpnode.get().expect("node");

        let params = node
            .enum_params_sync("Props", None)
            .expect("getting params");

        for a in params {
            let pod: wp::spa::SpaPod = a.get().unwrap();
            if pod.is_object() {
                let keys =
                    wp::spa::SpaIdTable::from_name("Spa:Pod:Object:Param:Props").expect("id table");
                let channelvolumes_key = keys
                    .find_value_from_short_name("channelVolumes")
                    .expect("channelVolumes key");
                let volume_key = keys
                    .find_value_from_short_name("volume")
                    .expect("volume key");
                let mute_key = keys
                    .find_value_from_short_name("mute")
                    .expect("mute key");

                if let Some(val) = pod.find_spa_property(&channelvolumes_key) {
                    let mut volumes: Vec<f32> = Vec::new();
                    for a in val.array_iterator() {
                        volumes.push(a);
                    }
                    if volumes.len() == 0 {
                        wp::log::warning!("Got 0 channel volumes, ignoring...");
                        return;
                    }
                    if *volumes.first().unwrap() == 0f32 {
                        wp::log::warning!("Got 0 as first volume, ignoring...");
                        return;
                    }
                    self.set_channel_volumes_vec(&volumes);
                    let avgvol: f32 = volumes.iter().sum::<f32>() / volumes.len() as f32;
                    self.set_volume(avgvol);
                }

                // if let Some(val) = pod.find_spa_property(&volume_key) {
                //     if let Some(volume) = val.float() {
                //         self.set_volume(volume);
                //     }
                // }

                if let Some(val) = pod.find_spa_property(&mute_key) {
                    if let Some(mute) = val.boolean() {
                        self.set_mute(mute);
                    }
                }
            }
        }
    }

    pub fn send_volume(&self) {
        let imp = self.imp();
        let node = imp.wpnode.get().expect("node in send_volume");
        let mixerapi = self.imp().mixerapi.get().expect("Mixer api must be set on PwNodeObject");
        let bound_id = node.bound_id();
        let result = mixerapi.emit_by_name::<Option<glib::Variant>>("get-volume", &[&node.bound_id()]);
        if result.is_none() {
            wp::log::warning!("Node {bound_id} does not support volume");
            return;
        }

        let variant = glib::VariantDict::new(None);
        variant.insert("volume", self.volume() as f64);
        variant.insert("mute", self.mute());

        let result = mixerapi.emit_by_name::<bool>("set-volume", &[&bound_id, &variant.to_variant()]);
        if result == false {
            wp::log::warning!("Cannot set volume on {bound_id}");
        }
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
        if let Some(value) = self
            .imp()
            .channel_volumes
            .borrow_mut()
            .get_mut(index as usize)
        {
            *value = volume;
        }
        self.notify_channel_volumes();
    }

    pub fn set_property_change_handler_with_blocker<
        F: Fn(&PwNodeObject, &glib::ParamSpec) + 'static,
    >(
        &self,
        name: &str,
        handler: F,
    ) {
        let sigid = self.connect_notify_local(Some(name), handler);
        self.imp()
            .signalblockers
            .borrow_mut()
            .insert(name.to_string(), sigid);
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

    pub fn set_format(&self, format: AudioFormat) {
        self.imp().format.set(Some(format));

        self.emit_by_name::<()>("format", &[]);
    }

    pub fn format(&self) -> Option<AudioFormat> {
        self.imp().format.get()
    }
}
