// SPDX-License-Identifier: GPL-3.0-or-later

use glib::closure_local;
use gtk::{self, prelude::*, subclass::prelude::*};
use glib::clone;
use wp::pw::ProxyExt;
use std::cell::{Cell, RefCell};
use wireplumber as wp;
use crate::ui::PwProfileRow;
use crate::macros::*;

mod imp {
    use crate::backend::{NodeType, PwNodeObject, PwRouteFilterModel, PwRouteObject};

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[properties(wrapper_type = super::PwRouteDropDown)]
    #[template(resource = "/com/saivert/pwvucontrol/gtk/route-dropdown.ui")]
    pub struct PwRouteDropDown {
        #[property(get, set = Self::set_nodeobject, nullable)]
        pub(super) nodeobject: RefCell<Option<PwNodeObject>>,

        #[template_child]
        pub route_dropdown: TemplateChild<gtk::DropDown>,

        pub(super) block_signal: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PwRouteDropDown {
        const NAME: &'static str = "PwRouteDropDown";
        type Type = super::PwRouteDropDown;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl PwRouteDropDown {
        pub fn update_selected(&self) {
            if let Some(index) = self.get_route_index() {
                pwvucontrol_info!("update_selected with index {index}");
                self.obj().set_selected_no_send(index);
            }
        }

        fn get_route_index(&self) -> Option<u32> {
            let nodeobject = self.nodeobject.borrow();
            let nodeobject = nodeobject.as_ref().unwrap();

            let Some(deviceobject) = nodeobject.get_device() else {
                return None;
            };
            match nodeobject.nodetype() {
                NodeType::Source => Some(deviceobject.route_index_input()),
                NodeType::Sink => Some(deviceobject.route_index_output()),
                _ => None
            }
        }

        fn get_route_model(&self) -> Option<PwRouteFilterModel> {
            let nodeobject = self.nodeobject.borrow();
            let nodeobject = nodeobject.as_ref().unwrap();

            let deviceobject = nodeobject.get_device().expect("device");
            match nodeobject.nodetype() {
                NodeType::Source => Some(deviceobject.routemodel_input()),
                NodeType::Sink => Some(deviceobject.routemodel_output()),
                _ => None
            }
        }

        pub fn set_nodeobject(&self, new_nodeobject: Option<&PwNodeObject>) {
            self.nodeobject.replace(new_nodeobject.cloned());

            if let Some(nodeobject) = new_nodeobject {

                let deviceobject = nodeobject.get_device().expect("device");

                self.block_signal.set(true);
                pwvucontrol_info!("self.route_dropdown.set_model({});", deviceobject.wpdevice().bound_id());
                self.route_dropdown.set_model(self.get_route_model().as_ref());
                if let Some(index) = self.get_route_index() {
                    pwvucontrol_info!("self.route_dropdown.set_selected({index});");
                    self.route_dropdown.set_selected(index);
                }

                self.block_signal.set(false);

                deviceobject.connect_local("pre-update-route", false,
                    clone!(@weak self as widget => @default-return None, move |_| {
                        widget.block_signal.set(true);

                        None
                    })
                );

                deviceobject.connect_local("post-update-route", false,
                clone!(@weak self as widget => @default-return None, move |_| {
                        widget.block_signal.set(false);
                        pwvucontrol_info!("About to call widget.update_selected() inside post-update-route handler");
                        widget.update_selected();

                        None
                    })
                );

                deviceobject.connect_route_index_output_notify(
                    clone!(@weak self as widget => move |_| widget.update_selected())
                );
            } else {
                self.route_dropdown.set_model(gtk::gio::ListModel::NONE);
            }
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for PwRouteDropDown {
        fn dispose(&self) {
            self.dispose_template();
        }

        fn constructed(&self) {
            self.parent_constructed();

            fn setup_handler(item: &glib::Object, list: bool) {
                let item: &gtk::ListItem = item.downcast_ref().expect("ListItem");
                let profilerow = PwProfileRow::new();

                profilerow.setup::<PwRouteObject>(item, list);
                item.set_child(Some(&profilerow));
            }

            fn bind_handler(item: &glib::Object, dropdown: &gtk::DropDown) {
                let item: &gtk::ListItem = item.downcast_ref().expect("ListItem");
                let profilerow = item
                    .child()
                    .and_downcast::<PwProfileRow>()
                    .expect("PwProfileRow child");

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

            let dropdown = self.route_dropdown.get();

            let factory = gtk::SignalListItemFactory::new();
            factory.connect_setup(|_, item| setup_handler(item, false));

            let list_factory = gtk::SignalListItemFactory::new();
            list_factory.connect_setup(|_, item| setup_handler(item, true));
            list_factory.connect_bind(clone!(@weak dropdown => move |_, item| bind_handler(item, &dropdown)));
            list_factory.connect_unbind(|_, item| unbind_handler(item));

            self.route_dropdown.set_factory(Some(&factory));
            self.route_dropdown.set_list_factory(Some(&list_factory));


            let widget = self.obj();
            let selected_handler = closure_local!(
                @watch widget => move |dropdown: &gtk::DropDown, _pspec: &glib::ParamSpec| {
                wp::info!("selected");
                if widget.imp().block_signal.get() {
                    return;
                }

                if let Some(nodeobject) = widget.nodeobject() {
                    pwvucontrol_critical!("Had set profile to {}", dropdown.selected());

                    if let Some(routeobject) = dropdown.selected_item().and_downcast::<PwRouteObject>() {
                        nodeobject.set_route(&routeobject);
                    }

                }
            });
            self.route_dropdown.connect_closure("notify::selected", true, selected_handler);
        }
    }

    impl WidgetImpl for PwRouteDropDown {}
}

glib::wrapper! {
    pub struct PwRouteDropDown(ObjectSubclass<imp::PwRouteDropDown>) @extends gtk::Widget;
}

impl PwRouteDropDown {

    pub fn set_selected_no_send(&self, position: u32) {
        let imp = self.imp();

        imp.block_signal.set(true);
        imp.route_dropdown.set_selected(position);
        imp.block_signal.set(false);
    }
}

impl Default for PwRouteDropDown {
    fn default() -> Self {
        glib::Object::new()
    }
}
