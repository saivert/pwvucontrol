// SPDX-License-Identifier: GPL-3.0-or-later

use crate::macros::*;
use crate::{
    backend::NodeType, backend::PwDeviceObject, backend::PwNodeFilterModel, backend::PwNodeModel, backend::PwNodeObject, ui::PwvucontrolWindow,
    ui::PwvucontrolWindowView, PwvucontrolApplication,
};
use gtk::{
    gio,
    glib::{self, clone, Properties},
    prelude::*,
    subclass::prelude::*,
};
use once_cell::unsync::OnceCell;
use std::{cell::RefCell, str::FromStr};
use wireplumber as wp;
use wp::{
    plugin::{PluginFeatures, *},
    pw::{MetadataExt, PipewireObjectExt2, ProxyExt},
    registry::{Constraint, ConstraintType, Interest, ObjectManager},
};

mod imp {

    use super::*;

    #[derive(Properties)]
    #[properties(wrapper_type = super::PwvucontrolManager)]
    pub struct PwvucontrolManager {
        #[property(get)]
        pub wp_core: OnceCell<wp::core::Core>,
        pub wp_object_manager: OnceCell<wp::registry::ObjectManager>,

        #[property(get)]
        pub(crate) node_model: PwNodeModel,

        #[property(get)]
        pub(crate) stream_output_model: PwNodeFilterModel,

        #[property(get)]
        pub(crate) stream_input_model: PwNodeFilterModel,

        #[property(get)]
        pub(crate) source_model: PwNodeFilterModel,

        #[property(get)]
        pub(crate) sink_model: PwNodeFilterModel,

        #[property(get)]
        pub(crate) device_model: gio::ListStore,

        pub metadata_om: OnceCell<wp::registry::ObjectManager>,
        pub metadata: RefCell<Option<wp::pw::Metadata>>,

        #[property(get)]
        pub default_nodes_api: OnceCell<Plugin>,
        #[property(get)]
        pub mixer_api: OnceCell<Plugin>,

        #[property(get, set, construct_only)]
        application: RefCell<Option<PwvucontrolApplication>>,
    }

    impl Default for PwvucontrolManager {
        fn default() -> Self {
            Self {
                wp_core: Default::default(),
                wp_object_manager: Default::default(),
                node_model: Default::default(),
                stream_input_model: PwNodeFilterModel::new(NodeType::StreamInput, None::<gio::ListModel>),
                stream_output_model: PwNodeFilterModel::new(NodeType::StreamOutput, None::<gio::ListModel>),
                source_model: PwNodeFilterModel::new(NodeType::Source, None::<gio::ListModel>),
                sink_model: PwNodeFilterModel::new(NodeType::Sink, None::<gio::ListModel>),
                device_model: gio::ListStore::new::<PwDeviceObject>(),
                metadata_om: Default::default(),
                metadata: Default::default(),
                default_nodes_api: Default::default(),
                mixer_api: Default::default(),
                application: Default::default(),
            }
        }
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

            self.stream_input_model.set_model(Some(self.node_model.clone()));
            self.stream_output_model.set_model(Some(self.node_model.clone()));
            self.sink_model.set_model(Some(self.node_model.clone()));
            self.source_model.set_model(Some(self.node_model.clone()));

