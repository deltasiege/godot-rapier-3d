use core::panic;

use godot::prelude::*;
use rapier3d::parry::utils::hashmap::HashMap;

use crate::adapters::GR3DInputAdapter;
use crate::network::*;
use crate::types::*;
use crate::utils::get_hash;

#[derive(Debug)]
pub struct LocalPeer {
    pub metadata: PeerMetadata,
    pub buffers: PeerBuffers,

    pub input_adapter: Option<Gd<GR3DInputAdapter>>, // The input adapter for this peer
    pub world_snapshots: HashMap<Tick, Vec<u8>>,     // The world snapshots of this peer
}

impl LocalPeer {
    pub fn new(metadata: PeerMetadata) -> Self {
        if metadata.idx.is_none() {
            let msg = format!(
                "Local peer index must be known on creation. Peer ID: {}",
                metadata.id
            );
            log::error!("{}", msg);
            panic!("{}", msg);
        }

        Self {
            metadata,
            buffers: PeerBuffers::default(),

            input_adapter: None,
            world_snapshots: HashMap::default(),
        }
    }

    pub fn record_world_snapshot(&mut self, tick: Tick, snapshot: Vec<u8>) {
        self.buffers.world_hashes.insert(tick, get_hash(&snapshot));
        self.world_snapshots.insert(tick, snapshot);
    }
}
