use godot::prelude::*;

use super::deserialize_message_from_peer;
use crate::nodes::*;
use crate::{actions::Operation, interface::GR3DNet, Action};

/// Deserializes a remote peer update message and records relevant data against the peer
pub fn ingest_peer_message(net: &mut GR3DNet, sender_peer_id: i64, ser_message: PackedByteArray) {
    let message = match deserialize_message_from_peer(ser_message) {
        Some(message) => message,
        None => return,
    };

    let peer = match net.peers.iter_mut().find(|p| p.peer_id == sender_peer_id) {
        Some(peer) => peer,
        None => {
            log::error!("Received tick data from unknown peer {}", sender_peer_id);
            return;
        }
    };

    // Overwrite actions in the peer's buffer with actions in the message
    for (tick, actions) in message.actions {
        if actions.is_empty() {
            peer.actions.swap_remove(&tick);
        } else {
            peer.actions.insert(tick, actions);
        }
    }

    // Overwrite state hashes for all given ticks
    for hash in message.state_hashes {
        peer.state_hashes.insert(hash.0, hash.1);
    }

    peer.next_local_action_tick_requested = message.next_action_tick_requested;
    peer.next_local_hash_tick_requested = message.next_hash_tick_requested;
    peer.last_remote_action_tick_received = message.tick;
    peer.last_remote_hash_tick_received = message.tick;
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
