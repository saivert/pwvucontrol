// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;

use glib::{subclass::types::ObjectSubclassIsExt, ObjectExt, ToVariant, closure_local};
use wireplumber as wp;
use wp::pw::ProxyExt;

use super::*;

impl PwNodeObject {
    pub(crate) fn get_mixer_api(&self) {
        let manager = PwvucontrolManager::default();
        let mixerapi = manager.mixer_api();

        let changed_handler = closure_local!(@watch self as widget => move |_mixerapi: &wp::plugin::Plugin, id: u32|{
            if id == widget.boundid() {
                widget.imp().block.set(true);
                widget.update_volume_using_mixerapi();
                widget.imp().block.set(false);
            }
        });

        mixerapi.connect_closure("changed", true, changed_handler);
    }

    pub(crate) fn send_volume_using_mixerapi(&self, what: PropertyChanged) {
        let imp = self.imp();
        let node = imp.wpnode.get().expect("node in send_volume");
        let manager = PwvucontrolManager::default();
        let mixerapi = manager.mixer_api();
        let bound_id = node.bound_id();
        let result =
            mixerapi.emit_by_name::<Option<glib::Variant>>("get-volume", &[&node.bound_id()]);
        if result.is_none() {
            pwvucontrol_warning!("Node {bound_id} does not support volume");
            return;
        }

        let variant = glib::VariantDict::new(None);
        match what {
            PropertyChanged::Mute => {
                variant.insert("mute", self.mute());
            }
            PropertyChanged::Volume => {
                let mut channel_volumes = self.channel_volumes_vec();
                let max = self.volume();
                let t = *channel_volumes
                    .iter()
                    .max_by(|a, b| a.total_cmp(b))
                    .expect("Max");
                if t > 0.0 {
                    for v in channel_volumes.iter_mut() {
                        *v = *v * max / t;
                    }
                } else {
                    for v in channel_volumes.iter_mut() {
                        *v = max;
                    }
                }
                if let Some(cv) = self.make_channel_volumes_variant(&self.channel_volumes_vec()) {
                    variant.insert("channelVolumes", cv);
                }
                self.set_channel_volumes_vec_no_send(&channel_volumes);
            }
            PropertyChanged::ChannelVolumes => {
                if let Some(cv) = self.make_channel_volumes_variant(&self.channel_volumes_vec()) {
                    variant.insert("channelVolumes", cv);
                }
            }
        }

        let result =
            mixerapi.emit_by_name::<bool>("set-volume", &[&bound_id, &variant.to_variant()]);
        if !result {
            pwvucontrol_warning!("Cannot set volume on {bound_id}");
        }
    }

    fn make_channel_volumes_variant(&self, channel_volumes: &Vec<f32>) -> Option<glib::Variant> {
        let t_audiochannel =
            wp::spa::SpaIdTable::from_name("Spa:Enum:AudioChannel").expect("audio channel type");

        if let Some(format) = self.format() {
            let positions = format.positions;

            let mut channel_volumes_map: HashMap<String, glib::Variant> =
                HashMap::with_capacity(channel_volumes.len());
            for (i, v) in channel_volumes.iter().enumerate() {
                let mut map: HashMap<String, glib::Variant> = HashMap::with_capacity(2);
                let channel_name = t_audiochannel
                    .find_value(positions[i])
                    .expect("channel name")
                    .short_name();
                map.insert("channel".to_string(), channel_name.to_variant());
                map.insert("volume".to_string(), (*v as f64).to_variant());
                channel_volumes_map.insert(i.to_string(), map.to_variant());
            }

            return Some(channel_volumes_map.to_variant());
        }
        None
    }

    pub(crate) fn update_volume_using_mixerapi(&self) {
        let manager = PwvucontrolManager::default();
        let mixerapi = manager.mixer_api();
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
                let mut newvec = vec![0f32; channel_volumes.len()];
                for (index_str, v) in channel_volumes.iter() {
                    let index: u32 = index_str.parse().expect("erroneous index");
                    let map: HashMap<String, glib::Variant> = v.get().unwrap();
                    let volume: Option<f64> = map.get("volume").and_then(|x| x.get());
                    let channelname: String =
                        map.get("channel").and_then(|x| x.get()).unwrap_or_default();
                    let channel = t_audiochannel.find_value_from_short_name(&channelname);

                    if let (Some(c), Some(v)) = (channel, volume) {
                        pwvucontrol_debug!("Index: {index}, Number: {} = {}", c.number(), v);
                        newvec[index as usize] = v as f32;
                    } else {
                        pwvucontrol_critical!("Got invalid data via mixer-api");
                    }
                }
                self.set_channel_volumes_vec(&newvec);
            } else {
                pwvucontrol_critical!("Cannot get channel volumes via mixer-api");
            }

            // Wireplumber's mixerapi always sets volume to first channel volume
            // instead we will use the max channel
            let maxvol: Option<f32> = self
                .channel_volumes_vec()
                .iter()
                .max_by(|a, b| a.total_cmp(b))
                .copied();
            
            if let Some(maxvol) = maxvol {
                self.set_volume(maxvol);
            }

            let mute: Option<bool> = map.get("mute").and_then(|x| x.get());
            if let Some(m) = mute {
                self.set_mute(m);
                pwvucontrol_debug!("Setting mute to {m:?}");
            }

            // Update the liststore
            //self.update_channel_objects();
        }
    }
}
