use godot::prelude::*;
use rapier3d::parry::utils::hashmap::HashMap;

use crate::adapters::GR3DInputAdapter;
use crate::types::*;
use crate::utils::get_hash;

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

impl PeerBuffers {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.inputs.clear();
        self.ser_inputs.clear();
        self.input_hashes.clear();
        self.world_hashes.clear();
    }
}

/// Store current inputs in the input buffer.
pub fn capture_current_inputs(
    buffers: &mut PeerBuffers,
    current_tick: Tick,
    adapter: &mut Option<Gd<GR3DInputAdapter>>,
) {
    match adapter {
        Some(adapter) => {
            let input_map = adapter.bind_mut().get_inputs();
            let serialized_inputs = adapter.bind_mut().get_ser_inputs();
            let input_hash = get_hash(&serialized_inputs);

            buffers.inputs.insert(current_tick, input_map);
            buffers.ser_inputs.insert(current_tick, serialized_inputs);
            buffers.input_hashes.insert(current_tick, input_hash);
        }
        None => {
            log::error!("capture_current_inputs failed: input adapter is not set");
        }
    }
}
