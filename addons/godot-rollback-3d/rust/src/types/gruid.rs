use godot::builtin::Variant;
use godot::meta::ToGodot;
use serde::{Deserialize, Serialize};

use crate::utils::try_parse;
use crate::{types::*, Network};

// Note: to sort a HashMap by GRUID key, just use: `hashmap.sort_unstable_keys()`

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GRUID {
    pub peer_index: PeerIndex,
    pub generation: u32,
}

impl GRUID {
    pub fn new(peer_index: PeerIndex, generation: u32) -> Self {
        Self {
            peer_index,
            generation,
        }
    }

    pub fn try_from_string(gruid: &String) -> Option<Self> {
        let parts: Vec<&str> = gruid.split('-').collect();
        if parts.len() != 2 {
            log::error!("Invalid GRUID format: {}", gruid);
            return None;
        }
        let peer_index = try_parse::<PeerIndex>(parts[0])?;
        let generation = try_parse::<u32>(parts[1])?;
        Some(Self::new(peer_index, generation))
    }

    pub fn get_peer_id(&self, network: &Network) -> Option<PeerId> {
        match network.get_peer_id(self.peer_index) {
            Some(peer_id) => Some(peer_id),
            None => {
                log::error!(
                    "GRUID::get_peer_id: Peer index {} not found in network",
                    self.peer_index
                );
                None
            }
        }
    }

    pub fn to_variant(&self) -> Variant {
        self.to_string().to_variant()
    }
}

impl std::fmt::Display for GRUID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}-{}", self.peer_index, self.generation)
    }
}
