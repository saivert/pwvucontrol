// SPDX-License-Identifier: GPL-3.0-or-later

use gtk::{self, prelude::*, subclass::prelude::*};
use std::cell::Cell;

mod imp {
    use gtk::{graphene, gsk};

    use crate::macros::*;

    use super::*;

    #[derive(Debug, Default, glib::Properties)]
    #[properties(wrapper_type = super::PwPeakMeter)]
    pub struct PwPeakMeter {
        #[property(get, set = Self::set_level)]
        pub(super) level: Cell<f32>,

        #[property(get, set)]
        pub(super) use_led: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PwPeakMeter {
        const NAME: &'static str = "PwPeakMeter";
        type Type = super::PwPeakMeter;
        type ParentType = gtk::Widget;
    }

    impl PwPeakMeter {
        fn set_level(&self, level: f32) {
            self.level.set(level);
            self.obj().queue_draw();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for PwPeakMeter {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().add_css_class("vumeter");
        }
    }

    impl WidgetImpl for PwPeakMeter {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            const NUM_BLOCKS: u32 = 20;
            const GREEN_LIMIT: u32 = (0.6 * NUM_BLOCKS as f32) as u32;
            const YELLOW_LIMIT: u32 = (0.9 * NUM_BLOCKS as f32) as u32;

            let color_green = pwvucontrol_hex_to_rgba!(0x33 0xd1 0x7a);
            let color_yellow = pwvucontrol_hex_to_rgba!(0xf6 0xd3 0x2d);
            let color_red = pwvucontrol_hex_to_rgba!(0xe0 0x1b 0x24);

            let width = self.obj().width() as u32;
            let w = self.obj().width() as f32;
            let h = self.obj().height() as f32;

            let level = self.level.get() as f32;
            let bounding_box = graphene::Rect::new(0.0, 0.0, w, h);

            let rounded_rect = gsk::RoundedRect::from_rect(bounding_box, 5.0);

            snapshot.push_rounded_clip(&rounded_rect);

            if !self.use_led.get() {
                snapshot.append_color(&color_green, &graphene::Rect::new(0.0, 0.0, level * w, h));
            } else {
                let discrete_level = (level * NUM_BLOCKS as f32).floor() as u32;
                let mut block_width = width / NUM_BLOCKS;
                let extra_space = width - block_width * NUM_BLOCKS;
                if extra_space > 0 {
                    block_width += 1;
                }
                let mut block_area_width = block_width;
                let mut block_area_x = 0;

                for i in 0..discrete_level {
                    if extra_space > 0 && i == extra_space {
                        block_area_width -= 1;
                    }

                    let color = match i {
                        0..GREEN_LIMIT => color_green,
                        GREEN_LIMIT..YELLOW_LIMIT => color_yellow,
                        _ => color_red,
                    };
                    snapshot.append_color(&color, &graphene::Rect::new(block_area_x as f32, 0.0, block_area_width as f32 - 1.0, h));
                    block_area_x += block_area_width;
                }
            }

            snapshot.pop();
        }

        fn measure(&self, orientation: gtk::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            match orientation {
                gtk::Orientation::Horizontal => (10, 10, -1, -1),
                gtk::Orientation::Vertical => (10, 20, -1, -1),
                _ => panic!("Invalid orientation passed to measure"),
            }
        }
    }
}

glib::wrapper! {
    pub struct PwPeakMeter(ObjectSubclass<imp::PwPeakMeter>) @extends gtk::Widget;
}

impl PwPeakMeter {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }
}

impl Default for PwPeakMeter {
    fn default() -> Self {
        Self::new()
    }
}
