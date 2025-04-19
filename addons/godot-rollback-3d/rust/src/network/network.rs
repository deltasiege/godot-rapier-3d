use godot::prelude::*;

use crate::adapters::GR3DNetworkAdapter;
use crate::config::MAX_BUFFER_LEN;
use crate::interface::*;
use crate::network::*;
use crate::types::*;
use crate::utils::*;
use crate::world::*;

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

    pub fn add_remote_peer(&mut self, peer_id: PeerId) {
        self.remote_peers
            .push(RemotePeer::new(PeerMetadata::new(peer_id)));
        log::debug!("Added peer: {}", peer_id);
    }

    pub fn remove_remote_peer(&mut self, peer_id: PeerId) {
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

    pub fn get_remote_peer(&self, peer_id: PeerId) -> Option<&RemotePeer> {
        match self
            .remote_peers
            .iter()
            .find(|peer| peer.metadata.id == peer_id)
        {
            Some(peer) => Some(peer),
            None => {
                log::error!("Remote peer with ID {} not found", peer_id);
                None
            }
        }
    }

    pub fn get_remote_peer_mut(&mut self, peer_id: PeerId) -> Option<&mut RemotePeer> {
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

    /// Returns a specific input made by the remote peer with the given PeerId at the given tick.
    pub fn get_remote_input(
        &self,
        peer_id: PeerId,
        tick: Tick,
        input_key: GString,
    ) -> Option<Variant> {
        self.get_remote_inputs(peer_id, tick)?
            .get(&input_key)
            .cloned()
    }

    /// Returns all inputs made by the remote peer with the given PeerId at the given tick.
    fn get_remote_inputs(&self, peer_id: PeerId, tick: Tick) -> Option<InputMap> {
        self.get_remote_peer(peer_id)?
            .buffers
            .inputs
            .get(&tick)
            .cloned()
    }

    /// Returns the PeerIndex of the given PeerId.
    pub fn get_peer_index(&self, peer_id: PeerId) -> Option<PeerIndex> {
        if self.local_peer.get_peer_id() == Some(peer_id) {
            return self.local_peer.get_peer_index();
        }

        for remote_peer in &self.remote_peers {
            if remote_peer.metadata.id == peer_id {
                return remote_peer.metadata.idx;
            }
        }

        None
    }

    /// Returns the PeerId of the given PeerIndex.
    pub fn get_peer_id(&self, peer_index: PeerIndex) -> Option<PeerId> {
        if self.local_peer.get_peer_index() == Some(peer_index) {
            return self.local_peer.get_peer_id();
        }

        for remote_peer in &self.remote_peers {
            if remote_peer.metadata.idx == Some(peer_index) {
                return Some(remote_peer.metadata.id);
            }
        }

        None
    }
}

pub fn on_physics_process(gr3d: &mut GR3D, runtime: Gd<Node>, step_world: bool) {
    if !gr3d.network.started {
        return;
    }
    let tick = gr3d.world.time.tick.clone();

    process_rollbacks(gr3d); // Perform any required rollbacks
    record_all_advantages(gr3d, false); // Record all peer advantages

    // Capture inputs the local peer is making at the current tick
    capture_current_inputs(
        &mut gr3d.network.local_peer.buffers,
        tick,
        &mut gr3d.network.local_peer.input_adapter,
    );

    if step_world {
        step(gr3d, 1); // Step the world by one tick if requested
    }

    process_godot_actions(gr3d, runtime); // Add/remove Godot nodes to ensure sync with Rapier world
    gr3d.network.send_updates_to_all_remote_peers(tick);
    gr3d.network.log_buffer_holes();
}

pub fn on_received_tick_data(
    gr3d: &mut GR3D,
    peer_id: PeerId,
    data: PackedByteArray,
) -> Option<()> {
    let update_message = decode_from_packed_byte_array::<UpdateMessage>(&data)?;
    let mut input_adapter = gr3d.network.local_peer.input_adapter.clone()?;
    let peer = gr3d.network.get_remote_peer_mut(peer_id)?;
    peer.record_update_message(&update_message, &mut input_adapter);

    // Compare newly inserted remote_peer.buffers.input_hashes to dirty remote_peer.combined_input_hashes
    detect_missed_predictions(peer, &mut gr3d.rollback_state.invalid_ticks);

    let predict_until_tick = gr3d.world.time.tick + (MAX_BUFFER_LEN as u64);
    peer.predict_inputs_until_tick(predict_until_tick, &mut input_adapter);

    // TODO this could potentially happen at the same time if multiple remote peers land packets at same time
    // - calls to input adapter need to be deferred?

    Some(())
}

/// Compare dirty version of combined_input_hashes that is yet to be recaculated with freshly known input_hashes of a given remote peer.
/// Raise rollback flags if the hashes differ.
fn detect_missed_predictions(remote_peer: &RemotePeer, invalid_ticks: &mut Vec<Tick>) {
    for (tick, (hash, was_predicted)) in remote_peer.combined_input_hashes.iter() {
        if !*was_predicted {
            continue;
        }

        if let Some(input_hash) = remote_peer.buffers.input_hashes.get(tick) {
            if *hash != *input_hash {
                log::debug!(
                    "Missed prediction for peer_id {} at tick {}: {} != {}",
                    remote_peer.metadata.id,
                    tick,
                    hash,
                    input_hash
                );
                invalid_ticks.push(*tick);
            }
        }
    }
}
