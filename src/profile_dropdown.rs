// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{pwdeviceobject::PwDeviceObject, pwprofileobject::{PwProfileObject, ProfileAvailability}};
use glib::closure_local;
use gtk::{self, prelude::*, subclass::prelude::*};
use glib::clone;
use wp::pw::ProxyExt;
use std::cell::{Cell, RefCell};
use wireplumber as wp;

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

        //pub(super) stringlist: RefCell<gtk::StringList>,
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
        // NOTE: Commented code is for when using a separate stringlist as model for the dropdown to decouple it from the
        // listmodel of the Device object as one of two strategies for breaking feedback loop.
        //
        // pub fn set_deviceobject(&self, new_deviceobject: Option<&PwDeviceObject>) {
        //     self.deviceobject.replace(new_deviceobject.cloned());

        //     if let Some(deviceobject) = new_deviceobject {
        //         self.update_profiles();

        //         deviceobject.connect_local("profiles-changed", false,
        //             clone!(@weak self as widget => @default-return None, move |_| widget.update_profiles())
        //         );

        //         deviceobject.connect_profile_index_notify(
        //             clone!(@weak self as widget => move |_| widget.update_selected()),
        //         );
        //     }
        // }

        // pub fn update_profiles(&self) -> Option<glib::Value> {
        //     wp::log::info!("update_profiles");
        //     self.block_signal.set(true);

        //     let deviceobject = self.deviceobject.borrow();
        //     let deviceobject = deviceobject.as_ref().unwrap();

        //     let mut strings = Vec::from_iter(
        //         deviceobject
        //             .get_profiles()
        //             .iter()
        //             .map(|x| (*x.0, x.1.to_owned())),
        //     );
        //     strings.sort();

        //     let a = Vec::from_iter(strings.iter().map(|x| x.1.as_str()));
        //     let new_stringlist = gtk::StringList::new(&a);

        //     self.profile_dropdown.set_model(Some(&new_stringlist));
        //     self.stringlist.replace(new_stringlist);

        //     self.profile_dropdown
        //         .set_selected(deviceobject.profile_index());

        //     self.block_signal.set(false);

        //     None
        // }

        pub fn update_selected(&self) {
            let deviceobject = self.deviceobject.borrow();
            let deviceobject = deviceobject.as_ref().unwrap();

            wp::log::info!("update_selected with index {}", deviceobject.profile_index());
            self.obj().set_selected_no_send(deviceobject.profile_index());
        }

        pub fn set_deviceobject(&self, new_deviceobject: Option<&PwDeviceObject>) {
            self.deviceobject.replace(new_deviceobject.cloned());

            if let Some(deviceobject) = new_deviceobject {
                self.block_signal.set(true);
                wp::log::info!("self.profile_dropdown.set_model({});", deviceobject.wpdevice().bound_id());
                self.profile_dropdown.set_model(Some(deviceobject));
                wp::log::info!("self.profile_dropdown.set_selected({});", deviceobject.profile_index());

                self.profile_dropdown.set_selected(deviceobject.profile_index());

                self.block_signal.set(false);

                deviceobject.connect_local("pre-update", false,
                    clone!(@weak self as widget => @default-return None, move |_| {
                        widget.block_signal.set(true);

                        None
                    })
                );
                deviceobject.connect_local("post-update", false,
                clone!(@weak self as widget => @default-return None, move |_| {
                        widget.block_signal.set(false);
                        widget.update_selected();

                        None
                    })
                );


                deviceobject.connect_profile_index_notify(
                    clone!(@weak self as widget => move |_| widget.update_selected())
                );
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
                let box_ = gtk::Box::new(gtk::Orientation::Horizontal, 6);
                let unavailable_icon = gtk::Image::from_icon_name("action-unavailable-symbolic");
                let label = gtk::Label::new(None);
                box_.append(&label);
                box_.append(&unavailable_icon);
                label.set_xalign(0.0);
                if !list {
                    label.set_ellipsize(gtk::pango::EllipsizeMode::End);
                }
                label.set_use_markup(true);

                item.property_expression("item")
                    .chain_property::<PwProfileObject>("description")
                    .bind(&label, "label", gtk::Widget::NONE);

                // let opacity_closure = closure_local!(|_: Option<glib::Object>, availability: u32| {
                //     match availability {
                //         2 => 1.0f32,
                //         _ => 0.5f32
                //     }
                // });

                // item.property_expression("item")
                //     .chain_property::<PwProfileObject>("availability")
                //     .chain_closure::<f32>(opacity_closure)
                //     .bind(&label, "opacity", glib::Object::NONE);

                let icon_closure = closure_local!(|_: Option<glib::Object>, availability: ProfileAvailability| {
                    availability == ProfileAvailability::No
                });

                item.property_expression("item")
                    .chain_property::<PwProfileObject>("availability")
                    .chain_closure::<bool>(icon_closure)
                    .bind(&unavailable_icon, "visible", glib::Object::NONE);
                
                item.set_child(Some(&box_));
            }

            let factory = gtk::SignalListItemFactory::new();
            factory.connect_setup(|_, item| setup_handler(item, false));

            let list_factory = gtk::SignalListItemFactory::new();
            list_factory.connect_setup(|_, item| setup_handler(item, true));

            self.profile_dropdown.set_factory(Some(&factory));
            self.profile_dropdown.set_list_factory(Some(&list_factory));

            self.profile_dropdown.set_enable_search(true);

            let widget = self.obj();
            let selected_handler = closure_local!(
                @watch widget => move |dropdown: &gtk::DropDown, _pspec: &glib::ParamSpec| {
                wp::info!("selected");
                if widget.imp().block_signal.get() {
                    return;
                }

                if let Some(deviceobject) = widget.deviceobject() {
                    wp::log::critical!("Had set profile to {}", dropdown.selected());
                    
                    deviceobject.set_profile(dropdown.selected() as i32);
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
