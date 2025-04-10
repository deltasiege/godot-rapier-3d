use godot::prelude::*;
use rapier3d::parry::utils::hashmap::HashMap;

use crate::adapters::GR3DInputAdapter;
use crate::network::*;
use crate::types::*;
use crate::utils::get_hash;

#[derive(Debug)]
pub struct LocalPeer {
    pub metadata: Option<PeerMetadata>,
    pub buffers: PeerBuffers,

    pub input_adapter: Option<Gd<GR3DInputAdapter>>, // The input adapter for this peer
    pub world_snapshots: HashMap<Tick, Vec<u8>>,     // The world snapshots of this peer
}

impl LocalPeer {
    pub fn new() -> Self {
        Self {
            metadata: None,
            buffers: PeerBuffers::default(),
            input_adapter: None,
            world_snapshots: HashMap::default(),
        }
    }

    pub fn get_adapter(&self) -> Option<&Gd<GR3DInputAdapter>> {
        match self.input_adapter {
            Some(ref adapter) => Some(adapter),
            None => {
                log::error!("Input adapter is not set");
                None
            }
        }
    }

    pub fn record_world_snapshot(&mut self, tick: Tick, snapshot: Vec<u8>) {
        self.buffers.world_hashes.insert(tick, get_hash(&snapshot));
        self.world_snapshots.insert(tick, snapshot);
    }
}
