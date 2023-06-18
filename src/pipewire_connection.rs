// Copyright 2021 Tom A. Wagner <tom.a.wagner@protonmail.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3 as published by
// the Free Software Foundation.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.
//
// SPDX-License-Identifier: GPL-3.0-only

mod state;

use std::{
    cell::RefCell,
    collections::HashMap,
    io,
    rc::Rc,
};

use gtk::glib::{self, clone};
use log::{debug, info, warn};
use pipewire::{
    link::{Link, LinkChangeMask, LinkListener, LinkState},
    node::{Node, NodeListener},
    prelude::*,
    properties,
    registry::{GlobalObject, Registry},
    spa::{Direction, ForeignDict},
    types::ObjectType,
    Context, Core, MainLoop,
};

use pipewire::spa::pod::deserialize::{
    DeserializeError, DeserializeSuccess, ObjectPodDeserializer, PodDeserialize, PodDeserializer,
    Visitor,
};
use pipewire::spa::pod::serialize::{GenError, PodSerialize, PodSerializer, SerializeSuccess};

use crate::{GtkMessage, MediaType, NodeType, PipewireMessage};
use state::{Item, State};

enum ProxyItem {
    Link {
        _proxy: Link,
        _listener: LinkListener,
    },
    Node {
        _proxy: Node,
        _listener: NodeListener,
    },
}


#[derive(Clone)]
struct PwConnection {
    mainloop: MainLoop,
    core: Rc<Core>,
    registry: Rc<Registry>,
    proxies: Rc<RefCell<HashMap<u32, ProxyItem>>>,
    state: Rc<RefCell<State>>,
    gtk_sender: Rc<glib::Sender<PipewireMessage>>,

}

impl PwConnection {
    pub fn new(
        mainloop: MainLoop,
        core: Rc<Core>,
        registry: Rc<Registry>,
        proxies: Rc<RefCell<HashMap<u32, ProxyItem>>>,
        state: Rc<RefCell<State>>,
        gtk_sender: Rc<glib::Sender<PipewireMessage>>,
    ) -> PwConnection {
        PwConnection {
            mainloop,
            core,
            registry,
            proxies,
            state,
            gtk_sender,
        }
    }

    pub fn handle_gtk_message(&self, msg: GtkMessage) {
        match msg {
            GtkMessage::ToggleLink { port_from, port_to } => self.toggle_link(port_from, port_to),
            GtkMessage::Terminate => self.mainloop.quit(),
            GtkMessage::SetVolume{id, channel_volumes, volume, mute} => self.set_volume(id, channel_volumes, volume, mute),
        }
    }

    pub fn handle_global(&self, global: &GlobalObject<ForeignDict>) {
        match global.type_ {
            ObjectType::Node => self.handle_node(global),
            ObjectType::Port => self.handle_port(global),
            ObjectType::Link => self.handle_link(global),
            _ => {
                // Other objects are not interesting to us
            }
        }
    }

    pub fn handle_global_remove(&self, id: u32) {

        if let Some(item) = self.state.borrow_mut().remove(id) {
            self.gtk_sender.send(match item {
                Item::Node { .. } => PipewireMessage::NodeRemoved {id},
                Item::Port { node_id } => PipewireMessage::PortRemoved {id, node_id},
                Item::Link { .. } => PipewireMessage::LinkRemoved {id},
            }).expect("Failed to send message");
        } else {
            // warn!(
            //     "Attempted to remove item with id {} that is not saved in state",
            //     id
            // );
        }

        self.proxies.borrow_mut().remove(&id);
    }

    fn set_volume(
        &self,
        id: u32,
        channel_volumes: Option<Vec<f32>>,
        volume: Option<f32>,
        mute: Option<bool>,
    ) {
        let mut buf = io::Cursor::new(Vec::new());
        let p = FormatProps {
            channel_volumes,
            mute,
            volume,
        };
        if let Ok(x) = PodSerializer::serialize(&mut buf, &p) {
            let proxies = self.proxies.borrow_mut();
    
            if let Some(ProxyItem::Node { _proxy: node, .. }) = proxies.get(&id) {
                node.set_param(pipewire::spa::param::ParamType::Props, 0, &x.0.get_ref());
            }
    
        } else {
            log::error!("Cannot serialize SomeProps");
        }
    }

