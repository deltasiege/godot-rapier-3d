use godot::prelude::*;
use rapier3d::parry::utils::hashmap::HashMap;

use crate::adapters::GR3DInputAdapter;
use crate::network::UpdateFrame;
use crate::types::*;
use crate::utils::get_hash;

#[derive(Debug, Clone)]
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

    /// Returns a string containing containing ticks missing between the starts and ends of all buffers.
    pub fn get_holes(&self) -> String {
        format!(
            "inputs: {:?}\n ser_inputs: {:?}\n input_hashes: {:?}\n world_hashes: {:?}",
            get_holes_in_buffer(self.inputs.clone()),
            get_holes_in_buffer(self.ser_inputs.clone()),
            get_holes_in_buffer(self.input_hashes.clone()),
            get_holes_in_buffer(self.world_hashes.clone())
        )
    }

    pub fn record_update_frame(
        &mut self,
        frame: &UpdateFrame,
        input_adapter: &mut Gd<GR3DInputAdapter>,
    ) {
        let input_hash = get_hash(&frame.ser_inputs);
        log::trace!(
            "Recording received remote input hash: {}, world hash: {}, for tick: {}",
            input_hash,
            frame.world_hash,
            frame.tick
        );

        let overriden_input_hash = self.input_hashes.insert(frame.tick, input_hash);
        let overriden_world_hash = self.world_hashes.insert(frame.tick, frame.world_hash);

        if overriden_input_hash.is_some() && overriden_input_hash != Some(input_hash) {
            log::warn!(
                "Conflicting remote input hashes received for tick {}: {} != {}",
                frame.tick,
                overriden_input_hash.unwrap(),
                input_hash
            );
        }

        if overriden_world_hash.is_some() && overriden_world_hash != Some(frame.world_hash) {
            log::warn!(
                "Conflicting remote world hashes received for tick {}: {} != {}",
                frame.tick,
                overriden_world_hash.unwrap(),
                frame.world_hash
            );
        }

        self.ser_inputs.insert(frame.tick, frame.ser_inputs.clone());

        let inputs: InputMap = input_adapter
            .bind_mut()
            .deserialize_inputs(&frame.ser_inputs);

        self.inputs.insert(frame.tick, inputs);
    }
}

/// Returns a vector of ticks that are missing in the buffer.
fn get_holes_in_buffer<T>(buffer: HashMap<Tick, T>) -> Vec<Tick> {
    let mut keys: Vec<Tick> = buffer.into_keys().collect();
    keys.sort_unstable();
    let mut holes = Vec::new();
    if keys.is_empty() {
        return holes;
    }
    let mut last_tick = keys[0];
    for &tick in &keys[1..] {
        if tick != last_tick + 1 {
            holes.push(tick);
        }
        last_tick = tick;
    }
    holes
}
