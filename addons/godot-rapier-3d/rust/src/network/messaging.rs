use bincode::{
    config::standard,
    serde::{decode_from_slice, encode_to_vec},
};
use godot::prelude::*;
use rapier3d::parry::utils::hashmap::HashMap;
use serde::{Deserialize, Serialize};

use super::{GR3DNetworkAdapter, Peer};
use crate::interface::GR3DNet;

// TODO need serialized buffer of all timestep -> local actions - should go in world buffer
// and separate HashMap of all timestep -> local state hashes - should go in world buffer

/// Sent over the network to all peers
#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateMessage {
    pub tick: usize, // The physics timestep that the attached data corresponds to
    pub actions: HashMap<usize, Vec<u8>>, // All actions known to the sending peer for this tick
    pub state_hashes: HashMap<usize, u64>, // Tick -> state hash. All state hashes beyond the one that receiving peer has asked for
    pub next_action_tick_requested: usize, // The next action tick that the sending peer wants from the receiving peer
    pub next_hash_tick_requested: usize, // The next hash tick that the sending peer wants from the receiving peer
}

pub fn send_local_actions_to_all_peers(net: &GR3DNet, adapter: &Gd<GR3DNetworkAdapter>) {
    for peer in &net.peers {
        let message = match create_update_message_for_peer(
            peer,
            net,
            peer.next_local_action_tick_requested,
            peer.next_local_hash_tick_requested,
        ) {
            Some(message) => message,
            None => continue,
        };
        let data = PackedByteArray::from(message.as_slice());
        adapter.bind().send_tick_data(peer.peer_id, data);
    }
}

fn create_update_message_for_peer(
    peer: &Peer,
    net: &GR3DNet,
    next_action_tick_requested: usize,
    next_hash_tick_requested: usize,
) -> Option<Vec<u8>> {
    let current_tick = net.tick;

    // Get all actions that are after the next tick that the peer has requested
    let mut actions_since_requested = HashMap::default();
    for tick in peer.next_local_action_tick_requested..current_tick {
        if let Some(frame) = net.world_buffer.local_buffer.get(&tick) {
            if let Some(ser_actions) = &frame.ser_actions {
                actions_since_requested.insert(tick, ser_actions.clone());
            }
        }
    }

    // Get all state hashes that are after the next tick that the peer has requested
    let mut hashes_since_requested = HashMap::default();
    for tick in peer.next_local_hash_tick_requested..current_tick {
        if let Some(hash) = net.world_buffer.get_physics_state_hash(tick) {
            hashes_since_requested.insert(tick, hash);
        }
    }

    let message = UpdateMessage {
        tick: net.tick,
        actions: actions_since_requested,
        state_hashes: hashes_since_requested,
        next_action_tick_requested,
        next_hash_tick_requested,
    };

    match encode_to_vec(message, standard()) {
        Ok(bytes) => Some(bytes),
        Err(e) => {
            log::error!("Failed to encode update message. Error: {}", e);
            None
        }
    }
}

pub fn deserialize_message_from_peer(ser_message: PackedByteArray) -> Option<UpdateMessage> {
    match decode_from_slice(ser_message.to_vec().as_slice(), standard()) {
        Ok(message) => Some(message.0),
        Err(e) => {
            log::error!("Failed to deserialize update message. Error: {}", e);
            None
        }
    }
}