            self.setup_wp_connection();
            self.setup_metadata_om();
        }
    }

    impl PwvucontrolManager {
        fn setup_wp_connection(&self) {
            wp::core::Core::init_with_flags(wp::InitFlags::ALL);

            if !wp::Log::level_is_enabled(glib::LogLevelFlags::LEVEL_WARNING) {
                wp::Log::set_default_level("1");
            }

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

            wp_core
                .load_component("libwireplumber-module-mixer-api", "module", None)
                .expect("loadig mixer-api plugin");
            wp_core
                .load_component("libwireplumber-module-default-nodes-api", "module", None)
                .expect("loadig mixer-api plugin");

            wp_om.add_interest({
                let interest = wp::registry::ObjectInterest::new(wp::pw::Node::static_type());
                let variant =
                    glib::Variant::from_str("('Stream/Output/Audio', 'Stream/Input/Audio', 'Audio/Source', 'Audio/Sink')").expect("variant");
                interest.add_constraint(
                    wp::registry::ConstraintType::PwGlobalProperty,
                    "media.class",
                    wp::registry::ConstraintVerb::InList,
                    Some(&variant),
                );

                interest
            });

            wp_om.add_interest({
                let interest = wp::registry::ObjectInterest::new(wp::pw::Device::static_type());
                interest.add_constraint(
                    wp::registry::ConstraintType::PwGlobalProperty,
                    "media.class",
                    wp::registry::ConstraintVerb::Equals,
                    Some(&"Audio/Device".to_variant()),
                );

                interest
            });

            wp_om.request_object_features(wp::pw::Node::static_type(), wp::core::ObjectFeatures::ALL);

            wp_om.request_object_features(wp::pw::GlobalProxy::static_type(), wp::core::ObjectFeatures::ALL);

            wp_om.connect_object_added(clone!(@weak self as imp, @weak wp_core as core => move |_, object| {
                let devicemodel = &imp.device_model;
                if let Some(node) = object.dynamic_cast_ref::<wp::pw::Node>() {
                    // Hide ourselves.
                    if node.name().unwrap_or_default() == "pwvucontrol-peak-detect" {
                        return;
                    }

                    // Hide any playback from pavucontrol (mainly volume control notification sound).
                    if node.name().unwrap_or_default() == "pavucontrol" {
                        return;
                    }

                    // Hide any notification sounds.
                    // The presence of the event.id property means most likely this is an event sound.
                    if node.pw_property::<String>("event.id").is_ok() {
                        return;
                    }
                    // Or media.role being Notification.
                    if node.pw_property::<String>("media.role").unwrap_or_default() == "Notification" {
                        return;
                    }

                    // Hide applications that only record for peak meter.
                    if node.pw_property::<String>("stream.monitor").is_ok() {
                        return;
                    }

                    pwvucontrol_info!("Got node: {} bound id {}", node.name().unwrap_or_default(), node.bound_id());
                    let pwobj = PwNodeObject::new(node);
                    let model = &imp.node_model;
                    model.append(&pwobj);
                } else if let Some(device) = object.dynamic_cast_ref::<wp::pw::Device>() {
                    let n: String = device.pw_property("device.name").unwrap();
                    pwvucontrol_info!("Got device: {n} bound id {}", device.bound_id());
                    devicemodel.append(&PwDeviceObject::new(device));
                } else {
                    unreachable!("Object must be one of the above, but is {:?} instead", object.type_());
                }
            }));

            wp_om.connect_object_removed(clone!(@weak self as imp => move |_, object| {
                let devicemodel = &imp.device_model;
                if let Some(node) = object.dynamic_cast_ref::<wp::pw::Node>() {
                    pwvucontrol_info!("removed: {:?} id: {}", node.name(), node.bound_id());
                    let model = &imp.node_model;
                    model.remove(node.bound_id());

                } else if let Some(device) = object.dynamic_cast_ref::<wp::pw::Device>() {
                    for item in devicemodel.iter::<PwDeviceObject>() {
                        if let Ok(item) = item {
                            if item.wpdevice().bound_id() == device.bound_id() {
                                if let Some(pos) = devicemodel.find(&item) {
                                    pwvucontrol_info!("Removed device {} @ pos {pos}", device.bound_id());
                                    devicemodel.remove(pos);
                                }
                            }
                        }
                    }
                } else {
                    pwvucontrol_info!("Object must be one of the above, but is {:?} instead", object.type_());
                }
            }));

            glib::MainContext::default().spawn_local(clone!(@weak self as manager, @weak wp_core as core, @weak wp_om as om => async move {
                let plugin_names = vec![("mixer-api", &manager.mixer_api), ("default-nodes-api", &manager.default_nodes_api)];

                let mut count = 0;
                for (plugin_name, plugin_cell) in plugin_names.iter() {
                    if let Some(plugin) = Plugin::find(&core, plugin_name) {
                        let result = plugin.activate_future(PluginFeatures::ENABLED).await;
                        if result.is_err() {
                            pwvucontrol_critical!("Cannot activate plugin {plugin_name}");
                        } else {
                            plugin_cell.set(plugin).expect("Plugin not set");
                            pwvucontrol_info!("Activated plugin {plugin_name}");
                            count += 1;
                            if count == plugin_names.len() {
                                core.install_object_manager(&om);
                            }
                        }
                    } else {
                        pwvucontrol_critical!("Cannot find plugin {plugin_name}");
                        PwvucontrolApplication::default().quit();
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

            metadata_om.add_interest(
                [Constraint::compare(ConstraintType::PwGlobalProperty, "metadata.name", "default", true)]
                    .iter()
                    .collect::<Interest<wp::pw::Metadata>>(),
            );

            metadata_om.request_object_features(wp::pw::GlobalProxy::static_type(), wp::core::ObjectFeatures::ALL);

            metadata_om.connect_object_added(clone!(@weak self as imp, @weak wp_core as core => move |_, object| {
                if let Some(metadataobj) = object.dynamic_cast_ref::<wp::pw::Metadata>() {
                    pwvucontrol_info!("added metadata object: {:?}", metadataobj.bound_id());
                    imp.metadata.replace(Some(metadataobj.clone()));
                    for a in metadataobj.new_iterator(u32::MAX).expect("iterator") {
                        let (s, k, t, v) = wp::pw::Metadata::iterator_item_extract(&a);
                        pwvucontrol_info!("Metadata value: {s}, {k:?}, {t:?}, {v:?}");
                    }
                } else {
                    unreachable!("Object must be one of the above, but is {:?} instead", object.type_());
                }
            }));

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
    pub fn new () -> Self {
        glib::Object::builder()
            .build()
    }

    pub fn get_device_by_id(&self, id: u32) -> Option<PwDeviceObject> {
        let devicemodel = &self.imp().device_model;
        for device in devicemodel.iter::<PwDeviceObject>() {
            if let Ok(device) = device {
                if device.wpdevice().bound_id() == id {
                    return Some(device);
                }
            }
        }
        None
    }

    pub fn get_node_by_id(&self, id: u32) -> Option<PwNodeObject> {
        let nodemodel = &self.imp().node_model;
        for node in nodemodel.iter::<PwNodeObject>() {
            if let Ok(node) = node {
                if node.wpnode().bound_id() == id {
                    return Some(node);
                }
            }
        }
        None
    }
}

impl Default for PwvucontrolManager {
    fn default() -> Self {
        PwvucontrolApplication::default().manager()
    }
}