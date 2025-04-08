/// Data describing a peer in the network.
pub struct PeerMetadata {
    pub id: i64,            // The unique identifier for this peer (peer_id in Godot)
    pub idx: i64,           // The index of this peer in the list of peers
    pub is_spectator: bool, // Whether this peer is a spectator
}

impl PeerMetadata {
    pub fn new(id: i64, idx: i64) -> Self {
        Self {
            id,
            idx,
            is_spectator: false,
        }
    }
}
