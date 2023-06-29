use std::collections::HashMap;

use crate::application::PwvucontrolApplication;

use glib::{subclass::types::ObjectSubclassIsExt, clone, ObjectExt, ToVariant};
use wireplumber as wp;
use wp::pw::ProxyExt;

impl super::PwNodeObject {
    pub fn get_mixer_api(&self) {
        let imp = self.imp();

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

        imp.mixerapi
            .set(mixerapi)
            .expect("mixerapi only set once in PwNodeObject");

        imp.mixerapi.get().unwrap().connect_local(
            "changed",
            true,
            clone!(@weak self as obj => @default-return None, move |x| {
                // let mixerapi: wp::plugin::Plugin = x[0].get().expect("MixerApi in changed event");
                let id: u32 = x[1].get().expect("Id in in changed event");
                wp::log::info!("From mixer-api changed event: {id}");
                if id == obj.serial() {
                    obj.update_volume_using_mixerapi();
                }
                None
            }),
        );
    }

    pub fn send_volume_using_mixerapi(&self) {
        let imp = self.imp();
        let node = imp.wpnode.get().expect("node in send_volume");
        let mixerapi = self
            .imp()
            .mixerapi
            .get()
            .expect("Mixer api must be set on PwNodeObject");
        let bound_id = node.bound_id();
        let result =
            mixerapi.emit_by_name::<Option<glib::Variant>>("get-volume", &[&node.bound_id()]);
        if result.is_none() {
            wp::log::warning!("Node {bound_id} does not support volume");
            return;
        }

        let variant = glib::VariantDict::new(None);
        variant.insert("volume", self.volume() as f64);
        variant.insert("mute", self.mute());

        let result =
            mixerapi.emit_by_name::<bool>("set-volume", &[&bound_id, &variant.to_variant()]);
        if result == false {
            wp::log::warning!("Cannot set volume on {bound_id}");
        }
    }

    pub fn update_volume_using_mixerapi(&self) {
        let mixerapi = self
            .imp()
            .mixerapi
            .get()
            .expect("Mixer api must be set on PwNodeObject");
        let node = self
            .imp()
            .wpnode
            .get()
            .expect("WpNode must be set on PwNodeObject");
        let result =
            mixerapi.emit_by_name::<Option<glib::Variant>>("get-volume", &[&node.bound_id()]);
        if let Some(r) = result {
            let map: HashMap<String, glib::Variant> = r.get().unwrap();
            let t_audiochannel = wp::spa::SpaIdTable::from_name("Spa:Enum:AudioChannel")
                .expect("audio channel type");

            let result: Option<HashMap<String, glib::Variant>> =
                map.get("channelVolumes").and_then(|x| x.get());
            if let Some(channel_volumes) = result {
                for (index_str, v) in channel_volumes.iter() {
                    let index: u32 = index_str.parse().expect("erroneous index");
                    let map: HashMap<String, glib::Variant> = v.get().unwrap();
                    let volume: Option<f64> = map.get("volume").and_then(|x| x.get());
                    let channelname: String =
                        map.get("channel").and_then(|x| x.get()).unwrap_or_default();
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

            let volume: Option<f64> = map.get("volume").and_then(|x| x.get());
            if let Some(v) = volume {
                self.set_volume(v as f32);
                wp::log::info!("Setting volume to {v}");
            }

            let mute: Option<bool> = map.get("mute").and_then(|x| x.get());
            if let Some(m) = mute {
                self.set_mute(m);
                wp::log::info!("Setting mute to {m:?}");
            }
            self.imp().block.set(false);
        }
    }
}
