use rapier3d::parry::utils::hashmap::HashMap;

use crate::types::*;

#[derive(Debug)]
pub struct PeerBuffers {
    pub inputs: HashMap<Tick, InputMap>,
    pub ser_inputs: HashMap<Tick, Vec<u8>>,
    pub input_hashes: HashMap<Tick, u64>,
    pub world_hashes: HashMap<Tick, u64>,
}

impl Default for PeerBuffers {
    fn default() -> Self {
        Self {
            inputs: HashMap::default(),
            ser_inputs: HashMap::default(),
            input_hashes: HashMap::default(),
            world_hashes: HashMap::default(),
        }
    }
}
