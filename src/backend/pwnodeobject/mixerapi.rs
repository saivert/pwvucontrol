// SPDX-License-Identifier: GPL-3.0-or-later

use std::{cell::Cell, collections::HashMap};

use glib::closure_local;
use wireplumber as wp;
use wp::pw::ProxyExt;

use super::*;

#[derive(Debug, PartialEq)]
struct MixerVolumeState {
    channel_volumes: Option<Vec<f64>>,
    volume: Option<f64>,
    mute: Option<bool>,
}

struct BlockRestore<'a> {
    block: &'a Cell<bool>,
    previous: bool,
}

impl Drop for BlockRestore<'_> {
    fn drop(&mut self) {
        self.block.set(self.previous);
    }
}

fn valid_volume(value: &glib::Variant) -> Option<f64> {
    value.get::<f64>().filter(|value| value.is_finite() && *value >= 0.0 && *value <= f32::MAX as f64)
}

fn parse_channel_volumes(value: &glib::Variant) -> Option<Vec<f64>> {
    let channel_volumes = value.get::<HashMap<String, glib::Variant>>()?;
    if channel_volumes.is_empty() {
        return None;
    }

    // Do not size a vector from an untrusted index. A contiguous map with N
    // entries can only contain indices in 0..N.
    let len = channel_volumes.len();
    let mut indexed_volumes = Vec::with_capacity(len);
    for (index, entry) in channel_volumes {
        let index = index.parse::<usize>().ok()?;
        if index >= len {
            return None;
        }

        let entry = entry.get::<HashMap<String, glib::Variant>>()?;
        let volume = entry.get("volume").and_then(valid_volume)?;
        indexed_volumes.push((index, volume));
    }

    indexed_volumes.sort_unstable_by_key(|(index, _)| *index);
    if indexed_volumes.iter().enumerate().any(|(expected, (actual, _))| expected != *actual) {
        return None;
    }

    Some(indexed_volumes.into_iter().map(|(_, volume)| volume).collect())
}

fn parse_mixer_volume_state(map: &HashMap<String, glib::Variant>) -> MixerVolumeState {
    let channel_volumes = map.get("channelVolumes").and_then(parse_channel_volumes);
    let volume = channel_volumes
        .as_ref()
        .and_then(|volumes| volumes.iter().copied().max_by(f64::total_cmp))
        .or_else(|| map.get("volume").and_then(valid_volume));
    let mute = map.get("mute").and_then(glib::Variant::get::<bool>);

    MixerVolumeState { channel_volumes, volume, mute }
}

fn volume_as_f32(volume: f64) -> f32 {
    volume as f32
}

fn apply_mixer_volume_state(
    state: MixerVolumeState,
    block: &Cell<bool>,
    mut set_channel_volumes: impl FnMut(&[f32]),
    mut set_volume: impl FnMut(f32),
    mut set_mute: impl FnMut(bool),
) {
    let previous = block.replace(true);
    let _restore = BlockRestore { block, previous };

    if let Some(channel_volumes) = state.channel_volumes {
        let channel_volumes = channel_volumes.into_iter().map(volume_as_f32).collect::<Vec<_>>();
        set_channel_volumes(&channel_volumes);
    }

    if let Some(volume) = state.volume {
        set_volume(volume_as_f32(volume));
    }

    if let Some(mute) = state.mute {
        set_mute(mute);
    }
}

fn channel_volumes_to_variant(channel_volumes: &[f32], channel_names: Option<&[Option<String>]>) -> Option<glib::Variant> {
    if channel_volumes.is_empty() {
        return None;
    }

    let mut channel_volumes_map = HashMap::with_capacity(channel_volumes.len());
    for (index, volume) in channel_volumes.iter().enumerate() {
        let mut entry = HashMap::with_capacity(2);
        entry.insert("volume".to_string(), (*volume as f64).to_variant());
        if let Some(channel_name) = channel_names.and_then(|names| names.get(index)).and_then(Option::as_ref) {
            entry.insert("channel".to_string(), channel_name.to_variant());
        }
        channel_volumes_map.insert(index.to_string(), entry.to_variant());
    }

    Some(channel_volumes_map.to_variant())
}

impl PwNodeObject {
    pub(crate) fn get_mixer_api(&self) {
        let manager = PwvucontrolManager::default();
        let mixerapi = manager.mixer_api();

        let changed_handler = closure_local!(
            #[watch(rename_to = widget)]
            self,
            move |_mixerapi: &wp::plugin::Plugin, id: u32| {
                if id == widget.boundid() {
                    widget.update_volume_using_mixerapi();
                }
            }
        );

        mixerapi.connect_closure("changed", true, changed_handler);
    }

