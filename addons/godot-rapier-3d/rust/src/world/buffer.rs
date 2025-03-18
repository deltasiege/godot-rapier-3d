use godot::prelude::*;
use std::collections::HashMap;

pub struct WorldBuffer {
    /// timestep_id = physics_state
    /// List of physics states for each timestep
    pub physics_states: HashMap<usize, Vec<u8>>,
    /// List of inputs to apply to controlled objects on each timestep
    /// timestep_id = [{ node, desired_movement }, ...]
    pub inputs: HashMap<usize, HashMap<Gd<Node3D>, Vector3>>,
    /// Maximum buffer size
    pub max_len: usize,
}

// UP TO

// fn get_buffered_snapshot(&self, timestep_id: i64) -> Option<Vec<u8>> {
//     self.snapshot_buffer.get(&(timestep_id as usize)).cloned()
// }

// fn save_current_snapshot_to_buffer(&mut self) {
//     if let Some(snapshot) = self.get_current_snapshot() {
//         self.snapshot_buffer
//             .insert(self.state.timestep_id, snapshot);
//     }
// }

// fn remove_oldest_snapshot_from_buffer(&mut self) {
//     let oldest = match self.snapshot_buffer.keys().min() {
//         Some(oldest) => *oldest,
//         None => {
//             log::error!("Failed removing oldest snapshot. Clearing entire snapshot buffer.");
//             self.snapshot_buffer.clear();
//             return;
//         }
//     };
//     self.snapshot_buffer.remove(&oldest);
// }

// pub fn remove_snapshots_after(&mut self, timestep_id: i64) {
//     let keys: Vec<usize> = self.snapshot_buffer.keys().cloned().collect();
//     for key in keys {
//         if key > timestep_id as usize {
//             self.snapshot_buffer.remove(&key);
//         }
//     }
// }
