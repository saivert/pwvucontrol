use wireplumber::pipewire::spa::sys::*;

pub fn format_to_string(format: u32) -> &'static str {
    match format {
        SPA_AUDIO_FORMAT_UNKNOWN => "UNKNOWN",
        SPA_AUDIO_FORMAT_ENCODED => "ENCODED",
        SPA_AUDIO_FORMAT_S8 => "S8",
        SPA_AUDIO_FORMAT_U8 => "U8",
        SPA_AUDIO_FORMAT_S16_LE => "S16_LE",
        SPA_AUDIO_FORMAT_S16_BE => "S16_BE",
        SPA_AUDIO_FORMAT_U16_LE => "U16_LE",
        SPA_AUDIO_FORMAT_U16_BE => "U16_BE",
        SPA_AUDIO_FORMAT_S24_32_LE => "S24_32_LE",
        SPA_AUDIO_FORMAT_S24_32_BE => "S24_32_BE",
        SPA_AUDIO_FORMAT_U24_32_LE => "U24_32_LE",
        SPA_AUDIO_FORMAT_U24_32_BE => "U24_32_BE",
        SPA_AUDIO_FORMAT_S32_LE => "S32_LE",
        SPA_AUDIO_FORMAT_S32_BE => "S32_BE",
        SPA_AUDIO_FORMAT_U32_LE => "U32_LE",
        SPA_AUDIO_FORMAT_U32_BE => "U32_BE",
        SPA_AUDIO_FORMAT_S24_LE => "S24_LE",
        SPA_AUDIO_FORMAT_S24_BE => "S24_BE",
        SPA_AUDIO_FORMAT_U24_LE => "U24_LE",
        SPA_AUDIO_FORMAT_U24_BE => "U24_BE",
        SPA_AUDIO_FORMAT_S20_LE => "S20_LE",
        SPA_AUDIO_FORMAT_S20_BE => "S20_BE",
        SPA_AUDIO_FORMAT_U20_LE => "U20_LE",
        SPA_AUDIO_FORMAT_U20_BE => "U20_BE",
        SPA_AUDIO_FORMAT_S18_LE => "S18_LE",
        SPA_AUDIO_FORMAT_S18_BE => "S18_BE",
        SPA_AUDIO_FORMAT_U18_LE => "U18_LE",
        SPA_AUDIO_FORMAT_U18_BE => "U18_BE",
        SPA_AUDIO_FORMAT_F32_LE => "F32_LE",
        SPA_AUDIO_FORMAT_F32_BE => "F32_BE",
        SPA_AUDIO_FORMAT_F64_LE => "F64_LE",
        SPA_AUDIO_FORMAT_F64_BE => "F64_BE",
        SPA_AUDIO_FORMAT_ULAW => "ULAW",
        SPA_AUDIO_FORMAT_ALAW => "ALAW",
        SPA_AUDIO_FORMAT_U8P => "U8P",
        SPA_AUDIO_FORMAT_S16P => "S16P",
        SPA_AUDIO_FORMAT_S24_32P => "S24_32P",
        SPA_AUDIO_FORMAT_S32P => "S32P",
        SPA_AUDIO_FORMAT_S24P => "S24P",
        SPA_AUDIO_FORMAT_F32P => "F32P",
        SPA_AUDIO_FORMAT_F64P => "F64P",
        SPA_AUDIO_FORMAT_S8P => "S8P",
        _ => "UNKNOWN"

    }
}

pub fn get_channel_name(channel: u32) -> &'static str {
    let c_str = unsafe {
        let c_buf =
            spa_debug_type_find_short_name(spa_type_audio_channel, channel);
        if c_buf.is_null() {
            return "Unknown";
        }
        std::ffi::CStr::from_ptr(c_buf)
    };
    c_str.to_str().unwrap()
}

pub fn get_channel_name_for_position(index: u32, format: Option<spa_audio_info_raw>) -> String {
    match format {
        Some(f) => get_channel_name(f.position[index as usize]).to_string(),
        None => index.to_string()
    }
}