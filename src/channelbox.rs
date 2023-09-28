// SPDX-License-Identifier: GPL-3.0-or-later

use crate::pwchannelobject::PwChannelObject;

mod imp {

    use super::*;
    use std::cell::RefCell;
    use gtk::{prelude::*, subclass::prelude::*};
    use glib::Properties;
    
    #[derive(Debug, Default, gtk::CompositeTemplate, Properties)]
    #[template(resource = "/com/saivert/pwvucontrol/gtk/channelbox.ui")]
    #[properties(wrapper_type = super::PwChannelBox)]
    pub struct PwChannelBox {
        #[property(get, set, construct_only)]
        row_data: RefCell<Option<PwChannelObject>>,

        // Template widgets
        #[template_child]
        pub label: TemplateChild<gtk::Label>,
        #[template_child]
        pub scale: TemplateChild<gtk::Scale>,

    }

    #[glib::object_subclass]
    impl ObjectSubclass for PwChannelBox {
        const NAME: &'static str = "PwChannelBox";
        type Type = super::PwChannelBox;
        type ParentType = gtk::ListBoxRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for PwChannelBox {

        fn constructed(&self) {
            fn linear_to_cubic(_binding: &glib::Binding, i: f32) -> Option<f64> {
                Some(i.cbrt() as f64)
            }

            fn cubic_to_linear(_binding: &glib::Binding, i: f64) -> Option<f32> {
                Some((i * i * i) as f32)
            }


            self.parent_constructed();

            let item = self.row_data.borrow();
            let item = item.as_ref().cloned().unwrap();

            item.bind_property("volume", &self.scale.adjustment(), "value")
                .sync_create()
                .bidirectional()
                .transform_to(linear_to_cubic)
                .transform_from(cubic_to_linear)
                .build();

            item.bind_property("name", &self.label.get(), "label")
                .sync_create()
                .build();
        }


    }
    impl WidgetImpl for PwChannelBox {}
    impl ListBoxRowImpl for PwChannelBox {}

    #[gtk::template_callbacks]
    impl PwChannelBox {}
}

glib::wrapper! {
    pub struct PwChannelBox(ObjectSubclass<imp::PwChannelBox>)
        @extends gtk::Widget, gtk::ListBoxRow,
        @implements gtk::Actionable;
}

impl PwChannelBox {
    pub(crate) fn new(channelobj: &PwChannelObject) -> Self {
        glib::Object::builder()
            .property("row-data", channelobj)
            .build()
    }
}
