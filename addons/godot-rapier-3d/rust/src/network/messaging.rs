use std::fmt::{Display, Formatter};

use bincode::{
    config::standard,
    serde::{decode_from_slice, encode_to_vec},
};
use godot::prelude::*;
use rapier3d::parry::utils::hashmap::HashMap;
use serde::{Deserialize, Serialize};

use super::PeerBufferFrame;
use crate::interface::GR3DNet;

// TODO need serialized buffer of all timestep -> local actions - should go in world buffer
// and separate HashMap of all timestep -> local state hashes - should go in world buffer

/// Sent over the network to all peers
#[derive(Serialize, Deserialize, Clone)]
pub struct UpdateMessage {
    pub tick: usize, // The tick that this message was created on

    pub frames: Vec<PeerBufferFrame>, // All frames that the receiving peer has not acknowledged yet
    pub node_cache: HashMap<u32, String>, // Cache index -> node cuid as recorded by the sending peer (acknowledges that previous node cuids have been saved)

    /// Array of ticks that the sending peer wants from the receiving peer.
    /// Ticks after the latest entry in the array should still be sent if available
    /// Any ticks that are missing from this array (prior to the latest entry) - means that the sending peer has acknowledged them and they don't need to be resent
    pub requested_ticks: Vec<usize>,
}

pub fn send_local_ticks_to_all_peers(net: &mut GR3DNet) {
    let adapter = match &net.network_adapter {
        Some(adapter) => adapter.clone(),
        None => {
            log::error!("Network adapter not attached");
            return;
        }
    };

    for peer in net.peers.iter() {
        let last_tick = net.tick - 1;

        // Get all frames requested by the peer
        let mut requested_frames_and_beyond = Vec::new();
        for tick in peer.get_requested_local_ticks_and_beyond(last_tick) {
            if let Some(frame) = net.world_buffer.local_buffer.get(&tick) {
                let ser_actions = match frame.get_serialized_actions(&mut net.node_cache) {
                    Some(actions) => actions,
                    None => {
                        log::error!("Failed to serialize actions for tick {}", tick);
                        continue;
                    }
                };

                requested_frames_and_beyond.push(PeerBufferFrame::new(
                    tick,
                    ser_actions,
                    frame.physics_hash,
                ));
            } else {
                log::error!(
                    "Peer requested tick {} that doesn't exist in local_buffer",
                    tick
                ); // This can happen if local_buffer is pruned before sync is achieved,
                   // or a recent tick failed to be recorded in the local_buffer and the peer is asking for it
            }
        }

        let message = UpdateMessage {
            tick: last_tick,
            frames: requested_frames_and_beyond,
            requested_ticks: peer.get_ticks_to_request_and_beyond(last_tick),
        };

        let ser_message = match encode_to_vec(message.clone(), standard()) {
            Ok(bytes) => bytes,
            Err(e) => {
                log::error!("Failed to encode update message. Error: {}", e);
                continue;
            }
        };

        log::trace!(
            "Sending update message to peer ({} bytes): {}",
            ser_message.len(),
            message
        );
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

impl Display for UpdateMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "UpdateMessage {{ tick: {}, frames: {:?}, requested_ticks: {:?} }}",
            self.tick,
            self.frames
                .clone()
                .into_iter()
                .map(|frame| frame.tick)
                .collect::<Vec<usize>>(),
            self.requested_ticks
        )
    }
}
