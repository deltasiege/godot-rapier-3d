/// Data describing a peer in the network.
#[derive(Debug, Clone)]
pub struct PeerMetadata {
    pub id: i64,            // The unique identifier for this peer (peer_id in Godot)
    pub idx: Option<i64>,   // The unique index of this peer in the host peer's list of peers
    pub is_spectator: bool, // Whether this peer is a spectator
}

impl PeerMetadata {
    pub fn new(id: i64) -> Self {
        Self {
            id,
            idx: None,
            is_spectator: false,
        }
    }

    pub fn new_with_idx(id: i64, idx: i64) -> Self {
        Self {
            id,
            idx: Some(idx),
            is_spectator: false,
        }
    }
}
