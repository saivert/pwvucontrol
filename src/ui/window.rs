// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::Duration;

use crate::{
    application::PwvucontrolApplication,
    backend::{NodeType, PwDeviceObject, PwNodeObject, PwvucontrolManager},
    config::{APP_ID, PROFILE},
    ui::{devicebox::PwDeviceBox, PwSinkBox, PwStreamBox},
};
use adw::subclass::prelude::*;
use gettextrs::gettext;
use glib::clone;
use gtk::{gio, prelude::*};
use std::cell::Cell;
use std::time;

pub enum PwvucontrolWindowView {
    Connected,
    Disconnected,
}
mod imp {
    use crate::backend::PwNodeFilterModel;

    use super::*;

    #[derive(Debug, gtk::CompositeTemplate)]
    #[template(resource = "/com/saivert/pwvucontrol/gtk/window.ui")]
    pub struct PwvucontrolWindow {
        #[template_child]
        pub header_bar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub stack: TemplateChild<adw::ViewStack>,
        #[template_child]
        pub playbacklist: TemplateChild<gtk::ListView>,
        #[template_child]
        pub recordlist: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub inputlist: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub outputlist: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub cardlist: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub viewstack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub reconnectbtn: TemplateChild<gtk::Button>,
        #[template_child]
        pub info_banner: TemplateChild<adw::Banner>,

        pub settings: gio::Settings,

        pub beep_elapsed: Cell<time::Instant>,

        pub experimental_toggle: Cell<bool>,

        pub filtermodel: PwNodeFilterModel,
    }