    pub(crate) fn send_volume_using_mixerapi(&self, what: PropertyChanged) {
        let node = self.wpnode();
        let manager = PwvucontrolManager::default();
        let mixerapi = manager.mixer_api();
        let bound_id = node.bound_id();
        let result = mixerapi.emit_by_name::<Option<glib::Variant>>("get-volume", &[&node.bound_id()]);
        if result.is_none() {
            pwvucontrol_warning!("Node {bound_id} does not support volume");
            return;
        }

        let variant = glib::VariantDict::new(None);
        let has_payload = match what {
            PropertyChanged::Mute => {
                variant.insert("mute", self.mute());
                true
            }
            PropertyChanged::Volume => {
                let mut channel_volumes = self.channel_volumes_vec();
                let max = self.volume();
                if let Some(current_max) = channel_volumes.iter().copied().max_by(f32::total_cmp) {
                    if current_max > 0.0 {
                        for volume in &mut channel_volumes {
                            *volume = *volume * max / current_max;
                        }
                    } else {
                        channel_volumes.fill(max);
                    }

                    if let Some(channel_volumes_variant) = self.make_channel_volumes_variant(&channel_volumes) {
                        variant.insert("channelVolumes", channel_volumes_variant);
                    }
                    self.set_channel_volumes_vec_no_send(&channel_volumes);
                } else {
                    variant.insert("volume", max as f64);
                }
                true
            }
            PropertyChanged::ChannelVolumes => {
                if let Some(channel_volumes) = self.make_channel_volumes_variant(&self.channel_volumes_vec()) {
                    variant.insert("channelVolumes", channel_volumes);
                    true
                } else {
                    false
                }
            }
        };

        if !has_payload {
            return;
        }

        let result = mixerapi.emit_by_name::<bool>("set-volume", &[&bound_id, &variant.to_variant()]);
        if !result {
            pwvucontrol_warning!("Cannot set volume on {bound_id}");
        }
    }

    fn make_channel_volumes_variant(&self, channel_volumes: &[f32]) -> Option<glib::Variant> {
        let channel_names = self.format().and_then(|format| {
            let audio_channels = wp::spa::SpaIdTable::from_name("Spa:Enum:AudioChannel")?;
            Some(
                (0..channel_volumes.len())
                    .map(|index| {
                        format
                            .positions
                            .get(index)
                            .and_then(|position| audio_channels.find_value(*position))
                            .and_then(|value| value.short_name())
                            .map(|name| name.to_string())
                    })
                    .collect::<Vec<_>>(),
            )
        });

        channel_volumes_to_variant(channel_volumes, channel_names.as_deref())
    }

