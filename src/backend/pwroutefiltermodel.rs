// SPDX-License-Identifier: GPL-3.0-or-later

use glib::{closure_local, subclass::prelude::*, Properties};
use gtk::{gio, prelude::*, subclass::prelude::*};
use std::cell::{Cell, RefCell};
use im_rc::Vector;
use super::{ParamAvailability, PwRouteObject, RouteDirection};

mod imp {
    use super::*;

    #[derive(Debug, Properties, Default)]
    #[properties(wrapper_type = super::PwRouteFilterModel)]
    pub struct PwRouteFilterModel {
        /// Contains the items that matches the filter predicate.
        pub(super) filtered_model: RefCell<Vector<PwRouteObject>>,

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
    impl ObjectImpl for PwRouteFilterModel {
    }

    impl ListModelImpl for PwRouteFilterModel {
        fn item_type(&self) -> glib::Type {
            PwRouteObject::static_type()
        }
        fn n_items(&self) -> u32 {
            self.filtered_model.borrow().len() as u32
        }
        fn item(&self, position: u32) -> Option<glib::Object> {
            self.filtered_model
                .borrow()
                .get(position as usize)
                .map(|o| o.clone().upcast::<glib::Object>())
        }
    }

    impl PwRouteFilterModel {
        pub fn set_model(&self, new_model: Option<&gio::ListModel>) {
            let removed = self.filtered_model.borrow().len() as u32;

            if let Some(new_model) = new_model {

                assert!(self.item_type().is_a(new_model.item_type()));

                let widget = self.obj();
                let handler = closure_local!(@watch widget => move |listmodel: &gio::ListModel, _position: u32, _removed: u32, _added: u32| {
                    let u: Vector<PwRouteObject> = listmodel.iter::<PwRouteObject>()
                        .map_while(Result::ok)
                        .filter(|routeobject| {
                            routeobject.direction() == widget.direction() && routeobject.availability() == ParamAvailability::Yes
                        })
                        .collect();

                    let removed = widget.imp().filtered_model.borrow().len() as u32;
                    let added = {
                        let mut filtered_model = widget.imp().filtered_model.borrow_mut();
                        filtered_model.clear();
                        filtered_model.append(u);
                        filtered_model.len() as u32
                    };
                    widget.items_changed(0, removed, added);
                });
                handler.invoke::<()>(&[&new_model, &0u32, &0u32, &0u32]);
                new_model.connect_closure("items-changed", true, handler);

                self.model.replace(Some(new_model.clone().upcast()));
            } else {
                self.filtered_model.borrow_mut().clear();
                self.obj().items_changed(0, removed, 0);
            }
        }
    }
}

glib::wrapper! {
    pub struct PwRouteFilterModel(ObjectSubclass<imp::PwRouteFilterModel>) @implements gio::ListModel;
}

impl PwRouteFilterModel {
    pub(crate) fn new(direction: RouteDirection, model: Option<&impl glib::IsA<gio::ListModel>>) -> Self
    {
        glib::Object::builder()
        .property("model", model)
        .property("direction", direction)
        .build()
    }
}
