// SPDX-License-Identifier: GPL-3.0-or-later

use super::{ParamAvailability, PwRouteObject, RouteDirection};
use glib::{closure_local, Properties};
use gtk::{gio, prelude::*, subclass::prelude::*};
use imbl::OrdSet;
use std::cell::{Cell, RefCell};

mod imp {
    use super::*;

    #[derive(Debug, Properties, Default)]
    #[properties(wrapper_type = super::PwRouteFilterModel)]
    pub struct PwRouteFilterModel {
        /// Contains the items that matches the filter predicate.
        pub(super) hashset: RefCell<OrdSet<u32>>,

        #[property(get, set, construct_only, builder(RouteDirection::Unknown))]
        pub(super) direction: Cell<RouteDirection>,

        /// The model we are filtering.
        #[property(get, set = Self::set_model, nullable)]
        pub(super) model: RefCell<Option<gio::ListModel>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PwRouteFilterModel {
        const NAME: &'static str = "PwRouteFilterModel";
        type Type = super::PwRouteFilterModel;
        type Interfaces = (gio::ListModel,);
    }

    #[glib::derived_properties]
    impl ObjectImpl for PwRouteFilterModel {}

    impl ListModelImpl for PwRouteFilterModel {
        fn item_type(&self) -> glib::Type {
            PwRouteObject::static_type()
        }
        fn n_items(&self) -> u32 {
            self.hashset.borrow().len() as u32
        }
        fn item(&self, position: u32) -> Option<glib::Object> {
            if let Some(pos) = self.hashset.borrow().iter().nth(position as usize) {
                if let Some(model) = self.model.borrow().as_ref() {
                    return model.item(*pos);
                };
            };

            None
        }
    }

    impl PwRouteFilterModel {
        pub fn set_model(&self, new_model: Option<&gio::ListModel>) {
            if let Some(new_model) = new_model {
                assert!(self.item_type().is_a(new_model.item_type()));

                let widget = self.obj();
                let handler = closure_local!(@watch widget => move |listmodel: &gio::ListModel, position: u32, _removed: u32, _added: u32| {
                    let removed = widget.imp().hashset.borrow().len() as u32;

                    let mut hashset = OrdSet::new();

                    for (a, routeobject) in listmodel.iter::<PwRouteObject>()
                        .skip(position as usize)
                        .map_while(Result::ok)
                        .enumerate() {
                        if routeobject.direction() == widget.direction() && routeobject.availability() == ParamAvailability::Yes || routeobject.availability() == ParamAvailability::Unknown {
                            hashset.insert(a as u32);
                        }
                    }

                    let added = hashset.len() as u32;
                    widget.imp().hashset.replace(hashset);
                    widget.items_changed(0, removed, added);
                });
                handler.invoke::<()>(&[&new_model, &0u32, &0u32, &0u32]);
                new_model.connect_closure("items-changed", true, handler);

                self.model.replace(Some(new_model.clone().upcast()));
            } else {
                self.hashset.borrow_mut().clear();
                let removed = self.hashset.borrow().len() as u32;
                self.obj().items_changed(0, removed, 0);
            }
        }
    }
}

glib::wrapper! {
    pub struct PwRouteFilterModel(ObjectSubclass<imp::PwRouteFilterModel>) @implements gio::ListModel;
}

impl PwRouteFilterModel {
    pub(crate) fn new(direction: RouteDirection, model: Option<&impl IsA<gio::ListModel>>) -> Self {
        glib::Object::builder().property("model", model).property("direction", direction).build()
    }
}
