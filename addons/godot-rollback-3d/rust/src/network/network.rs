use godot::prelude::*;

use crate::adapters::GR3DNetworkAdapter;
use crate::interface::GR3D;
use crate::network::*;
use crate::types::*;
use crate::utils::decode_from_packed_byte_array;
use crate::utils::encode_to_packed_byte_array;
use crate::world::step;

#[derive(Debug)]
pub struct Network {
    pub started: bool,
    pub host_starting: bool,
    pub adapter: Option<Gd<GR3DNetworkAdapter>>,
    pub peer_map: Option<PeerMap>,
    pub local_peer: LocalPeer,
    pub remote_peers: Vec<RemotePeer>,
}

impl Network {
    pub fn new() -> Self {
        Self {
            started: false,
            host_starting: false,
            adapter: None,
            peer_map: None,
            local_peer: LocalPeer::new(),
            remote_peers: Vec::new(),
        }
    }

    pub fn add_remote_peer(&mut self, peer_id: i64) {
        self.remote_peers
            .push(RemotePeer::new(PeerMetadata::new(peer_id)));
        log::debug!("Added peer: {}", peer_id);
    }

    pub fn remove_remote_peer(&mut self, peer_id: i64) {
        self.remote_peers.retain(|peer| peer.metadata.id != peer_id);
        log::debug!("Removed peer: {}", peer_id);
    }

    pub fn send_updates_to_all_remote_peers(&self, current_tick: Tick) {
        if let Some(adapter) = self.get_adapter() {
            for remote_peer in &self.remote_peers {
                let update_message =
                    UpdateMessage::from_peers(&self.local_peer, &remote_peer, current_tick);

                log::trace!(
                    "Sending update message to peer {}: {}",
                    remote_peer.metadata.id,
                    update_message
                );

                adapter.bind().send_tick_data(
                    remote_peer.metadata.id,
                    encode_to_packed_byte_array(&update_message),
                );
            }
        }
    }

    pub fn log_buffer_holes(&self) {
        for remote_peer in &self.remote_peers {
            log::trace!(
                "Remote peer {}: buffer holes: {:?}",
                remote_peer.metadata.id,
                remote_peer.buffers.get_holes()
            );
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

    pub fn get_adapter_mut(&mut self) -> Option<&mut Gd<GR3DNetworkAdapter>> {
        match self.adapter {
            Some(ref mut adapter) => Some(adapter),
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
}

pub fn on_physics_process(gr3d: &mut GR3D, step_world: bool) {
    if !gr3d.network.started {
        return;
    }
    let tick = gr3d.world.time.tick.clone();
    record_all_advantages(gr3d, false);
    capture_current_inputs(
        &mut gr3d.network.local_peer.buffers,
        tick,
        &mut gr3d.network.local_peer.input_adapter,
    );

    if step_world {
        step(gr3d, 1);
    }

    gr3d.network.send_updates_to_all_remote_peers(tick);
    gr3d.network.log_buffer_holes();
}

pub fn on_received_tick_data(gr3d: &mut GR3D, peer_id: i64, data: PackedByteArray) -> Option<()> {
    let update_message = decode_from_packed_byte_array::<UpdateMessage>(&data)?;
    let mut input_adapter = gr3d.network.local_peer.input_adapter.clone()?;
    let peer = gr3d.network.get_remote_peer_mut(peer_id)?;
    peer.record_update_message(&update_message, &mut input_adapter);
    Some(())
}