    /// Handle a new node being added
    fn handle_node(
        &self,
        node: &GlobalObject<ForeignDict>,
    ) {
        debug!(
            "New node (id:{}) appeared, setting up node listener.",
            node.id
        );

        let node_id = node.id.clone();

        let proxy: pipewire::node::Node = self.registry.bind(node).expect("Failed to bind to node proxy");

        let listener = proxy
            .add_listener_local()
            .info(clone!(@strong self.proxies as proxies, @strong self.gtk_sender as sender => move |ni| {
                if ni.change_mask().contains(pipewire::node::NodeChangeMask::PARAMS) {
                    let proxies = proxies.borrow_mut();

                    if let Some(ProxyItem::Node { _proxy: hi, .. }) = proxies.get(&ni.id()) {
                        hi.enum_params(0, Some(pipewire::spa::param::ParamType::Props), 0, u32::MAX);
                        hi.enum_params(0, Some(pipewire::spa::param::ParamType::Format), 0, u32::MAX);
                    }
                }

                if ni.change_mask().contains(pipewire::node::NodeChangeMask::PROPS) {
                    let dict = ni.props().expect("Cannot get props from node info");
                    let mut props: HashMap<String, String> = HashMap::new();

                    for (key,value) in dict.iter() {
                        props.insert(key.to_string(), value.to_string());
                    }

                    sender.send(PipewireMessage::NodeProps{
                        id: node_id,
                        props: props,
                        }).expect("Failed to send NodeProps message");
                }
            }))
            .param(clone!(@strong self.gtk_sender as sender => move |_seq, _id, _start, _num, param| {
                if _id == pipewire::spa::param::ParamType::Format {

                    let audio_info = unsafe {
                        let mut audio_info: pipewire::spa::sys::spa_audio_info_raw = std::mem::zeroed();

                        pipewire::spa::sys::spa_format_audio_raw_parse(
                            param.as_ptr() as *const pipewire::spa::sys::spa_pod,
                            &mut audio_info as *mut pipewire::spa::sys::spa_audio_info_raw 
                        );
                        audio_info
                    };
                    sender.send(PipewireMessage::NodeFormat{
                        id: node_id,
                        channels: audio_info.channels,
                        rate: audio_info.rate,
                        format: audio_info.format,
                        position: audio_info.position,
                        })
                        .expect("Failed to send NodeFormat message");

                    return;
                }

                if _id == pipewire::spa::param::ParamType::Props { 
                
                    let (_, x) = PodDeserializer::deserialize_from::<FormatProps>(&param).expect("Error deserializing into Value");
                    if let Some(channel_volumes) = x.channel_volumes {
                        sender.send(PipewireMessage::NodeParam{
                            id: node_id,
                            param: crate::ParamType::ChannelVolumes(channel_volumes.clone())})
                            .expect("Failed to send ChannelVolumes message");
                    }

                    if let Some(volume) = x.volume {
                        sender.send(PipewireMessage::NodeParam{
                            id: node_id,
                            param: crate::ParamType::Volume(volume.clone())})
                            .expect("Failed to send Volume message");
                    }

                    if let Some(mute) = x.mute {
                        sender.send(PipewireMessage::NodeParam{
                            id: node_id,
                            param: crate::ParamType::Mute(mute.clone())})
                            .expect("Failed to send Mute message");
                    }
                }
            }))
            .register();

        self.proxies.borrow_mut().insert(
            node.id,
            ProxyItem::Node {
                _proxy: proxy,
                _listener: listener,
            },
        );

        let props = node
            .props
            .as_ref()
            .expect("Node object is missing properties");

        // Get the nicest possible name for the node, using a fallback chain of possible name attributes.
        let name = String::from(
            props
                .get("node.description")
                .or_else(|| props.get("node.nick"))
                .or_else(|| props.get("node.name"))
                .unwrap_or_default(),
        );

        // FIXME: Instead of checking these props, the "EnumFormat" parameter should be checked instead.
        let media_type = props.get("media.class").and_then(|class| {
            if class.contains("Audio") {
                Some(MediaType::Audio)
            } else if class.contains("Video") {
                Some(MediaType::Video)
            } else if class.contains("Midi") {
                Some(MediaType::Midi)
            } else {
                None
            }
        });

        let media_class = |class: &str| {
            if class.contains("Sink") {
                Some(NodeType::Sink)
            } else if class.contains("Output") {
                Some(NodeType::Output)
            } else if class.contains("Input") {
                Some(NodeType::Input)
            } else if class.contains("Source") {
                Some(NodeType::Source)
            } else {
                None
            }
        };

        let node_type = props
            .get("media.category")
            .and_then(|class| {
                if class.contains("Duplex") {
                    None
                } else {
                    props.get("media.class").and_then(media_class)
                }
            })
            .or_else(|| props.get("media.class").and_then(media_class));

        self.state.borrow_mut().insert(
            node.id,
            Item::Node {
                // widget: node_widget,
                media_type,
            },
        );

        self.gtk_sender
            .send(PipewireMessage::NodeAdded {
                id: node.id,
                name,
                node_type,
            })
            .expect("Failed to send message");
    }

