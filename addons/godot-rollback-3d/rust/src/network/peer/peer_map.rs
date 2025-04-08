use godot::prelude::*;

use crate::network::*;
use crate::types::*;
use crate::utils::{decode_or_none, encode_or_none};

/// Array of all peer IDs known to the host peer, used to uniquely define the peer_index of each peer
pub fn get_peer_map(network: &Network) -> PackedByteArray {
    match get_peer_map_bytes(network) {
        Ok(peer_map) => PackedByteArray::from(peer_map),
        Err(e) => {
            log::error!("Failed to get peer map: {}", e);
            PackedByteArray::new()
        }
    }
}

/// Decode the peer map data and update the local and remote peers accordingly.
pub fn apply_peer_map_to_network(
    network: &mut Network,
    peer_map: PackedByteArray,
) -> Result<(), ()> {
    let adapter = match network.get_adapter() {
        Some(adapter) => adapter,
        None => return Err(()),
    };

    let local_id = adapter.bind().get_unique_id();
    match decode_peer_map(peer_map) {
        Some(peer_map) => {
            for (idx, peer_id) in peer_map.iter().enumerate() {
                if peer_id == &local_id {
                    network.local_peer = Some(LocalPeer::new(PeerMetadata::new_with_idx(
                        *peer_id, idx as i64,
                    )));
                } else if let Some(peer) = network.get_remote_peer_mut(*peer_id) {
                    peer.metadata.id = *peer_id;
                    peer.metadata.idx = Some(idx as i64);
                } else {
                    log::error!("Local/remote peer with ID {} not found", peer_id);
                }
            }
        }
        None => {
            log::error!("Failed to decode peer map data");
        }
    }
    Ok(())
}

/// Decodes peer map into Godot array of ints.
pub fn peer_map_to_godot(peer_map: PackedByteArray) -> Variant {
    decode_peer_map(peer_map).unwrap_or_default().to_variant()
}

fn decode_peer_map(peer_map: PackedByteArray) -> Option<Vec<i64>> {
    decode_or_none::<PeerMap>(peer_map.as_slice())
}

fn get_peer_map_bytes(network: &Network) -> Result<Vec<u8>, String> {
    let mut peer_map = Vec::new();
    let local_id = match network.get_adapter() {
        Some(adapter) => adapter.bind().get_unique_id(),
        None => return Err("No network adapter found".to_string()),
    };

    peer_map.push(local_id);

    for peer in network.remote_peers.iter() {
        peer_map.push(peer.metadata.id);
    }

    match encode_or_none(&peer_map) {
        Some(bytes) => Ok(bytes),
        None => Err("Failed to encode peer map".to_string()),
    }
}