    impl Default for PwvucontrolWindow {
        fn default() -> Self {
            Self {
                header_bar: TemplateChild::default(),
                stack: TemplateChild::default(),
                playbacklist: TemplateChild::default(),
                recordlist: TemplateChild::default(),
                inputlist: TemplateChild::default(),
                outputlist: TemplateChild::default(),
                cardlist: TemplateChild::default(),
                viewstack: TemplateChild::default(),
                reconnectbtn: TemplateChild::default(),
                settings: gio::Settings::new(APP_ID),
                info_banner: TemplateChild::default(),
                beep_elapsed: Cell::new(std::time::Instant::now()),
                experimental_toggle: Cell::new(false),
                filtermodel: Default::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PwvucontrolWindow {
        const NAME: &'static str = "PwvucontrolWindow";
        type Type = super::PwvucontrolWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PwvucontrolWindow {
        fn constructed(&self) {
            self.parent_constructed();

            // Devel Profile
            if PROFILE == "Devel" {
                self.obj().add_css_class("devel");
            }

            crate::ui::remember_window_size(self.obj().upcast_ref(), &self.settings);

            self.obj().setup_scroll_blocker(&*self.playbacklist);
            self.obj().setup_scroll_blocker(&*self.recordlist);
            self.obj().setup_scroll_blocker(&*self.inputlist);
            self.obj().setup_scroll_blocker(&*self.outputlist);

            let manager = PwvucontrolManager::default();

            let wp_core = manager.wp_core();

            wp_core.connect_connected(clone!(@weak self as window => move |_obj| {
                window.obj().set_view(PwvucontrolWindowView::Connected);
            }));

            wp_core.connect_disconnected(clone!(@weak self as window => move |_obj| {
                window.obj().set_view(PwvucontrolWindowView::Disconnected);
            }));

            manager.node_model().connect_items_changed(clone!(@weak self as widget => move |_,_,_,_| {
                widget.obj().update_info_bar();
            }));
            manager.device_model().connect_items_changed(clone!(@weak self as widget => move |_,_,_,_| {
                widget.obj().update_info_bar();
            }));

            glib::idle_add_local_once(clone!(@weak self as widget => move || {widget.obj().update_info_bar();}));

            let factory = gtk::SignalListItemFactory::new();
            factory.connect_setup(|_, item| {
                let item = item.downcast_ref::<gtk::ListItem>().expect("ListItem");
                item.set_activatable(false);
            });
            factory.connect_bind(|_, item| {
                let item: &gtk::ListItem = item.downcast_ref().expect("ListItem");
                let node: PwNodeObject = item.item().and_downcast().expect("RowData is of wrong type");
                let whoaw: Option<gtk::Widget> = match node.nodetype() {
                    NodeType::StreamInput | NodeType::StreamOutput if !node.hidden() => Some(PwStreamBox::new(&node).upcast()),
                    NodeType::Sink | NodeType::Source if !node.hidden() => Some(PwSinkBox::new(&node).upcast()),
                    _ => None,
                };
                item.set_child(whoaw.as_ref());
            });
            factory.connect_unbind(|_, item| {
                let item: &gtk::ListItem = item.downcast_ref().expect("ListItem");
                // let streambox: PwStreamBox = item.child().and_downcast().expect("RowData is of wrong type");
                // streambox.emit_by_name::<()>("destroy", &[]);
                item.set_child(gtk::Widget::NONE);
            });

            self.playbacklist.set_factory(Some(&factory));
            self.playbacklist.set_show_separators(true);

            self.filtermodel.set_model(Some(manager.node_model()));

            let sorter = gtk::CustomSorter::new(|a, b| {
                let node_a: &PwNodeObject = a.downcast_ref().unwrap();
                let node_b: &PwNodeObject = b.downcast_ref().unwrap();
                node_a.nodetype().cmp(&node_b.nodetype()).into()
            });
            let sorted_model = gtk::SortListModel::new(Some(self.filtermodel.clone()), None::<gtk::Sorter>);
            sorted_model.set_section_sorter(Some(&sorter));

            let selection_model = gtk::NoSelection::new(Some(sorted_model));

            self.playbacklist.set_header_factory(Some(&Self::create_header_factory()));

            self.playbacklist.set_model(Some(&selection_model));

            self.inputlist.bind_model(
                Some(&manager.source_model()),
                clone!(@weak self as window => @default-panic, move |item| {
                    PwSinkBox::new(
                        item.downcast_ref::<PwNodeObject>()
                            .expect("RowData is of wrong type"),
                    )
                    .upcast::<gtk::Widget>()
                }),
            );

            self.outputlist.bind_model(
                Some(&manager.sink_model()),
                clone!(@weak self as window => @default-panic, move |item| {
                    PwSinkBox::new(
                        item.downcast_ref::<PwNodeObject>()
                            .expect("RowData is of wrong type"),
                    )
                    .upcast::<gtk::Widget>()
                }),
            );

            self.cardlist.bind_model(
                Some(&manager.device_model()),
                clone!(@weak self as window => @default-panic, move |item| {
                    let obj: &PwDeviceObject = item.downcast_ref().expect("PwDeviceObject");
                    PwDeviceBox::new(obj).upcast::<gtk::Widget>()
                }),
            );

            self.reconnectbtn.connect_clicked(|_| {
                let manager = PwvucontrolManager::default();
                if let Some(core) = manager.imp().wp_core.get() {
                    core.connect();
                }
            });

            let overamplification_action = self.settings.create_action("enable-overamplification");
            self.obj().add_action(&overamplification_action);
            let use_led_peakmeter_action = self.settings.create_action("use-peakmeter-led");
            self.obj().add_action(&use_led_peakmeter_action);
            let beep_on_volume_changes_action = self.settings.create_action("beep-on-volume-changes");
            self.obj().add_action(&beep_on_volume_changes_action);

            let experimental_action = gio::SimpleAction::new("bleh", None);
            experimental_action.connect_activate(clone!(@weak self as obj => move |_, _| {
                let state = !obj.experimental_toggle.get();
                obj.experimental_toggle.set(state);

                let node_type = if state { NodeType::StreamOutput } else { NodeType::Undefined };

                if state {
                    obj.playbacklist.set_header_factory(gtk::ListItemFactory::NONE);
                } else {
                    obj.playbacklist.set_header_factory(Some(&Self::create_header_factory()));
                }

                obj.filtermodel.set_nodetype(node_type);
            }));
            self.obj().add_action(&experimental_action);
            PwvucontrolApplication::default().set_accels_for_action("win.bleh", &["<Control>y"]);
        }
    }
    impl WidgetImpl for PwvucontrolWindow {}
    impl WindowImpl for PwvucontrolWindow {}
    impl ApplicationWindowImpl for PwvucontrolWindow {}
    impl AdwApplicationWindowImpl for PwvucontrolWindow {}

    impl PwvucontrolWindow {
        fn create_header_factory() -> gtk::SignalListItemFactory {
            let header_factory = gtk::SignalListItemFactory::new();

            header_factory.connect_setup(|_, item| {
                let header = item.downcast_ref::<gtk::ListHeader>().expect("ListHeader");
                let label = gtk::Label::new(Some("nice"));
                header.set_child(Some(&label));
            });

            header_factory.connect_bind(|_, item| {
                let header = item.downcast_ref::<gtk::ListHeader>().expect("ListHeader");
                let label: gtk::Label = header.child().and_downcast().expect("Label in section");
                if let Some(node) = header.item().and_downcast_ref::<PwNodeObject>() {
                    let title = format!(
                        "{} ({})",
                        match node.nodetype() {
                            NodeType::StreamOutput => gettext("Playback"),
                            NodeType::StreamInput => gettext("Recording"),
                            NodeType::Sink => gettext("Output devices"),
                            NodeType::Source => gettext("Input devices"),
                            _ => gettext("Unknown"),
                        },
                        header.n_items()
                    );

                    label.set_label(&title);
                }
            });

            header_factory
        }
    }
}

glib::wrapper! {
    pub struct PwvucontrolWindow(ObjectSubclass<imp::PwvucontrolWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl PwvucontrolWindow {
    pub fn new(application: &PwvucontrolApplication) -> Self {
        glib::Object::builder().property("application", application).build()
    }

    pub(crate) fn set_view(&self, view: PwvucontrolWindowView) {
        let imp = self.imp();
        match view {
            PwvucontrolWindowView::Connected => imp.viewstack.set_visible_child_name("connected"),
            PwvucontrolWindowView::Disconnected => imp.viewstack.set_visible_child_name("disconnected"),
        }
    }

    /// This prevents child widgets from capturing scroll events
    fn setup_scroll_blocker(&self, listbox: &impl IsA<gtk::Widget>) {
        let scrolledwindow = listbox
            .ancestor(gtk::ScrolledWindow::static_type())
            .and_then(|x| x.downcast::<gtk::ScrolledWindow>().ok())
            .expect("downcast to scrolled window");

        let ecs = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
        ecs.set_propagation_phase(gtk::PropagationPhase::Capture);
        ecs.set_propagation_limit(gtk::PropagationLimit::SameNative);

        // Need to actually handle the scroll event in order to block propagation
        ecs.connect_local(
            "scroll",
            false,
            clone!(@weak scrolledwindow => @default-return None, move |v| {
                let y: f64 = v.get(2).unwrap().get().unwrap();

                // No way to redirect this event to underlying widget so we need to reimplement the scroll handling
                let adjustment = scrolledwindow.vadjustment();

                if (adjustment.upper() - adjustment.page_size()).abs() < f64::EPSILON {
                    return Some(false.to_value());
                }

                adjustment.set_value(adjustment.value() + y*adjustment.page_size().powf(2.0 / 3.0));

                Some(true.to_value())
            }),
        );
        scrolledwindow.add_controller(ecs);
    }

    fn update_info_bar(&self) {
        let manager = PwvucontrolManager::default();
        let imp = self.imp();

        let message = if manager.device_model().n_items() == 0 {
            gettext("No sound cards detected. Check pipewire configuration.")
        } else {
            gettext("No sound devices detected. Check profiles in Card tab.")
        };
        imp.info_banner.set_title(&message);
        imp.info_banner.set_revealed(manager.node_model().n_items() == 0);
    }

    pub(crate) fn play_beep(&self) {
        if !self.imp().settings.boolean("beep-on-volume-changes") {
            return;
        }
        if self.imp().beep_elapsed.get().elapsed() > Duration::from_secs(1) {
            self.display().beep();
            self.imp().beep_elapsed.set(time::Instant::now());
        }
    }

    pub(crate) fn select_tab(&self, tab: i32) {
        match tab {
            1 => self.imp().stack.set_visible_child_name("playback"),
            2 => self.imp().stack.set_visible_child_name("recording"),
            3 => self.imp().stack.set_visible_child_name("inputdevices"),
            4 => self.imp().stack.set_visible_child_name("outputdevices"),
            5 => self.imp().stack.set_visible_child_name("cards"),
            _ => {}
        }
    }
}

impl Default for PwvucontrolWindow {
    fn default() -> Self {
        PwvucontrolApplication::default().active_window().unwrap().downcast().unwrap()
    }
}
