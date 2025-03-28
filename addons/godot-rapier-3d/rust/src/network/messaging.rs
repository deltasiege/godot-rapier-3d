use std::fmt::{Debug, Formatter};

use bincode::{
    config::standard,
    serde::{decode_from_slice, encode_to_vec},
};
use godot::prelude::*;
use rapier3d::parry::utils::hashmap::HashMap;
use serde::{Deserialize, Serialize};

use crate::{config::MAX_BUFFER_LEN, interface::GR3DNet};

// TODO need serialized buffer of all timestep -> local actions - should go in world buffer
// and separate HashMap of all timestep -> local state hashes - should go in world buffer

/// Sent over the network to all peers
#[derive(Serialize, Deserialize)]
pub struct UpdateMessage {
    pub tick: usize,                       // The tick that this message was created on
    pub actions: HashMap<usize, Vec<u8>>, // Tick -> serialized actions. All actions known to the sending peer for the given ticks
    pub state_hashes: HashMap<usize, u64>, // Tick -> state hash. All state hashes calculated by the sending peer for the given ticks
    pub next_action_tick_requested: usize, // The next action tick that the sending peer wants from the receiving peer
    pub next_hash_tick_requested: usize, // The next hash tick that the sending peer wants from the receiving peer
}

pub fn send_local_actions_to_all_peers(net: &mut GR3DNet) {
    let adapter = match &net.network_adapter {
        Some(adapter) => adapter,
        None => {
            log::error!("Network adapter not attached");
            return;
        }
    };

    for peer in net.peers.iter_mut() {
        let current_tick = net.tick;

        // Get all actions that are after the next tick that the peer has requested
        let mut actions_since_requested = HashMap::default();
        let clamped_start = clamp_start_tick(peer.next_local_action_tick_requested, current_tick);

        for tick in clamped_start..current_tick {
            if let Some(frame) = net.world_buffer.local_buffer.get(&tick) {
                if let Some(ser_actions) = &frame.ser_actions {
                    actions_since_requested.insert(tick, ser_actions.clone());
                }
            }
        }

        // Get all state hashes that are after the next tick that the peer has requested
        let mut hashes_since_requested = HashMap::default();
        let clamped_start = clamp_start_tick(peer.next_local_hash_tick_requested, current_tick);

        for tick in clamped_start..current_tick {
            if let Some(hash) = net.world_buffer.get_physics_state_hash(tick) {
                hashes_since_requested.insert(tick, hash);
            }
        }

        let message = UpdateMessage {
            tick: net.tick,
            actions: actions_since_requested,
            state_hashes: hashes_since_requested,
            next_action_tick_requested: peer.last_remote_action_tick_received + 1,
            next_hash_tick_requested: peer.last_remote_hash_tick_received + 1,
        };

        let ser_message = match encode_to_vec(message, standard()) {
            Ok(bytes) => bytes,
            Err(e) => {
                log::error!("Failed to encode update message. Error: {}", e);
                continue;
            }
        };

        peer.last_local_message_size = ser_message.len();
        let data = PackedByteArray::from(ser_message.as_slice());
        adapter.bind().send_tick_data(peer.peer_id, data);
    }
}

pub fn deserialize_message_from_peer(
    ser_message: PackedByteArray,
) -> Option<(UpdateMessage, usize)> {
    match decode_from_slice(ser_message.to_vec().as_slice(), standard()) {
        Ok(message) => Some(message),
        Err(e) => {
            log::error!("Failed to deserialize update message. Error: {}", e);
            None
        }
    }
}

impl Debug for UpdateMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UpdateMessage")
            .field("tick", &self.tick)
            .field("actions", &self.actions.len())
            .field("state_hashes", &self.state_hashes.len())
            .field(
                "next_action_tick_requested",
                &self.next_action_tick_requested,
            )
            .field("next_hash_tick_requested", &self.next_hash_tick_requested)
            .finish()
    }
}

// Starting tick cannot be MAX_BUFFER_LEN behind current tick, and cannot be greater than current tick
fn clamp_start_tick(requested: usize, current: usize) -> usize {
    let minimum_start = current.saturating_sub(MAX_BUFFER_LEN);
    let start_tick = std::cmp::max(minimum_start, requested + 1);
    std::cmp::min(start_tick, current)
}
