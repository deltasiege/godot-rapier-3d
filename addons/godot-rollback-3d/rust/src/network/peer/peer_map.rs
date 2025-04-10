use godot::prelude::*;

use crate::network::*;

/// Array of all peer IDs known to the host peer, used to uniquely define the peer_index of each peer
pub fn get_peer_map(network: &Network) -> Result<Array<i64>, ()> {
    match network.get_adapter() {
        Some(adapter) => {
            let mut peer_map = Array::new();

            let local_id = adapter.bind().get_unique_id();

            peer_map.push(local_id);

            for peer in network.remote_peers.iter() {
                peer_map.push(peer.metadata.id);
            }

            Ok(peer_map)
        }
        None => Err(()),
    }
}

/// Decode the peer map data and update the local and remote peers accordingly.
pub fn apply_peer_map_to_network(network: &mut Network, peer_map: Array<i64>) -> Result<(), ()> {
    let adapter = match network.get_adapter() {
        Some(adapter) => adapter,
        None => return Err(()),
    };

    let local_id = adapter.bind().get_unique_id();
    network.peer_map = Some(peer_map.clone());

    for (idx, peer_id) in peer_map.iter_shared().enumerate() {
        if peer_id == local_id {
            network.local_peer = Some(LocalPeer::new(PeerMetadata::new_with_idx(
                peer_id, idx as i64,
            )));
        } else if let Some(peer) = network.get_remote_peer_mut(peer_id) {
            peer.metadata.id = peer_id;
            peer.metadata.idx = Some(idx as i64);
        } else {
            log::error!("Local/remote peer with ID {} not found", peer_id);
        }
    }
    Ok(())
}
