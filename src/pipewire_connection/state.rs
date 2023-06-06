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

use std::collections::HashMap;

use crate::MediaType;

/// Any pipewire item we need to keep track of.
/// These will be saved in the `State` struct associated with their id.
pub(super) enum Item {
    Node {
        // Keep track of the nodes media type to color ports on it.
        media_type: Option<MediaType>,
    },
    Port {
        // Save the id of the node this is on so we can remove the port from it
        // when it is deleted.
        node_id: u32,
    },
    Link {
        port_from: u32,
        port_to: u32,
    },
}

/// This struct keeps track of any relevant items and stores them under their IDs.
///
/// Given two port ids, it can also efficiently find the id of the link that connects them.
#[derive(Default)]
pub(super) struct State {
    /// Map pipewire ids to items.
    items: HashMap<u32, Item>,
    /// Map `(output port id, input port id)` tuples to the id of the link that connects them.
    links: HashMap<(u32, u32), u32>,
}

impl State {
    /// Create a new, empty state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new item under the specified id.
    pub fn insert(&mut self, id: u32, item: Item) {
        if let Item::Link {
            port_from, port_to, ..
        } = item
        {
            self.links.insert((port_from, port_to), id);
        }

        self.items.insert(id, item);
    }


    /// Get the item that has the specified id.
    pub fn get(&self, id: u32) -> Option<&Item> {
        self.items.get(&id)
    }

    /// Get the id of the link that links the two specified ports.
    pub fn get_link_id(&self, output_port: u32, input_port: u32) -> Option<u32> {
        self.links.get(&(output_port, input_port)).copied()
    }

    /// Remove the item with the specified id, returning it if it exists.
    pub fn remove(&mut self, id: u32) -> Option<Item> {
        let removed = self.items.remove(&id);

        if let Some(Item::Link { port_from, port_to }) = removed {
            self.links.remove(&(port_from, port_to));
        }

        removed
    }

    /// Convenience function: Get the id of the node a port is on
    pub fn get_node_of_port(&self, port: u32) -> Option<u32> {
        if let Some(Item::Port { node_id }) = self.get(port) {
            Some(*node_id)
        } else {
            None
        }
    }
}
