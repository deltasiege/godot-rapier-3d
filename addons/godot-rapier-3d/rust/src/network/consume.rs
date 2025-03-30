use godot::prelude::*;

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

    if message_length > MAX_MESSAGE_SIZE {
        log::warn!(
            "Received a message from peer {} that is too big ({} bytes): {:?}",
            peer.peer_id,
            message_length,
            message,
        );
    }

    /*
        Overwrite actions in the peer's buffer with actions in the message
        Raise rollback flags if changes are made to the peer's buffer

        Record the peer as having sent actions on the given tick
        and update the action complete tick if all peers have sent actions.
    */
    for (tick, actions) in message.actions {
        let actions_hash = get_hash(&actions);

        if peer.actions.contains_key(&tick) {
            let existing_hash = peer.action_hashes.get(&tick);
            if existing_hash == Some(&actions_hash) {
                log::trace!("Skipping actions ({}) received from peer {} for tick {} because matching actions were already received for that tick", actions_hash, peer.peer_id, tick);
                continue;
            } else {
                log::error!(
                    "Received conflicting actions from peer {} for tick {}: {} vs {}",
                    peer.peer_id,
                    tick,
                    existing_hash.unwrap_or(&0),
                    actions_hash
                );
            }
        };

        if actions.is_empty() {
            if peer.actions.contains_key(&tick) {
                net.rollback_flags.push(tick); // Prediction missed, actions were present in the buffer but not in the message
                log::trace!("Rollback flag raised for tick {} because actions were present in local buffer but not in message from peer {}", tick, peer.peer_id);
            }

            peer.remove_received_actions(tick);
        } else {
            let existing_actions = peer.actions.get(&tick).cloned().unwrap_or_default();
            if existing_actions != actions {
                net.rollback_flags.push(tick); // Prediction missed, actions in the buffer did not match the actions in the message
                log::trace!("Rollback flag raised for tick {} because actions in local buffer did not match message from peer {}", tick, peer.peer_id);
            }

            if let Some(de) = deserialize_actions(actions.clone(), &scene_root) {
                log::trace!(
                    "Inserting {} deserialized actions from peer {} into tick {}: {:?}",
                    de.len(),
                    peer.peer_id,
                    tick,
                    de
                );

                net.world_buffer.insert_actions(tick, de);
            }
            peer.record_received_actions(tick, actions);
        }

        if let Some(existing_peers) = net.action_complete_peers.get_mut(&tick) {
            let peer_is_absent = existing_peers.insert(peer.peer_id);

            if !peer_is_absent {
                log::warn!(
                    "Received duplicate actions from peer {} for tick {}",
                    peer.peer_id,
                    tick
                );
                // return; // TODO how to handle duplicate actions?
            }

            if existing_peers.len() == num_peers {
                net.action_complete_peers.swap_remove(&tick);
                if tick > net.action_complete_tick {
                    net.action_complete_tick = tick;
                    log::trace!("Action complete tick updated to {}", tick);
                }
            }
        } else {
            net.action_complete_peers
                .insert(tick, vec![peer.peer_id].into_iter().collect());

            if num_peers == 1 {
                if tick > net.action_complete_tick {
                    net.action_complete_tick = tick;
                    log::trace!("Action complete tick updated to {}", tick);
                }
            }
        }
    }

    /*
        Overwrite state hashes in the peer's buffer with hashes in the message

        Then, record the peer as having sent state hashes on the given ticks
        and update the physics_state_hash_complete tick if all peers have sent hashes for that tick.

        If the physics_state_hash_complete was updated, also check for synchronization and throw a
        critical error if there are state mismatches.
    */
    for (tick, hash) in message.state_hashes {
        if peer.state_hashes.contains_key(&tick) {
            let existing_hash = peer.state_hashes.get(&tick);
            if existing_hash == Some(&hash) {
                log::trace!("Skipping state hash ({}) received from peer {} for tick {} because a matching hash was already received for that tick", hash, peer.peer_id, tick);
                continue;
            } else {
                log::error!(
                    "Received conflicting state hashes from peer {} for tick {}: {} vs {}",
                    peer.peer_id,
                    tick,
                    existing_hash.unwrap(),
                    hash
                ); // TODO fatal error? or overwrite
            }
        };

        peer.record_received_state_hash(tick, hash);
        log::trace!(
            "Received state hash {} from peer {} for tick {}",
            hash,
            peer.peer_id,
            tick
        );

        let prev_hash_complete_tick = net.physics_hash_complete_tick.clone();

        if let Some(existing_map) = net.physics_hash_complete_peers.get_mut(&tick) {
            let old_hash = existing_map.insert(peer.peer_id, hash);

            if old_hash.is_some() && old_hash != Some(hash) {
                log::error!(
                    "Received conflicting state hashes from peer {} for tick {}: {} and {}",
                    peer.peer_id,
                    tick,
                    old_hash.unwrap(),
                    hash
                );
            }

            if existing_map.len() == num_peers {
                net.physics_hash_complete_peers.swap_remove(&tick);
                net.physics_hash_complete_tick =
                    std::cmp::max(net.physics_hash_complete_tick, tick);
            }
        } else {
            net.physics_hash_complete_peers
                .insert(tick, std::iter::once((peer.peer_id, hash)).collect());

            if num_peers == 1 {
                net.physics_hash_complete_tick =
                    std::cmp::max(net.physics_hash_complete_tick, tick);
            }
        }

        // If the physics state hash was updated, check for hash mismatches
        if prev_hash_complete_tick != net.physics_hash_complete_tick {
            let mut mismatches = Vec::new();
            let map = net
                .physics_hash_complete_peers
                .get(&net.physics_hash_complete_tick)
                .expect("Failed to get physics hash complete peers after update");

            let hashes = map.values().cloned().collect::<Vec<_>>();
            for (peer_id, hash) in map.iter() {
                if hashes.iter().any(|h| h != hash) {
                    mismatches.push(*peer_id);
                }
            }

            if !mismatches.is_empty() {
                log::error!(
                    "State mismatch detected at tick {} between peers {:?}",
                    net.physics_hash_complete_tick,
                    mismatches
                ); // TODO kill the game because of fatal error
            } else {
                if net.action_complete_tick >= net.physics_hash_complete_tick {
                    net.synchronized_tick = net.physics_hash_complete_tick;
                }
            }
        }
    }

    if peer.latest_remote_tick_received < message.tick {
        peer.latest_remote_tick_received = message.tick; // Record the latest tick received from the peer
        peer.latest_remote_action_tick_received = message.tick; // Record the tick the peer latest sent actions for
        peer.latest_remote_hash_tick_received = message.tick; // Record the tick the peer latest sent a state hash for

        // TODO need to check if the received arrays are contiguous - dont have missing slots

        peer.next_local_action_tick_requested = message.next_action_tick_requested; // Record the next action tick the peer is expecting as specified in message
        peer.next_local_hash_tick_requested = message.next_hash_tick_requested; // Record the next hash tick the peer is expecting as specified in message

        peer.remote_lag = (peer.latest_remote_action_tick_received + 1) as i64
            - peer.next_local_action_tick_requested as i64; // Calculate the lag between the peer's latest action tick and the next action tick it expects
        peer.latest_remote_message_size = message_length; // Record the size of the message received from the peer
    }
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
        net.world_buffer.insert_action(net.tick, action);
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
