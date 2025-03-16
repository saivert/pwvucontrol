// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::{Cell, RefCell};

use gtk::{
    glib::{self, Properties},
    prelude::*,
    subclass::prelude::*,
};

use super::ParamAvailability;
use super::RouteDirection;

mod imp {
    use super::*;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::PwRouteObject)]
    pub struct PwRouteObject {
        #[property(get, set)]
        index: Cell<u32>,
        #[property(get, set)]
        description: RefCell<String>,
        #[property(get, set, builder(ParamAvailability::Unknown))]
        availability: Cell<ParamAvailability>,
        #[property(get, set, builder(RouteDirection::Unknown))]
        direction: Cell<RouteDirection>,

        pub(super) profiles: RefCell<Vec<u32>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PwRouteObject {
        const NAME: &'static str = "PwRouteObject";
        type Type = super::PwRouteObject;
    }

    #[glib::derived_properties]
    impl ObjectImpl for PwRouteObject {}

    impl PwRouteObject {}
}

glib::wrapper! {
    pub struct PwRouteObject(ObjectSubclass<imp::PwRouteObject>);
}

impl PwRouteObject {
    pub(crate) fn new(index: u32, description: &str, availability: ParamAvailability, direction: RouteDirection, profiles: &[u32]) -> Self {
        let new: PwRouteObject = glib::Object::builder()
            .property("index", index)
            .property("description", format!("{description} ({index})"))
            .property("availability", availability)
            .property("direction", direction)
            .build();

        new.set_profiles(profiles);

        new
    }

    pub(crate) fn get_profiles(&self) -> Vec<u32> {
        self.imp().profiles.borrow().clone()
    }

    pub(crate) fn set_profiles(&self, list: &[u32]) {
        self.imp().profiles.replace(list.to_vec());
    }
}
