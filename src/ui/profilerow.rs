// SPDX-License-Identifier: GPL-3.0-or-later

use glib::closure_local;
use gtk::{prelude::*, subclass::prelude::*};
use std::cell::RefCell;

use crate::backend::ParamAvailability;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/saivert/pwvucontrol/gtk/profilerow.ui")]
    pub struct PwProfileRow {
        #[template_child]
        pub label: TemplateChild<gtk::Label>,
        #[template_child]
        pub unavailable_icon: TemplateChild<gtk::Image>,
        #[template_child]
        pub checkmark_icon: TemplateChild<gtk::Image>,

        pub(super) signalid: RefCell<Option<glib::SignalHandlerId>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PwProfileRow {
        const NAME: &'static str = "PwProfileRow";
        type Type = super::PwProfileRow;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PwProfileRow {}
    impl WidgetImpl for PwProfileRow {}
    impl BoxImpl for PwProfileRow {}
    impl PwProfileRow {}
}

glib::wrapper! {
    pub struct PwProfileRow(ObjectSubclass<imp::PwProfileRow>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl PwProfileRow {
    pub(crate) fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn setup<Type: glib::IsA<glib::Object>>(&self, item: &gtk::ListItem, list: bool) {
        let label = self.imp().label.get();
        let unavailable_icon = self.imp().unavailable_icon.get();

        if !list {
            label.set_ellipsize(gtk::pango::EllipsizeMode::End);
            self.imp().checkmark_icon.set_visible(false);
        }

        item.property_expression("item")
        .chain_property::<Type>("description")
        .bind(&label, "label", gtk::Widget::NONE);

        let icon_closure = closure_local!(|_: Option<glib::Object>, availability: ParamAvailability| {
            availability == ParamAvailability::No
        });

        item.property_expression("item")
            .chain_property::<Type>("availability")
            .chain_closure::<bool>(icon_closure)
            .bind(&unavailable_icon, "visible", glib::Object::NONE);
    }

    pub fn set_selected(&self, selected: bool) {
        self.imp().checkmark_icon.set_opacity(if selected {1.0} else {0.0});
    }

    pub fn set_handlerid(&self, id: Option<glib::SignalHandlerId>) {
        self.imp().signalid.replace(id);
    }
}
