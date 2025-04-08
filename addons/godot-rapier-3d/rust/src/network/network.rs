use godot::prelude::*;

use crate::adapters::GR3DNetworkAdapter;
use crate::interface::GR3D;
use crate::network::*;

pub struct Network {
    pub started: bool,
    pub host_starting: bool,
    pub adapter: Option<Gd<GR3DNetworkAdapter>>,
    pub local_peer: Option<LocalPeer>,
    pub remote_peers: Vec<RemotePeer>,
}

impl Network {
    pub fn new() -> Self {
        Self {
            started: false,
            host_starting: false,
            adapter: None,
            local_peer: None,
            remote_peers: Vec::new(),
        }
    }

    pub fn get_adapter(&self) -> Option<&Gd<GR3DNetworkAdapter>> {
        match self.adapter {
            Some(ref adapter) => Some(adapter),
            None => {
                log::error!("Network adapter is not attached");
                None
            }
        }
    }

    pub fn get_remote_peer_mut(&mut self, peer_id: i64) -> Option<&mut RemotePeer> {
        match self
            .remote_peers
            .iter_mut()
            .find(|peer| peer.metadata.id == peer_id)
        {
            Some(peer) => Some(peer),
            None => {
                log::error!("Remote peer with ID {} not found", peer_id);
                None
            }
        }
    }

    pub fn on_received_ping(&mut self, peer_id: i64, origin_ts: GString) {}
    pub fn on_received_ping_back(&mut self, peer_id: i64, origin_ts: GString, remote_ts: GString) {}
    pub fn on_received_tick_data(&mut self, peer_id: i64, data: PackedByteArray) {
        // TODO - store tick data in remote peer buffer
    }
}

pub fn network_physics_process(gr3d: &mut GR3D) {
    // TODO
    record_all_advantages(gr3d, false);
    // local_peer.record_inputs()
    // send_updates_to_all_peers
}
