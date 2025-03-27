use bincode::{
    config::standard,
    serde::{decode_from_slice, encode_to_vec},
};
use godot::prelude::*;
use rapier3d::parry::utils::hashmap::HashMap;
use serde::{Deserialize, Serialize};

use super::{GR3DNetworkAdapter, Peer};
use crate::{interface::GR3DSync, World};

// TODO need serialized buffer of all timestep -> local actions - should go in world buffer
// and separate HashMap of all timestep -> local state hashes - should go in world buffer

/// Sent over the network to all peers
#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateMessage {
    pub tick: u64,        // The physics timestep that the attached data corresponds to
    pub actions: Vec<u8>, // All actions known to the sending peer for this tick
    pub state_hashes: HashMap<u64, u64>, // Tick -> state hash. All state hashes beyond the one that receiving peer has asked for
    pub next_action_tick_requested: u64, // The next action tick that the sending peer wants from the receiving peer
    pub next_hash_tick_requested: u64, // The next hash tick that the sending peer wants from the receiving peer
}

pub fn send_local_actions_to_all_peers(
    peers: &Vec<Peer>,
    world: &World,
    network_adapter: &Gd<GR3DNetworkAdapter>,
) {
    for peer in peers {
        let message = match create_update_message_for_peer(
            peer,
            world,
            peer.next_local_action_tick_requested,
            peer.next_local_hash_tick_requested,
        ) {
            Some(message) => message,
            None => {
                log::error!("Failed to create update message for peer {}", peer.peer_id);
                continue;
            }
        };
        let data = PackedByteArray::from(message.as_slice());
        network_adapter.bind().send_tick_data(peer.peer_id, data);
    }
}

pub fn record_peer_tick_data(
    sync: &mut GR3DSync,
    sender_peer_id: i64,
    ser_message: PackedByteArray,
) {
    let message: UpdateMessage =
        match decode_from_slice(ser_message.to_vec().as_slice(), standard()) {
            Ok(message) => message.0,
            Err(e) => {
                log::error!("Failed to deserialize update message. Error: {}", e);
                return;
            }
        };

    let peer = match sync.peers.iter_mut().find(|p| p.peer_id == sender_peer_id) {
        Some(peer) => peer,
        None => {
            log::error!("Received tick data from unknown peer {}", sender_peer_id);
            return;
        }
    };

    peer.next_local_action_tick_requested = message.next_action_tick_requested;
    peer.next_local_hash_tick_requested = message.next_hash_tick_requested;
    peer.last_remote_action_tick_received = message.tick;
    peer.last_remote_hash_tick_received = message.tick;
}

fn create_update_message_for_peer(
    peer: &Peer,
    world: &World,
    next_action_tick_requested: u64,
    next_hash_tick_requested: u64,
) -> Option<Vec<u8>> {
    let tick = world.state.timestep_id;
    let bufferstep = world.buffer.get_step(tick as usize)?;
    let actions = bufferstep.ser_actions.clone()?;

    let message = UpdateMessage {
        tick: tick as u64,
        actions,
        state_hashes: get_state_hashes_for_peer(peer, world),
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

/// only give action complete state hashes?
fn get_state_hashes_for_peer(peer: &Peer, world: &World) -> HashMap<u64, u64> {
    let mut state_hashes = HashMap::default();
    let requested_tick = peer.next_local_hash_tick_requested;
    let mut idx = requested_tick;
    if idx >= (world.state.timestep_id - 1) as u64 {
        // TODO only go up to synchronized tick, not latest world tick
        log::error!("Requested state hash tick is beyond current tick");
        return state_hashes;
    }
    while idx < (world.state.timestep_id - 1) as u64 {
        match world.buffer.get_state_hash(idx as usize) {
            Some(hash) => {
                state_hashes.insert(idx, hash);
            }
            None => {}
        };
        idx += 1;
    }
    state_hashes
}

// TODO unneeded?
// Returns the earliest BufferStep that has a calculated physics state
// fn get_earliest_physics_step(buffer: &WorldBuffer) -> u64 {
//     let mut earliest = u64::MAX;
//     for (tick, step) in buffer.buffer.iter() {
//         if step.physics_state.is_some() {
//             earliest = *tick as u64;
//             break;
//         }
//     }
//     earliest
// }
