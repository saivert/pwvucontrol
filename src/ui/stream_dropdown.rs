// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{
    ui::WithDefaultListModel,
    backend::PwNodeObject,
    backend::PwvucontrolManager,
};
use glib::closure_local;
use gtk::{self, prelude::*, subclass::prelude::*};
use std::cell::{Cell, RefCell};
use wireplumber as wp;

mod imp {
    use crate::{backend::NodeType, pwvucontrol_warning};

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[properties(wrapper_type = super::PwOutputDropDown)]
    #[template(resource = "/com/saivert/pwvucontrol/gtk/output-dropdown.ui")]
    pub struct PwStreamDropDown {
        #[property(get, set = Self::set_nodeobj, nullable)]
        pub(super) nodeobj: RefCell<Option<PwNodeObject>>,

        #[template_child]
        pub outputdevice_dropdown: TemplateChild<gtk::DropDown>,

        pub(super) block_signal: Cell<bool>,
        pub(super) dropdown_model: RefCell<WithDefaultListModel>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PwStreamDropDown {
        const NAME: &'static str = "PwOutputDropDown";
        type Type = super::PwOutputDropDown;
        type ParentType = gtk::Widget;


        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[gtk::template_callbacks]
    impl PwStreamDropDown {
        fn set_nodeobj(&self, nodeobj: Option<&PwNodeObject>) {
            let manager = PwvucontrolManager::default();

            let Some(nodeobj) = nodeobj else {
                pwvucontrol_warning!("PwOutputDropDown::set_nodeobj: Tried to set nodeobj to None");
                return;
            };

            let model = match nodeobj.nodetype() {
                NodeType::StreamOutput => manager.sink_model(),
                NodeType::StreamInput => manager.source_model(),
                _ => return,
            };

            self.dropdown_model.replace(WithDefaultListModel::new(Some(&model)));
            self.outputdevice_dropdown.set_model(Some(&*self.dropdown_model.borrow()));

        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for PwStreamDropDown {
        // Needed for direct subclasses of GtkWidget;
        // Here you need to unparent all direct children
        // of your template.
        fn dispose(&self) {
            self.dispose_template();
        }

        fn constructed(&self) {
            self.parent_constructed();


            
            fn setup_handler(item: &glib::Object) {
                let item: &gtk::ListItem = item.downcast_ref().expect("ListItem");
                let label = gtk::Label::new(None);
                label.set_xalign(0.0);
                label.set_ellipsize(gtk::pango::EllipsizeMode::End);

                item.property_expression("item")
                    .chain_closure::<Option<String>>(closure_local!(
                        move |_: Option<glib::Object>, item: Option<glib::Object>| {
                            if let Some(item) = item {
                                if let Some(item) = item.downcast_ref::<PwNodeObject>() {
                                    return Some(item.name());
                                }
                                if let Some(item) = item.downcast_ref::<gtk::StringObject>() {
                                    return Some(item.string().to_string());
                                }
                            }

                            None
                        }
                    ))
                    .bind(&label, "label", gtk::Widget::NONE);

                item.set_child(Some(&label));
            }

            let factory = gtk::SignalListItemFactory::new();
            factory.connect_setup(|_, item| setup_handler(item));

            // We need to store the DropDown widget's internal default factory so we can reset the list-factory later
            // which would otherwise just use the factory we set
            let default_dropdown_factory = self.outputdevice_dropdown.factory();
            self.outputdevice_dropdown.set_factory(Some(&factory));
            self.outputdevice_dropdown.set_list_factory(default_dropdown_factory.as_ref());

            self.outputdevice_dropdown.set_enable_search(true);


            self.outputdevice_dropdown
                .set_expression(Some(gtk::ClosureExpression::new::<Option<String>>(
                    gtk::Expression::NONE,
                    closure_local!(move |item: glib::Object| {
                        if let Some(item) = item.downcast_ref::<PwNodeObject>() {
                            Some(item.name())
                        } else {
                            item.downcast_ref::<gtk::StringObject>().map(|item| item.string().to_string())
                        }
                    }),
                )));


            let widget = self.obj();
            let selected_handler = closure_local!(
                @watch widget => move |dropdown: &gtk::DropDown, _pspec: &glib::ParamSpec| {
                wp::info!("selected-item");
                let nodeobj = widget.imp().nodeobj.borrow();
                if nodeobj.is_none() {
                    return;
                }
                let nodeobj = nodeobj.as_ref().expect("nodeobj set on PwOutputDropDown");
                if widget.imp().block_signal.get() {
                    return;
                }
                if dropdown.selected() == 0 {
                    nodeobj.unset_default_target();
                    return;
                }
                if let Some(item) = dropdown.selected_item() {
                    if let Some(item) = item.downcast_ref::<PwNodeObject>() {
                        nodeobj.set_default_target(item);
                    }
                }
            });

            self.outputdevice_dropdown.connect_closure("notify::selected-item", true, selected_handler);
        }
    }

    impl WidgetImpl for PwStreamDropDown {}
}

glib::wrapper! {
    pub struct PwOutputDropDown(ObjectSubclass<imp::PwStreamDropDown>) @extends gtk::Widget;
}

impl PwOutputDropDown {

    pub fn new(nodeobj: Option<&PwNodeObject>) -> Self {
        glib::Object::builder()
        .property("nodeobj", nodeobj)
        .build()
    }

    pub fn set_selected_no_send(&self, position: u32) {
        let imp = self.imp();

        imp.block_signal.set(true);
        imp.outputdevice_dropdown.set_selected(position);
        imp.block_signal.set(false);
    }

    pub fn set_default_text(&self, text: &str) {
        let imp = self.imp();

        imp.block_signal.set(true);
        imp.dropdown_model.borrow().set_default_text(text);
        imp.block_signal.set(false);
    }
}

impl Default for PwOutputDropDown {
    fn default() -> Self {
        Self::new(None)
    }
}