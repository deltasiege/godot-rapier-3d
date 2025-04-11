use crate::types::*;

/// Data describing a peer in the network.
#[derive(Debug, Clone)]
pub struct PeerMetadata {
    pub id: PeerId,             // The unique identifier for this peer (peer_id in Godot)
    pub idx: Option<PeerIndex>, // The unique index of this peer in the host peer's list of peers
    pub is_spectator: bool,     // Whether this peer is a spectator
}

impl PeerMetadata {
    pub fn new(id: i64) -> Self {
        Self {
            id,
            idx: None,
            is_spectator: false,
        }
    }

    pub fn new_with_idx(id: PeerId, idx: PeerIndex) -> Self {
        Self {
            id,
            idx: Some(idx),
            is_spectator: false,
        }
    }
}
