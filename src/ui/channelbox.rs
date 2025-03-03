// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{backend::PwChannelObject, ui::PwVolumeScale};
use std::cell::RefCell;
use gtk::{prelude::*, subclass::prelude::*};

mod imp {
    use super::*;
    
    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/com/saivert/pwvucontrol/gtk/channelbox.ui")]
    #[properties(wrapper_type = super::PwChannelBox)]
    pub struct PwChannelBox {
        #[property(get, set, construct_only)]
        channel_object: RefCell<Option<PwChannelObject>>,

        // Template widgets
        #[template_child]
        pub label: TemplateChild<gtk::Label>,
        #[template_child]
        pub scale: TemplateChild<PwVolumeScale>,

    }

    #[glib::object_subclass]
    impl ObjectSubclass for PwChannelBox {
        const NAME: &'static str = "PwChannelBox";
        type Type = super::PwChannelBox;
        type ParentType = gtk::ListBoxRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for PwChannelBox {

        fn constructed(&self) {
            self.parent_constructed();

            let item = self.channel_object.borrow();
            let item = item.as_ref().cloned().unwrap();

            item.bind_property("volume", &self.scale.get(), "volume")
                .sync_create()
                .bidirectional()
                .build();

            item.bind_property("name", &self.label.get(), "label")
                .sync_create()
                .build();

        }


    }
    impl WidgetImpl for PwChannelBox {}
    impl ListBoxRowImpl for PwChannelBox {}

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
            .property("channel-object", channelobj)
            .build()
    }
}
