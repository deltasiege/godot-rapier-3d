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
    pub world_snapshots: HashMap<Tick, Vec<u8>>, // The world snapshots of this peer
    pub input_adapter: Option<Gd<GR3DInputAdapter>>, // The input adapter for this peer
}

impl LocalPeer {
    pub fn new() -> Self {
        Self {
            metadata: None,
            buffers: PeerBuffers::default(),
            world_snapshots: HashMap::default(),
            input_adapter: None,
        }
    }

    /// Returns true if the provided GRUID refers to a local rollback node.
    pub fn is_local_gruid(&self, gruid: GRUID) -> bool {
        self.is_local_peer_idx(gruid.0)
    }

    /// Returns true if the provided PeerIndex matches the local PeerIndex.
    pub fn is_local_peer_idx(&self, peer_index: PeerIndex) -> bool {
        self.get_peer_index() == Some(peer_index)
    }

    /// Returns true if the provided PeerId matches the local PeerId.
    pub fn is_local_peer_id(&self, peer_id: PeerId) -> bool {
        self.get_peer_id() == Some(peer_id)
    }

    /// Returns the peer ID of this local peer.
    pub fn get_peer_id(&self) -> Option<PeerId> {
        self.get_metadata().map(|metadata| metadata.id)
    }

    /// Returns the peer index of this local peer.
    pub fn get_peer_index(&self) -> Option<PeerIndex> {
        self.get_metadata().map(|metadata| metadata.idx)?
    }

    /// Returns the metadata of this local peer, or errors if not set.
    fn get_metadata(&self) -> Option<&PeerMetadata> {
        match self.metadata {
            Some(ref metadata) => Some(metadata),
            None => {
                log::error!("Local peer metadata is not set");
                None
            }
        }
    }

    /// Returns the input adapter of this local peer, or errors if not set.
    pub fn get_adapter(&self) -> Option<&Gd<GR3DInputAdapter>> {
        match self.input_adapter {
            Some(ref adapter) => Some(adapter),
            None => {
                log::error!("Input adapter is not set");
                None
            }
        }
    }

    /// Returns mutable version of the input adapter of this local peer, or errors if not set.
    pub fn get_adapter_mut(&mut self) -> Option<&mut Gd<GR3DInputAdapter>> {
        match self.input_adapter {
            Some(ref mut adapter) => Some(adapter),
            None => {
                log::error!("Input adapter is not set");
                None
            }
        }
    }

    /// Record the given serialized WorldSnapshot for the given tick.
    pub fn record_world_snapshot(&mut self, tick: Tick, snapshot: Vec<u8>) {
        self.buffers.world_hashes.insert(tick, get_hash(&snapshot));
        self.world_snapshots.insert(tick, snapshot);
    }
}
