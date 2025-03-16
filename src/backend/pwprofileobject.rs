// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::{Cell, RefCell};

use super::ParamAvailability;
use gtk::{
    glib::{self, Properties},
    prelude::*,
    subclass::prelude::*,
};

mod imp {

    use super::*;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::PwProfileObject)]
    pub struct PwProfileObject {
        #[property(get, set)]
        index: Cell<u32>,
        #[property(get, set)]
        description: RefCell<String>,
        #[property(get, set, builder(ParamAvailability::Unknown))]
        availability: Cell<ParamAvailability>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PwProfileObject {
        const NAME: &'static str = "PwProfileObject";
        type Type = super::PwProfileObject;
    }

    #[glib::derived_properties]
    impl ObjectImpl for PwProfileObject {}

    impl PwProfileObject {}
}

glib::wrapper! {
    pub struct PwProfileObject(ObjectSubclass<imp::PwProfileObject>);
}

impl PwProfileObject {
    pub(crate) fn new(index: u32, description: &str, availability: u32) -> Self {
        glib::Object::builder()
            .property("index", index)
            .property("description", description)
            .property("availability", ParamAvailability::from(availability))
            .build()
    }
}
