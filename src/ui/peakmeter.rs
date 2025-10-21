// SPDX-License-Identifier: GPL-3.0-or-later

use gtk::{self, prelude::*, subclass::prelude::*};
use std::cell::Cell;

mod imp {
    use gtk::{graphene, gsk};

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
            let num_blocks: u32 = (self.obj().width() / 20) as u32;
            let green_limit: u32 = (0.6 * num_blocks as f32) as u32;
            let yellow_limit: u32 = (0.9 * num_blocks as f32) as u32;

            let color_green = hex_to_rgb(0x33d17a);
            let color_yellow = hex_to_rgb(0xf6d32d);
            let color_red = hex_to_rgb(0xe01b24);

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
                let discrete_level = (level * num_blocks as f32).floor() as u32;
                let mut block_width = width / num_blocks;
                let extra_space = width - block_width * num_blocks;
                if extra_space > 0 {
                    block_width += 1;
                }
                let mut block_area_width = block_width;
                let mut block_area_x = 0;

                for i in 0..discrete_level {
                    if extra_space > 0 && i == extra_space {
                        block_area_width -= 1;
                    }

                    let color = if i < green_limit {
                        color_green
                    } else if i < yellow_limit {
                        color_yellow
                    } else {
                        color_red
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
                gtk::Orientation::Vertical => (10, 10, -1, -1),
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

fn hex_to_rgb(hex: u32) -> gtk::gdk::RGBA {
    let r = ((hex >> 16) & 0xFF) as f32 / 255.0;
    let g = ((hex >> 8) & 0xFF) as f32 / 255.0;
    let b = (hex & 0xFF) as f32 / 255.0;
    gtk::gdk::RGBA::new(r, g, b, 1.0)
}
