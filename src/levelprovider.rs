use std::{time::Duration, fmt::Debug};

use pipewire::{prelude::*, properties, stream::{*, self}, Core, Context, Loop};
use glib::{self, Continue, clone};
use std::os::fd::AsRawFd;

use crate::volumebox::PwVolumeBox;

pub(crate) struct LevelbarProvider {
    loop_: Loop,
    context: Context<pipewire::Loop>,
    core: Core,
    stream: Stream<f32>,
    listener: StreamListener<f32>,

}

impl Debug for LevelbarProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("LevelbarProvider")
    }
}

impl LevelbarProvider {
    pub fn new(volumebox: &PwVolumeBox) -> Result<Self, anyhow::Error> {
        let loop_ = Loop::new()?;
        let context = Context::new(&loop_)?;
        let core = context.connect(None)?;

        glib::source::unix_fd_add_local(loop_.fd().as_raw_fd(), glib::IOCondition::all(), {
            let loop_ = loop_.clone();
            move |_, _| {
                loop_.iterate(Duration::ZERO);

                Continue(true)
            }
        });

        let props = properties! {
            "media.type" => "Audio",
            "media.category" => "Capture",
            "media.role" => "Music",
            "node.rate" => "1/25",
            "node.latency" => "1/25",
            "node.name" => "pwvucontrol-peak-detect",
            "media.name" => "Peak detect",
            "resample.peaks" => "true",
            "stream.monitor" => "true"
        };

        let mut stream: Stream<f32> = Stream::new(&core, "peakdetect", props)?;

        let listener = stream.add_local_listener()
        .process(clone!(@weak volumebox => @default-panic, move |stream, last_peak| {
            match stream.dequeue_buffer() {
                None => println!("No buffer received"),
                Some(mut buffer) => {
                    let datas = buffer.datas_mut();

                    if let Some(d) = datas[0].data() {
                        let df: &mut [f32] = bytemuck::cast_slice_mut(d);
                        let mut max = df[0].clamp(0.0, 1.0);
                        const DECAY_STEP: f32 = 0.4;
                        if *last_peak >= DECAY_STEP && max < *last_peak - DECAY_STEP {
                            max = *last_peak - DECAY_STEP;
                        }
                        *last_peak = max;

                        volumebox.set_level(max);
                    }
                }
            };
        }))
        .register()?;

        Ok(Self {
            loop_,
            context,
            core,
            stream,
            listener,
        })
    }

    pub fn connect(&self, id: u32) -> Result<(), anyhow::Error> {
        let mut buffer = [0;1024];
        let fmtpod = create_audio_format_pod(&mut buffer);

        self.stream.connect(
            pipewire::spa::Direction::Input,
            Some(id),
            stream::StreamFlags::AUTOCONNECT
            | stream::StreamFlags::MAP_BUFFERS
            | stream::StreamFlags::RT_PROCESS,
            &mut [fmtpod])?;

        Ok(())
    }

}

impl Drop for LevelbarProvider {
    fn drop(&mut self) {

    }
}

fn create_audio_format_pod(buffer: &mut [u8]) -> *mut pipewire::spa::sys::spa_pod {
    unsafe {

        let mut b: pipewire::spa::sys::spa_pod_builder = std::mem::zeroed();
        b.data = buffer.as_mut_ptr() as *mut std::ffi::c_void;
        b.size = buffer.len() as u32;
        let mut audioinfo = pipewire::spa::sys::spa_audio_info_raw {
            format: pipewire::spa::sys::SPA_AUDIO_FORMAT_F32_LE,
            flags: 0,
            rate: 25,
            channels: 1,
            position: [pipewire::spa::sys::SPA_AUDIO_CHANNEL_UNKNOWN; 64],
        };

        audioinfo.position[0] = pipewire::spa::sys::SPA_AUDIO_CHANNEL_MONO;

        pipewire::spa::sys::spa_format_audio_raw_build(&mut b as *mut pipewire::spa::sys::spa_pod_builder,
            pipewire::spa::sys::SPA_PARAM_EnumFormat,
            &mut audioinfo as *mut pipewire::spa::sys::spa_audio_info_raw)
    }
}

