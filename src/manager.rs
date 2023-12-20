// SPDX-License-Identifier: GPL-3.0-or-later

 use gtk::{
    gio,
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
};

use wireplumber as wp;
use wp::{plugin::PluginFeatures, pw::MetadataExt, registry::{ObjectManager, Interest, Constraint, ConstraintType}};

use crate::{PwvucontrolWindow, PwvucontrolApplication};

mod imp {
    use std::{str::FromStr, cell::RefCell};

    use crate::{pwnodeobject::PwNodeObject, window::PwvucontrolWindowView, pwnodemodel::PwNodeModel, pwdeviceobject::PwDeviceObject};

    use super::*;
    use glib::Properties;
    use once_cell::unsync::OnceCell;
    use wp::{pw::{ProxyExt, PipewireObjectExt2}, plugin::*};

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::PwvucontrolManager)]
    pub struct PwvucontrolManager {
        pub wp_core: OnceCell<wp::core::Core>,
        pub wp_object_manager: OnceCell<wp::registry::ObjectManager>,

        pub nodemodel: PwNodeModel,
        pub sinkmodel: PwNodeModel,
        pub devicemodel: OnceCell<gio::ListStore>,

        pub metadata_om: OnceCell<wp::registry::ObjectManager>,
        pub metadata: RefCell<Option<wp::pw::Metadata>>,

        #[property(get, set, construct_only)]
        application: RefCell<Option<PwvucontrolApplication>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PwvucontrolManager {
        const NAME: &'static str = "PwvucontrolManager";
        type Type = super::PwvucontrolManager;
    }

    #[glib::derived_properties]
    impl ObjectImpl for PwvucontrolManager {
        fn constructed(&self) {
            self.parent_constructed();
            self.devicemodel.set(gio::ListStore::new::<PwDeviceObject>()).expect("devicemodel not set");

            self.setup_wp_connection();
            self.setup_metadata_om();
        }
    }

    impl PwvucontrolManager {
        fn setup_wp_connection(&self) {
            wp::core::Core::init();

            wireplumber::Log::set_default_level("3");

            let props = wp::pw::Properties::new_string("media.category=Manager");

            let wp_core = wp::core::Core::new(Some(&glib::MainContext::default()), Some(props));
            let wp_om = ObjectManager::new();

            wp_core.connect_local("connected", false, |_obj| {
                // let app = PwvucontrolManager::default();
                // let win = app.window.get().expect("window");
                let win = PwvucontrolWindow::default();
                win.set_view(PwvucontrolWindowView::Connected);
                None
            });

            wp_core.connect_local("disconnected", false, |_obj| {
                // let app = PwvucontrolManager::default();
                // let win = app.window.get().expect("window");
                let win = PwvucontrolWindow::default();
                win.set_view(PwvucontrolWindowView::Disconnected);
                None
            });


            wp_core.connect();

            wp_core.load_component("libwireplumber-module-mixer-api", "module", None).expect("loadig mixer-api plugin");
            wp_core.load_component("libwireplumber-module-default-nodes-api", "module", None).expect("loadig mixer-api plugin");

            

            wp_om.add_interest(
                {
                    let interest = wp::registry::ObjectInterest::new(
                        wp::pw::Node::static_type(),
                    );
                    let variant = glib::Variant::from_str("('Stream/Output/Audio', 'Stream/Input/Audio', 'Audio/Sink')").expect("variant");
                    interest.add_constraint(
                        wp::registry::ConstraintType::PwGlobalProperty,
                        "media.class",
                        wp::registry::ConstraintVerb::InList,
                        Some(&variant));
    
                    interest
                }
            );

            wp_om.add_interest(
                {
                    let interest = wp::registry::ObjectInterest::new(
                        wp::pw::Device::static_type(),
                    );
                    interest.add_constraint(
                        wp::registry::ConstraintType::PwGlobalProperty,
                        "media.class",
                        wp::registry::ConstraintVerb::Equals,
                        Some(&"Audio/Device".to_variant()));
    
                    interest
                }
            );

            wp_om.request_object_features(
                wp::pw::Node::static_type(),
                wp::core::ObjectFeatures::ALL,
            );

            wp_om.request_object_features(
                wp::pw::GlobalProxy::static_type(),
                wp::core::ObjectFeatures::ALL,
            );

            wp_om.connect_object_added(
                clone!(@weak self as imp, @weak wp_core as core => move |_, object| {
                    let devicemodel = imp.devicemodel.get().expect("devicemodel");
                    if let Some(node) = object.dynamic_cast_ref::<wp::pw::Node>() {
                        // Hide ourselves
                        if node.name() == Some("pwvucontrol-peak-detect".to_string()) {
                            return;
                        }
                        // Hide any playback from pavucontrol (mainly volume control notification sound)
                        if node.name() == Some("pavucontrol".to_string()) {
                            return;
                        }
                        if node.pw_property::<String>("media.class").unwrap_or_default() == "Stream/Input/Audio" {
                            if let Ok(medianame) = node.pw_property::<String>("application.id") {
                                let hidden_apps = ["org.PulseAudio.pavucontrol", "org.gnome.VolumeControl", "org.kde.kmixd"];
                                for app in hidden_apps {
                                    if app == medianame {
                                        return;
                                    }
                                }
                            }
                        }
                        wp::log::info!("added: {:?}", node.name());
                        let pwobj = PwNodeObject::new(node);
                        let model = match pwobj.nodetype() {
                            crate::NodeType::Sink => &imp.sinkmodel,
                            _ => &imp.nodemodel
                        };
                        model.append(&pwobj);
                    } else if let Some(device) = object.dynamic_cast_ref::<wp::pw::Device>() {
                        let n: String = device.pw_property("device.name").unwrap();
                        wp::log::info!("Got device {} {n}", device.bound_id());

                        devicemodel.append(&PwDeviceObject::new(device));
                        
                    } else {
                        unreachable!("Object must be one of the above, but is {:?} instead", object.type_());
                    }
                }),
            );

            wp_om.connect_object_removed(clone!(@weak self as imp => move |_, object| {
                let devicemodel = imp.devicemodel.get().expect("devicemodel");
                if let Some(node) = object.dynamic_cast_ref::<wp::pw::Node>() {
                    wp::log::info!("removed: {:?} id: {}", node.name(), node.bound_id());
                    let model = match crate::pwnodeobject::get_node_type_for_node(node) {
                        crate::NodeType::Sink => &imp.sinkmodel,
                        _ => &imp.nodemodel
                    };
                    model.remove(node.bound_id());

                } else if let Some(device) = object.dynamic_cast_ref::<wp::pw::Device>() {
                    for item in devicemodel.iter::<PwDeviceObject>() {
                        if let Ok(item) = item {
                            if item.wpdevice().bound_id() == device.bound_id() {
                                if let Some(pos) = devicemodel.find(&item) {
                                    wp::log::info!("Removed device {} @ pos {pos}", device.bound_id());
                                    devicemodel.remove(pos);
                                }
                            }
                        }
                    }
                } else {
                    wp::log::info!("Object must be one of the above, but is {:?} instead", object.type_());
                }
            }));

            glib::MainContext::default().spawn_local(clone!(@weak self as manager, @weak wp_core as core, @weak wp_om as om => async move {
                let plugin_names = vec!["mixer-api", "default-nodes-api"];
                let mut count = 0;
                for plugin_name in plugin_names {
                    if let Some(plugin) = Plugin::find(&core, plugin_name) {
                        let result = plugin.activate_future(PluginFeatures::ENABLED).await;
                        if result.is_err() {
                            wp::log::critical!("Cannot activate plugin {plugin_name}");
                        } else {
                            wp::log::info!("Activated plugin {plugin_name}");
                            count += 1;
                            if count == 2 {
                                core.install_object_manager(&om);
                            }
                        }
                    } else {
                        wp::log::critical!("Cannot find plugin {plugin_name}");
                        manager.application.borrow().as_ref().unwrap().quit();
                    }
                }
            }));

            self.wp_core
                .set(wp_core)
                .expect("wp_core should only be set once during application activation");
            self.wp_object_manager
                .set(wp_om)
                .expect("wp_object_manager should only be set once during application activation");
   
        }

        fn setup_metadata_om(&self) {
            let metadata_om = ObjectManager::new();

            let wp_core = self.wp_core.get().expect("wp_core to be set");

            metadata_om.add_interest([
                Constraint::compare(ConstraintType::PwGlobalProperty, "metadata.name", "default", true),
            ].iter().collect::<Interest<wp::pw::Metadata>>());

            metadata_om.request_object_features(
                wp::pw::GlobalProxy::static_type(),
                wp::core::ObjectFeatures::ALL,
            );

            metadata_om.connect_object_added(
                clone!(@weak self as imp, @weak wp_core as core => move |_, object| {
                    if let Some(metadataobj) = object.dynamic_cast_ref::<wp::pw::Metadata>() {
                        wp::log::info!("added metadata object: {:?}", metadataobj.bound_id());
                        imp.metadata.replace(Some(metadataobj.clone()));
                        for a in metadataobj.new_iterator(u32::MAX).expect("iterator") {
                            let (s, k, t, v) = wp::pw::Metadata::iterator_item_extract(&a);
                            wp::log::info!("Metadata value: {s}, {k:?}, {t:?}, {v:?}");
                        }
                    } else {
                        unreachable!("Object must be one of the above, but is {:?} instead", object.type_());
                    }
                }),
            );



            metadata_om.connect_objects_changed(clone!(@weak self as imp => move |_| {

            }));


            wp_core.install_object_manager(&metadata_om);
            self.metadata_om.set(metadata_om).expect("metadata object manager set already");
        }

    }
}

glib::wrapper! {
    pub struct PwvucontrolManager(ObjectSubclass<imp::PwvucontrolManager>);
}

impl PwvucontrolManager {
    pub(super) fn new<P: glib::IsA<gtk::Application>>(application: &P)  -> Self {
        glib::Object::builder()
            .property("application", application)
            .build()
    }
}
