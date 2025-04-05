use godot::prelude::*;
use rapier3d::parry::utils::hashmap::HashMap;

use super::deserialize_message_from_peer;
use crate::actions::serde::deserialize_actions;
use crate::config::MAX_MESSAGE_SIZE;
use crate::nodes::*;
use crate::utils::get_hash;
use crate::{actions::Operation, interface::GR3DNet, Action};

/// Deserializes a remote peer update message and records relevant data against the peer
pub fn ingest_peer_message(
    net: &mut GR3DNet,
    sender_peer_id: i64,
    ser_message: PackedByteArray,
    scene_root: Gd<Node>,
) {
    let num_peers = net.peers.len();
    let peer = match net.peers.iter_mut().find(|p| p.peer_id == sender_peer_id) {
        Some(peer) => peer,
        None => {
            log::error!("Received tick data from unknown peer {}", sender_peer_id);
            return;
        }
    };

    let (message, message_length) = match deserialize_message_from_peer(ser_message) {
        Some(message) => message,
        None => return,
    };

    log::log!(
        if message_length > MAX_MESSAGE_SIZE {
            log::Level::Warn
        } else {
            log::Level::Trace
        },
        "Received message from peer {} of size {}: {}",
        peer.peer_id,
        message_length,
        message
    );

    /*
        Overwrite actions in the peer's buffer with actions in the message
        Raise rollback flags if changes are made to the peer's buffer

        Record the peer as having sent actions on the given tick
        and update the action complete tick if all peers have sent actions.
    */
    for frame in message.frames {
        let tick = frame.tick;
        let deserialized_actions = deserialize_actions(frame.ser_actions.clone(), &scene_root);
        let actions_hash = get_hash(&frame.ser_actions);
        let mut physics_hash = None;

        if let Some(existing_frame) = peer.received_remote_ticks.get_mut(&tick) {
            log::trace!(
                "Received extra frame from peer {} for tick {}: {}",
                peer.peer_id,
                tick,
                frame
            );

            // Peer already gave us a frame for this tick, check if everything matches
            let existing_actions_hash = get_hash(&existing_frame.ser_actions);
            if existing_actions_hash != actions_hash {
                log::error!(
                    "Received conflicting actions from peer {} for tick {}: {} vs {}",
                    peer.peer_id,
                    tick,
                    existing_actions_hash,
                    actions_hash
                );
                net.rollback_flags.push(tick); // Prediction missed, actions in the buffer didn't match the actions in the message
                existing_frame.ser_actions = frame.ser_actions; // Overwrite the existing actions with the message actions
            }

            if existing_frame.state_hash != frame.state_hash {
                log::error!(
                    "Received conflicting state hash from peer {} for tick {}: {} vs {}",
                    peer.peer_id,
                    tick,
                    existing_frame.state_hash.unwrap_or(0),
                    frame.state_hash.unwrap_or(0)
                );

                net.rollback_flags.push(tick); // Prediction missed, state hash in the buffer didn't match the state hash in the message
                existing_frame.state_hash = frame.state_hash; // Overwrite the existing state hash with the message state hash
                physics_hash = match frame.state_hash {
                    Some(hash) => Some(hash),
                    None => existing_frame.state_hash,
                }
            }
        } else {
            log::trace!(
                "Received new frame from peer {} for tick {}: {}",
                peer.peer_id,
                tick,
                frame
            );
            peer.received_remote_ticks.insert(tick, frame);
        }

        // If we need to rollback as a result of the peer's message, deserialize and insert the actions into the combined world_buffer
        if net.rollback_flags.get(tick).is_some() {
            if let Some(de) = deserialized_actions {
                log::trace!(
                    "Inserting {} deserialized actions from peer {} into tick {}: {:?}",
                    de.len(),
                    peer.peer_id,
                    tick,
                    de
                );

                net.world_buffer.upsert_remote_actions(tick, de);
            };
        }

        if physics_hash.is_some() {
            // Record this peer as frame complete
            if !net.frame_complete_peers.contains_key(&tick) {
                net.frame_complete_peers.insert(tick, HashMap::default());
            }
            if let Some(completed_peers) = net.frame_complete_peers.get_mut(&tick) {
                completed_peers.insert(peer.peer_id, physics_hash.unwrap());

                if completed_peers.len() == num_peers {
                    // All peers have sent us a frame for this tick
                    // Confirm all physics hashes match
                    net.frame_complete_peers.swap_remove(&tick);
                    if tick > net.synchronized_tick {
                        let mut mismatches = Vec::new();
                        let map = net
                            .frame_complete_peers
                            .get(&tick)
                            .expect("Failed to get frame_complete_peers");

                        let hashes = map.values().cloned().collect::<Vec<_>>();
                        for (peer_id, hash) in map.iter() {
                            if hashes.iter().any(|h| h != hash) {
                                mismatches.push(*peer_id);
                            }
                        }

                        if !mismatches.is_empty() {
                            log::error!(
                                "State mismatch detected at tick {} between peers {:?}",
                                tick,
                                mismatches
                            ); // TODO kill the game because of fatal error
                        } else {
                            net.synchronized_tick = tick;
                            log::trace!("Synchronized tick updated to {}", tick);
                        }
                    }
                }
            }
        }
    }

    peer.latest_remote_tick_received =
        std::cmp::max(peer.latest_remote_tick_received, message.tick);
    peer.latest_local_tick_requested = std::cmp::max(
        peer.latest_local_tick_requested,
        message.requested_ticks.iter().max().cloned().unwrap_or(0),
    );
    peer.requested_local_ticks = message.requested_ticks;

    // Calculate the lag between the peer's latest action tick and the next action tick it expects
    peer.remote_lag =
        (peer.latest_remote_tick_received + 1) as i64 - peer.latest_local_tick_requested as i64;
}

/// Constructs a new action and then adds it to the local world buffer at the current tick
pub fn ingest_local_action(
    net: &mut GR3DNet,
    node: Gd<Node>,
    operation: Operation,
    data: Dictionary,
) {
    if let Some((cuid, handle)) = extract_ids(node.clone()) {
        let action = Action::new(cuid, handle, node, operation, data);
        net.world_buffer
            .upsert_local_actions(net.tick, vec![action]);
    }
}

fn extract_ids(node: Gd<Node>) -> Option<(GString, Option<(u32, u32)>)> {
    match node.get_class().to_string().as_str() {
        "RapierArea3D" => Some(get_ids(node.cast::<RapierArea3D>())),
        "RapierCollisionShape3D" => Some(get_ids(node.cast::<RapierCollisionShape3D>())),
        "RapierKinematicCharacter3D" => Some(get_ids(node.cast::<RapierKinematicCharacter3D>())),
        "RapierPIDCharacter3D" => Some(get_ids(node.cast::<RapierPIDCharacter3D>())),
        "RapierRigidBody3D" => Some(get_ids(node.cast::<RapierRigidBody3D>())),
        "RapierStaticBody3D" => Some(get_ids(node.cast::<RapierStaticBody3D>())),
        _ => {
            log::error!(
                "Node class not recognized: {}",
                node.get_class().to_string()
            );
            None
        }
    }
}

fn get_ids(node: Gd<impl IRapierObject>) -> (GString, Option<(u32, u32)>) {
    let cuid = node.bind().get_cuid();
    let handle = node.bind().get_handle_raw();
    (cuid, handle)
}