    pub(crate) fn update_volume_using_mixerapi(&self) {
        let manager = PwvucontrolManager::default();
        let mixerapi = manager.mixer_api();
        let node = self.imp().wpnode.get().expect("WpNode must be set on PwNodeObject");
        let result = mixerapi.emit_by_name::<Option<glib::Variant>>("get-volume", &[&node.bound_id()]);
        let Some(map) = result.and_then(|result| result.get::<HashMap<String, glib::Variant>>()) else {
            return;
        };

        let state = parse_mixer_volume_state(&map);
        let block = &self.imp().block;
        apply_mixer_volume_state(
            state,
            block,
            |channel_volumes| self.set_channel_volumes_vec(channel_volumes),
            |volume| self.set_volume(volume),
            |mute| {
                self.set_mute(mute);
                pwvucontrol_debug!("Setting mute to {mute:?}");
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn channel_entry(volume: glib::Variant, channel: Option<&str>) -> glib::Variant {
        let mut entry = HashMap::new();
        entry.insert("volume".to_string(), volume);
        if let Some(channel) = channel {
            entry.insert("channel".to_string(), channel.to_variant());
        }
        entry.to_variant()
    }

    fn mixer_map(
        channel_volumes: Option<HashMap<String, glib::Variant>>,
        volume: Option<glib::Variant>,
        mute: Option<bool>,
    ) -> HashMap<String, glib::Variant> {
        let mut map = HashMap::new();
        if let Some(channel_volumes) = channel_volumes {
            map.insert("channelVolumes".to_string(), channel_volumes.to_variant());
        }
        if let Some(volume) = volume {
            map.insert("volume".to_string(), volume);
        }
        if let Some(mute) = mute {
            map.insert("mute".to_string(), mute.to_variant());
        }
        map
    }

    #[test]
    fn parses_unlabeled_contiguous_channels_and_uses_max() {
        let channels = HashMap::from([
            ("0".to_string(), channel_entry(0.25_f64.to_variant(), None)),
            ("1".to_string(), channel_entry(0.75_f64.to_variant(), None)),
        ]);

        let state = parse_mixer_volume_state(&mixer_map(Some(channels), None, None));

        assert_eq!(state.channel_volumes, Some(vec![0.25, 0.75]));
        assert_eq!(state.volume, Some(0.75));
    }

    #[test]
    fn valid_channel_vector_takes_precedence_over_scalar() {
        let channels = HashMap::from([
            ("0".to_string(), channel_entry(0.4_f64.to_variant(), Some("FL"))),
            ("1".to_string(), channel_entry(0.6_f64.to_variant(), Some("FR"))),
        ]);

        let state = parse_mixer_volume_state(&mixer_map(Some(channels), Some(0.1_f64.to_variant()), None));

        assert_eq!(state.volume, Some(0.6));
    }

    #[test]
    fn falls_back_to_scalar_including_zero() {
        let state = parse_mixer_volume_state(&mixer_map(None, Some(0.0_f64.to_variant()), None));
        assert_eq!(state.channel_volumes, None);
        assert_eq!(state.volume, Some(0.0));

        let state = parse_mixer_volume_state(&mixer_map(Some(HashMap::new()), Some(0.3_f64.to_variant()), None));
        assert_eq!(state.channel_volumes, None);
        assert_eq!(state.volume, Some(0.3));
    }

    #[test]
    fn invalid_channel_vectors_fall_back_without_allocating_from_indices() {
        let invalid_channels = [
            HashMap::from([("1".to_string(), channel_entry(0.2_f64.to_variant(), None))]),
            HashMap::from([
                ("0".to_string(), channel_entry(0.2_f64.to_variant(), None)),
                ("2".to_string(), channel_entry(0.4_f64.to_variant(), None)),
            ]),
            HashMap::from([("999999999999999999999999999999999999".to_string(), channel_entry(0.2_f64.to_variant(), None))]),
            HashMap::from([("left".to_string(), channel_entry(0.2_f64.to_variant(), None))]),
            HashMap::from([("0".to_string(), "wrong entry type".to_variant())]),
            HashMap::from([("0".to_string(), channel_entry(1_u32.to_variant(), None))]),
            HashMap::from([("0".to_string(), channel_entry((-0.1_f64).to_variant(), None))]),
            HashMap::from([("0".to_string(), channel_entry(f64::NAN.to_variant(), None))]),
            HashMap::from([("0".to_string(), channel_entry(f64::INFINITY.to_variant(), None))]),
            HashMap::from([("0".to_string(), channel_entry(f64::MAX.to_variant(), None))]),
        ];

        for channels in invalid_channels {
            let state = parse_mixer_volume_state(&mixer_map(Some(channels), Some(0.7_f64.to_variant()), None));
            assert_eq!(state.channel_volumes, None);
            assert_eq!(state.volume, Some(0.7));
        }

        let mut wrong_channel_type = mixer_map(None, Some(0.7_f64.to_variant()), None);
        wrong_channel_type.insert("channelVolumes".to_string(), "wrong channelVolumes type".to_variant());
        let state = parse_mixer_volume_state(&wrong_channel_type);
        assert_eq!(state.channel_volumes, None);
        assert_eq!(state.volume, Some(0.7));
    }

    #[test]
    fn invalid_scalar_leaves_volume_unset_while_mute_is_independent() {
        for invalid_volume in
            [(-0.1_f64).to_variant(), f64::NAN.to_variant(), f64::INFINITY.to_variant(), f64::MAX.to_variant(), "wrong type".to_variant()]
        {
            let state = parse_mixer_volume_state(&mixer_map(None, Some(invalid_volume), Some(true)));
            assert_eq!(state.channel_volumes, None);
            assert_eq!(state.volume, None);
            assert_eq!(state.mute, Some(true));
        }
    }

    #[test]
    fn outgoing_channels_use_numeric_indices_without_format() {
        let variant = channel_volumes_to_variant(&[0.25, 0.5], None).expect("nonempty channel payload");
        let channels = variant.get::<HashMap<String, glib::Variant>>().expect("channel map");

        assert_eq!(channels.len(), 2);
        for (index, expected) in [("0", 0.25_f64), ("1", 0.5_f64)] {
            let entry = channels[index].get::<HashMap<String, glib::Variant>>().expect("channel entry");
            assert_eq!(entry["volume"].get::<f64>(), Some(expected));
            assert!(!entry.contains_key("channel"));
        }
        assert!(channel_volumes_to_variant(&[], None).is_none());
    }

    #[test]
    fn hydration_applies_complete_state_without_writes_and_restores_block() {
        #[derive(Default)]
        struct Probe {
            channel_volumes: Vec<f32>,
            volume: f32,
            mute: bool,
            writes: usize,
        }

        for previous in [false, true] {
            let block = Cell::new(previous);
            let probe = std::cell::RefCell::new(Probe::default());
            let count_write_if_unblocked = || usize::from(!block.get());

            apply_mixer_volume_state(
                MixerVolumeState { channel_volumes: Some(vec![0.25, 0.75]), volume: Some(0.75), mute: Some(true) },
                &block,
                |channel_volumes| {
                    let mut probe = probe.borrow_mut();
                    probe.channel_volumes = channel_volumes.to_vec();
                    probe.writes += count_write_if_unblocked();
                },
                |volume| {
                    let mut probe = probe.borrow_mut();
                    probe.volume = volume;
                    probe.writes += count_write_if_unblocked();
                },
                |mute| {
                    let mut probe = probe.borrow_mut();
                    probe.mute = mute;
                    probe.writes += count_write_if_unblocked();
                },
            );

            let probe = probe.into_inner();
            assert_eq!(probe.channel_volumes, vec![0.25, 0.75]);
            assert_eq!(probe.volume, 0.75);
            assert!(probe.mute);
            assert_eq!(probe.writes, 0);
            assert_eq!(block.get(), previous);
        }
    }
}
