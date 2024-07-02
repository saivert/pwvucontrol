// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{
    backend::{PwDeviceObject, PwProfileObject},
    macros::*,
    ui::PwProfileRow,
};
use glib::clone;
use glib::closure_local;
use gtk::{self, prelude::*, subclass::prelude::*};
use std::cell::{Cell, RefCell};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[properties(wrapper_type = super::PwProfileDropDown)]
    #[template(resource = "/com/saivert/pwvucontrol/gtk/profile-dropdown.ui")]
    pub struct PwProfileDropDown {
        #[property(get, set = Self::set_deviceobject, nullable)]
        pub(super) deviceobject: RefCell<Option<PwDeviceObject>>,

        #[template_child]
        pub profile_dropdown: TemplateChild<gtk::DropDown>,

        pub(super) block_signal: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PwProfileDropDown {
        const NAME: &'static str = "PwProfileDropDown";
        type Type = super::PwProfileDropDown;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl PwProfileDropDown {
        pub fn update_selected(&self) {
            let deviceobject = self.deviceobject.borrow();
            let deviceobject = deviceobject.as_ref().unwrap();

            if self.profile_dropdown.model().is_none() {
                return;
            }

            pwvucontrol_info!("update_selected with index {}", deviceobject.profile_index());
            self.obj().set_selected_no_send(deviceobject.profile_index());
        }

        pub fn set_deviceobject(&self, new_deviceobject: Option<&PwDeviceObject>) {
            self.deviceobject.replace(new_deviceobject.cloned());

            if let Some(deviceobject) = new_deviceobject {
                self.block_signal.set(true);
                self.profile_dropdown.set_model(Some(&deviceobject.profilemodel()));
                self.update_selected();
                self.block_signal.set(false);

                deviceobject.connect_local(
                    "pre-update-profile",
                    false,
                    clone!(@weak self as widget => @default-return None, move |_| {
                        widget.block_signal.set(true);

                        None
                    }),
                );

                deviceobject.connect_local(
                    "post-update-profile",
                    false,
                    clone!(@weak self as widget => @default-return None, move |_| {
                        widget.block_signal.set(false);
                        pwvucontrol_info!("About to call widget.update_selected() inside post-update-route handler");
                        // widget.update_selected();

                        None
                    }),
                );

                deviceobject.connect_profile_index_notify(clone!(@weak self as widget => move |_| widget.update_selected()));
            } else {
                self.profile_dropdown.set_model(gtk::gio::ListModel::NONE);
            }
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for PwProfileDropDown {
        fn dispose(&self) {
            self.dispose_template();
        }

        fn constructed(&self) {
            self.parent_constructed();

            fn setup_handler(item: &glib::Object, list: bool) {
                let item: &gtk::ListItem = item.downcast_ref().expect("ListItem");
                let profilerow = PwProfileRow::new();

                profilerow.setup::<PwProfileObject>(item, list);
                item.set_child(Some(&profilerow));
            }

            fn bind_handler(item: &glib::Object, dropdown: &gtk::DropDown) {
                let item: &gtk::ListItem = item.downcast_ref().expect("ListItem");
                let profilerow = item.child().and_downcast::<PwProfileRow>().expect("PwProfileRow child");

                let signal = dropdown.connect_selected_item_notify(clone!(@weak item => move |dropdown| {
                    let profilerow = item
                        .child()
                        .and_downcast::<PwProfileRow>()
                        .expect("PwProfileRow child");
                    profilerow.set_selected(dropdown.selected_item() == item.item());
                }));
                profilerow.set_handlerid(Some(signal));
            }

            fn unbind_handler(item: &glib::Object) {
                let item: &gtk::ListItem = item.downcast_ref().expect("ListItem");
                let profilerow = item
                    .child()
                    .and_downcast::<PwProfileRow>()
                    .expect("The child has to be a `PwProfileRow`.");
                profilerow.set_handlerid(None);
            }

            let dropdown = self.profile_dropdown.get();

            let factory = gtk::SignalListItemFactory::new();
            factory.connect_setup(|_, item| setup_handler(item, false));

            let list_factory = gtk::SignalListItemFactory::new();
            list_factory.connect_setup(|_, item| setup_handler(item, true));
            list_factory.connect_bind(clone!(@weak dropdown => move |_, item| bind_handler(item, &dropdown)));
            list_factory.connect_unbind(|_, item| unbind_handler(item));

            let expression = gtk::PropertyExpression::new(PwProfileObject::static_type(), gtk::Expression::NONE, "description");
            self.profile_dropdown.set_expression(Some(expression));
            self.profile_dropdown.set_enable_search(true);
            self.profile_dropdown.set_search_match_mode(gtk::StringFilterMatchMode::Substring);

            self.profile_dropdown.set_factory(Some(&factory));
            self.profile_dropdown.set_list_factory(Some(&list_factory));

            let widget = self.obj();
            let selected_handler = closure_local!(
                @watch widget => move |dropdown: &gtk::DropDown, _pspec: &glib::ParamSpec| {
                pwvucontrol_info!("Inside selected handler");
                if widget.imp().block_signal.get() {
                    pwvucontrol_info!("Early return from selected handler due to being blocked");
                    return;
                }

                if let Some(deviceobject) = widget.deviceobject() {

                    if let Some(item) = dropdown.selected_item().and_downcast::<PwProfileObject>() {
                        pwvucontrol_critical!("Setting profile to dropdown index = {} profile index = {}", dropdown.selected(), item.index());
                        deviceobject.set_profile(item.index() as i32);
                    }

                }
            });
            self.profile_dropdown.connect_closure("notify::selected", true, selected_handler);
        }
    }

    impl WidgetImpl for PwProfileDropDown {}
}

glib::wrapper! {
    pub struct PwProfileDropDown(ObjectSubclass<imp::PwProfileDropDown>) @extends gtk::Widget;
}

impl PwProfileDropDown {
    // pub fn new(nodeobj: Option<&PwNodeObject>) -> Self {
    //     glib::Object::builder()
    //     .property("nodeobj", nodeobj)
    //     .build()
    // }

    pub fn set_selected_no_send(&self, position: u32) {
        let imp = self.imp();

        imp.block_signal.set(true);
        imp.profile_dropdown.set_selected(position);
        imp.block_signal.set(false);
    }
}

impl Default for PwProfileDropDown {
    fn default() -> Self {
        //Self::new(None)
        glib::Object::new()
    }
}