    /// Handle a new port being added
    fn handle_port(
        &self,
        port: &GlobalObject<ForeignDict>,
    ) {
        let props = port
            .props
            .as_ref()
            .expect("Port object is missing properties");
        let name = props.get("port.name").unwrap_or_default().to_string();
        let node_id: u32 = props
            .get("node.id")
            .expect("Port has no node.id property!")
            .parse()
            .expect("Could not parse node.id property");
        let direction = if matches!(props.get("port.direction"), Some("in")) {
            Direction::Input
        } else {
            Direction::Output
        };

        // Find out the nodes media type so that the port can be colored.
        let media_type = if let Some(Item::Node { media_type, .. }) = self.state.borrow().get(node_id) {
            media_type.to_owned()
        } else {
            warn!("Node not found for Port {}", port.id);
            None
        };

        // Save node_id so we can delete this port easily.
        self.state.borrow_mut().insert(port.id, Item::Port { node_id });

        self.gtk_sender
            .send(PipewireMessage::PortAdded {
                id: port.id,
                node_id,
                name,
                direction,
                media_type,
            })
            .expect("Failed to send message");
    }


    /// Toggle a link between the two specified ports.
    fn toggle_link(
        &self,
        port_from: u32,
        port_to: u32,
    ) {
        let state = self.state.borrow_mut();
        if let Some(id) = state.get_link_id(port_from, port_to) {
            info!("Requesting removal of link with id {}", id);

            // FIXME: Handle error
            self.registry.destroy_global(id);
        } else {
            info!(
                "Requesting creation of link from port id:{} to port id:{}",
                port_from, port_to
            );

            let node_from = state
                .get_node_of_port(port_from)
                .expect("Requested port not in state");
            let node_to = state
                .get_node_of_port(port_to)
                .expect("Requested port not in state");

            if let Err(e) = self.core.create_object::<Link, _>(
                "link-factory",
                &properties! {
                    "link.output.node" => node_from.to_string(),
                    "link.output.port" => port_from.to_string(),
                    "link.input.node" => node_to.to_string(),
                    "link.input.port" => port_to.to_string(),
                    "object.linger" => "1"
                },
            ) {
                warn!("Failed to create link: {}", e);
            }
        }
    }



    /// Handle a new link being added
    fn handle_link(
        &self,
        link: &GlobalObject<ForeignDict>,
    ) {
        debug!(
            "New link (id:{}) appeared, setting up info listener.",
            link.id
        );

        let proxy: Link = self.registry.bind(link).expect("Failed to bind to link proxy");
        let listener = proxy
            .add_listener_local()
            .info(clone!(@strong self.state as state, @strong self.gtk_sender as sender => move |info| {
                debug!("Received link info: {:?}", info);

                let id = info.id();

                let mut state = state.borrow_mut();
                if let Some(Item::Link { .. }) = state.get(id) {
                    // Info was an update - figure out if we should notify the gtk thread
                    if info.change_mask().contains(LinkChangeMask::STATE) {
                        sender.send(PipewireMessage::LinkStateChanged {
                            id,
                            active: matches!(info.state(), LinkState::Active)
                        }).expect("Failed to send message");
                    }
                    // TODO -- check other values that might have changed
                } else {
                    // First time we get info. We can now notify the gtk thread of a new link.
                    let node_from = info.output_node_id();
                    let port_from = info.output_port_id();
                    let node_to = info.input_node_id();
                    let port_to = info.input_port_id();

                    state.insert(id, Item::Link {
                        port_from, port_to
                    });

                    sender.send(PipewireMessage::LinkAdded {
                        id,
                        node_from,
                        port_from,
                        node_to,
                        port_to,
                        active: matches!(info.state(), LinkState::Active)
                    }).expect(
                        "Failed to send message"
                    );
                }
            }))
            .register();

        self.proxies.borrow_mut().insert(
            link.id,
            ProxyItem::Link {
                _proxy: proxy,
                _listener: listener,
            },
        );
    }
}



