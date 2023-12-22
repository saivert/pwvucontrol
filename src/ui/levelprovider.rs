// SPDX-License-Identifier: GPL-3.0-or-later

use std::{fmt::Debug, time::Duration};

use glib::{self, clone, ControlFlow};
use pipewire::{properties, spa, stream::*, Context, Loop};
use std::os::fd::AsRawFd;

use crate::ui::volumebox::PwVolumeBox;

pub(crate) struct LevelbarProvider {
    _loop: Loop,
    _context: Context<pipewire::Loop>,
    stream: Option<Stream>,
    _listener: StreamListener<f32>,
}

impl Debug for LevelbarProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("LevelbarProvider")
    }
}

impl LevelbarProvider {
    pub fn new(volumebox: &PwVolumeBox, id: u32) -> Result<Self, anyhow::Error> {
        let loop_ = Loop::new()?;
        let context = Context::new(&loop_)?;
        let core = context.connect(None)?;

        let fd = loop_.fd();

        glib::source::unix_fd_add_local(fd.as_raw_fd(), glib::IOCondition::all(), {
            let loop_ = loop_.clone();
            move |_, _| {
                loop_.iterate(Duration::ZERO);

                ControlFlow::Continue
            }
        });

        let props = properties! {
            "node.rate" => "1/25",
            "node.latency" => "1/25",
            "node.name" => "pwvucontrol-peak-detect",
            "media.name" => "Peak detect",
            "resample.peaks" => "true",
            "stream.monitor" => "true"
        };

        let stream: Stream = Stream::new(&core, "peakdetect", props)?;

        let listener = stream.add_local_listener::<f32>()
        .process(clone!(@weak volumebox => @default-panic, move |stream, last_peak| {
            match stream.dequeue_buffer() {
                None => println!("No buffer received"),
                Some(mut buffer) => {
                    let datas = buffer.datas_mut();

                    if let Some(d) = datas[0].data() {
                        let chan = &d[0..std::mem::size_of::<f32>()];
                        let mut max = f32::from_le_bytes(chan.try_into().unwrap()).clamp(0.0, 1.0);

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

        let mut buffer: Vec<u8> = Vec::new();
        let fmtpod = create_audio_format_pod(&mut buffer);

        stream.connect(
            pipewire::spa::Direction::Input,
            Some(id),
            StreamFlags::AUTOCONNECT
                | StreamFlags::MAP_BUFFERS
                | StreamFlags::RT_PROCESS
                | StreamFlags::DONT_RECONNECT,
            &mut [fmtpod],
        )?;

        Ok(Self {
            _loop: loop_,
            _context: context,
            stream: Some(stream),
            _listener: listener,
        })
    }
}

impl Drop for LevelbarProvider {
    fn drop(&mut self) {
        if let Some(stream) = self.stream.take() {
            stream.disconnect().unwrap();
        }
    }
}

fn create_audio_format_pod(buffer: &mut Vec<u8>) -> &spa::pod::Pod {
    let mut audio_info = spa::param::audio::AudioInfoRaw::new();
    audio_info.set_format(spa::param::audio::AudioFormat::F32LE);
    audio_info.set_rate(25);
    audio_info.set_channels(1);
    audio_info.set_position([spa::sys::SPA_AUDIO_CHANNEL_MONO; 64]);

    let values = spa::pod::serialize::PodSerializer::serialize(
        std::io::Cursor::new(buffer),
        &spa::pod::Value::Object(pipewire::spa::pod::Object {
            type_: spa::sys::SPA_TYPE_OBJECT_Format,
            id: spa::sys::SPA_PARAM_EnumFormat,
            properties: audio_info.into(),
        }),
    )
    .unwrap()
    .0
    .into_inner();

    spa::pod::Pod::from_bytes(values).unwrap()
}
