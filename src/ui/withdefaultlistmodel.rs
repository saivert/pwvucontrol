// SPDX-License-Identifier: GPL-3.0-or-later

use crate::backend::PwNodeModel;
use glib::{Properties, closure_local};
use gtk::{gio, glib, prelude::*, subclass::prelude::*};
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::WithDefaultListModel)]
    pub struct WithDefaultListModel {
        pub(super) string_list: RefCell<Option<gtk::StringList>>,
        pub(super) flatten_list_model: RefCell<Option<gtk::FlattenListModel>>,

        #[property(get, set = Self::set_model)]
        pub(super) model: RefCell<Option<PwNodeModel>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WithDefaultListModel {
        const NAME: &'static str = "WithDefaultListModel";
        type Type = super::WithDefaultListModel;
        type Interfaces = (gio::ListModel,);
    }

    #[glib::derived_properties]
    impl ObjectImpl for WithDefaultListModel {
        fn constructed(&self) {
            self.parent_constructed();

            self.string_list
                .replace(Some(gtk::StringList::new(&["Default"])));
        }
    }

    impl ListModelImpl for WithDefaultListModel {
        fn item_type(&self) -> glib::Type {
            glib::Object::static_type()
        }
        fn n_items(&self) -> u32 {
            let model = self.flatten_list_model.borrow();
            if let Some(model) = model.as_ref() {
                model.n_items()
            } else {
                0
            }
        }
        fn item(&self, position: u32) -> Option<glib::Object> {
            let model = self.flatten_list_model.borrow();
            if let Some(model) = model.as_ref() {
                model.item(position)
            } else {
                None
            }
        }
    }

    impl WithDefaultListModel {
        pub fn set_model(&self, new_model: Option<&PwNodeModel>) {
            let removed = self.n_items();

            let string_list = self.string_list.borrow();
            let string_list = string_list.as_ref().unwrap();

            let composite_store = gio::ListStore::new::<gio::ListModel>();
            composite_store.append(string_list);

            if let Some(new_model) = new_model {
                composite_store.append(new_model);
            }

            let flattened_model = gtk::FlattenListModel::new(Some(composite_store));

            let widget = self.obj();
            let handler = closure_local!(@watch widget => move |_listmodel: &gio::ListModel, position: u32, removed: u32, added: u32| {
                widget.items_changed(position, removed, added);
            });
            flattened_model.connect_closure("items-changed", true, handler);

            let added = flattened_model.n_items();
            self.flatten_list_model.replace(Some(flattened_model));

            self.obj().items_changed(0, removed, added);
        }
    }
}

glib::wrapper! {
    pub struct WithDefaultListModel(ObjectSubclass<imp::WithDefaultListModel>) @implements gio::ListModel;
}

impl WithDefaultListModel {
    pub(crate) fn new(model: Option<&PwNodeModel>) -> Self {
        glib::Object::builder().property("model", model).build()
    }

    pub(crate) fn set_default_text(&self, text: &str) {
        let imp = self.imp();
        let string_list = imp.string_list.borrow();
        let string_list = string_list.as_ref().unwrap();
        string_list.splice(0, 1, &[text]);
    }
}

impl Default for WithDefaultListModel {
    fn default() -> Self {
        Self::new(None)
    }
}