/// The "main" function of the pipewire thread.
pub(super) fn thread_main(
    gtk_sender: glib::Sender<PipewireMessage>,
    pw_receiver: pipewire::channel::Receiver<GtkMessage>,
) {
    let mainloop = MainLoop::new().expect("Failed to create mainloop");
    let context = Context::new(&mainloop).expect("Failed to create context");
    let core = Rc::new(context.connect(None).expect("Failed to connect to remote"));
    let registry = Rc::new(core.get_registry().expect("Failed to get registry"));

    // Keep proxies and their listeners alive so that we can receive info events.
    let proxies = Rc::new(RefCell::new(HashMap::new()));

    let state = Rc::new(RefCell::new(State::new()));

    let pwconn = PwConnection::new(
        mainloop.clone(),
        core,
        registry.clone(),
        proxies,
        state,
        Rc::new(gtk_sender));

    // let pwconn = Rc::new(RefCell::new(pwconn));

    let _receiver = pw_receiver.attach(&mainloop, 
        clone!(@strong pwconn => move |msg| pwconn.handle_gtk_message(msg)
    ));

    let _listener = registry
        .add_listener_local()
        .global(clone!(@strong pwconn => move |global| {
            pwconn.handle_global(global);
            }
        ))
        .global_remove(clone!(@strong pwconn => move |id| {
            pwconn.handle_global_remove(id);
        }))
        .register();

    mainloop.run();
}

//#[derive(Default)]
struct FormatProps {
    volume: Option<f32>,
    mute: Option<bool>,
    channel_volumes: Option<Vec<f32>>,
}

const PROPS_KEY_CHANNEL_VOLUMES: u32 = 65544;
const PROPS_KEY_VOLUME: u32 = 65539;
const PROPS_KEY_MUTE: u32 = 65540;

impl<'de> PodDeserialize<'de> for FormatProps {
    fn deserialize(
        deserializer: PodDeserializer<'de>,
    ) -> Result<(Self, DeserializeSuccess<'de>), DeserializeError<&'de [u8]>>
    where
        Self: Sized,
    {
        use pipewire::spa::pod::{Value, ValueArray};
        struct ObjectVisitor;

        impl<'de> Visitor<'de> for ObjectVisitor {
            type Value = FormatProps;
            type ArrayElem = std::convert::Infallible;

            fn visit_object(
                &self,
                object_deserializer: &mut ObjectPodDeserializer<'de>,
            ) -> Result<Self::Value, DeserializeError<&'de [u8]>> {
                let mut tmp = FormatProps {
                    channel_volumes: None,
                    volume: None,
                    mute: None,
                };

                while let Some((value, key, _flags)) =
                    object_deserializer.deserialize_property::<Value>()?
                {
                    match key {
                        PROPS_KEY_CHANNEL_VOLUMES => {
                            if let Value::ValueArray(ValueArray::Float(channel_volumes)) = &value {
                                tmp.channel_volumes = Some(channel_volumes.clone());
                            }
                        }
                        PROPS_KEY_VOLUME => {
                            if let Value::Float(volume) = &value {
                                tmp.volume = Some(volume.clone());
                            }
                        }
                        PROPS_KEY_MUTE => {
                            if let Value::Bool(mute) = &value {
                                tmp.mute = Some(mute.clone());
                            }
                        }
                        _ => {}
                    }
                }

                Ok(tmp)
            }
        }

        deserializer.deserialize_object(ObjectVisitor)
    }
}

impl PodSerialize for FormatProps {
    fn serialize<O: io::Write + io::Seek>(
        &self,
        serializer: PodSerializer<O>,
    ) -> Result<SerializeSuccess<O>, GenError> {
        use pipewire::spa::pod::{Value, ValueArray};
        const OBJECT_TYPE_PROPS: u32 = 262146;
        const OBJECT_ID_PROPS: u32 = 2;

        let mut object_serializer =
            serializer.serialize_object(OBJECT_TYPE_PROPS, OBJECT_ID_PROPS)?;
        if let Some(channel_volumes) = &self.channel_volumes {
            object_serializer.serialize_property(
                PROPS_KEY_CHANNEL_VOLUMES,
                &Value::ValueArray(ValueArray::Float(channel_volumes.clone())),
                pipewire::spa::pod::PropertyFlags::empty(),
            )?;
        }

        if let Some(volume) = &self.volume {
            object_serializer.serialize_property(
                PROPS_KEY_VOLUME,
                &Value::Float(*volume),
                pipewire::spa::pod::PropertyFlags::empty(),
            )?;
        }

        if let Some(mute) = &self.mute {
            object_serializer.serialize_property(
                PROPS_KEY_MUTE,
                &Value::Bool(*mute),
                pipewire::spa::pod::PropertyFlags::empty(),
            )?;
        }

        object_serializer.end()
    }
}
