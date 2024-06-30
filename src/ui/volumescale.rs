// SPDX-License-Identifier: GPL-3.0-or-later

use crate::ui::PwvucontrolWindow;
use gettextrs::gettext;
use gtk::{prelude::*, subclass::prelude::*};
use std::cell::Cell;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/com/saivert/pwvucontrol/gtk/volumescale.ui")]
    #[properties(wrapper_type = super::PwVolumeScale)]
    pub struct PwVolumeScale {
        #[template_child]
        pub scale: TemplateChild<gtk::Scale>,
        #[template_child]
        pub value: TemplateChild<gtk::Label>,

        #[property(get, set = Self::set_volume)]
        pub volume: Cell<f32>,

        #[property(get, set = Self::set_use_overamplification)]
        pub use_overamplification: Cell<bool>,

        #[property(get, set = Self::set_overamplification)]
        pub overamplification: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PwVolumeScale {
        const NAME: &'static str = "PwVolumeScale";
        type Type = super::PwVolumeScale;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for PwVolumeScale {
        fn constructed(&self) {
            self.parent_constructed();

            fn linear_to_cubic(_binding: &glib::Binding, i: f32) -> Option<f64> {
                Some(i.cbrt() as f64)
            }

            fn cubic_to_linear(_binding: &glib::Binding, i: f64) -> Option<f32> {
                Some((i.powi(3)) as f32)
            }

            self.obj()
                .bind_property("volume", &self.scale.adjustment(), "value")
                .sync_create()
                .bidirectional()
                .transform_to(linear_to_cubic)
                .transform_from(cubic_to_linear)
                .build();

            let window = PwvucontrolWindow::default();
            window
                .imp()
                .settings
                .bind("enable-overamplification", self.obj().as_ref(), "overamplification")
                .get_only()
                .build();
        }
    }
    impl WidgetImpl for PwVolumeScale {}

    impl PwVolumeScale {
        fn set_volume(&self, volume: f32) {
            if self.volume.get() == volume {
                return;
            }
            self.volume.set(volume);

            let cubic_volume = volume.cbrt();

            let value_string = format!(
                "{:>16}",
                format!("{:.0}% ({:.2} dB)", cubic_volume * 100.0, (cubic_volume.powi(3)).log10() * 20.0)
            );

            self.value.set_label(&value_string);
        }

        fn set_use_overamplification(&self, value: bool) {
            if self.use_overamplification.get() == value {
                return;
            }
            self.use_overamplification.set(value);

            self.update_ui();
        }

        fn set_overamplification(&self, value: bool) {
            if self.overamplification.get() == value {
                return;
            }
            self.overamplification.set(value);

            self.update_ui();
        }

        fn update_ui(&self) {
            let overamplification = self.use_overamplification.get() && self.overamplification.get();

            let volume_scale = self.scale.get();
            volume_scale.clear_marks();
            volume_scale.add_mark(0.0, gtk::PositionType::Bottom, Some(&gettext("Silence")));
            volume_scale.add_mark(1.0, gtk::PositionType::Bottom, Some(&gettext("100%")));

            if overamplification {
                volume_scale.add_mark(1.525, gtk::PositionType::Bottom, Some(&gettext("150%")));
                volume_scale.set_range(0.0, 1.525);
            } else {
                volume_scale.set_range(0.0, 1.0);
            }
        }
    }
}

glib::wrapper! {
    pub struct PwVolumeScale(ObjectSubclass<imp::PwVolumeScale>)
        @extends gtk::Widget,
        @implements gtk::Actionable;
}

impl PwVolumeScale {
    pub(crate) fn new() -> Self {
        glib::Object::builder().build()
    }
}

impl Default for PwVolumeScale {
    fn default() -> Self {
        Self::new()
    }
}
