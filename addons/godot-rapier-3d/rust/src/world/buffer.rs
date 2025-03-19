use crate::{
    interface::{Action, Operation},
    World,
};
use std::collections::HashMap;

use super::state::restore_snapshot;

pub struct WorldBuffer {
    pub buffer: HashMap<usize, BufferStep>,
    pub max_len: usize,
}

/// Represents a single timestep in the buffer
pub struct BufferStep {
    timestep_id: usize,             // The timestep id of this step
    physics_state: Option<Vec<u8>>, // The state of the physics world at the beginning of this timestep
    actions: Vec<Action>,           // List of actions to apply during this timestep
}

impl WorldBuffer {
    pub fn new(max_len: usize) -> Self {
        Self {
            buffer: HashMap::new(),
            max_len,
        }
    }

    pub fn default() -> Self {
        Self::new(1000)
    }

    /// Returns the buffer step at the given timestep.
    pub fn get_step(&self, timestep_id: usize) -> Option<&BufferStep> {
        self.buffer.get(&timestep_id)
    }

    /// Returns the physics state at the given timestep.
    pub fn get_physics_state(&self, timestep_id: usize) -> Option<Vec<u8>> {
        self.buffer
            .get(&timestep_id)
            .and_then(|step| step.physics_state.clone())
    }

    /// Rolls the given world back to the given timestep and
    /// rewrites the physics state history from that point forward
    /// back to the current timestep
    pub fn corrective_rollback(&mut self, world: &mut World, target: BufferStep) {
        let current_timestep = world.state.timestep_id.clone();
        let steps_to_resim = current_timestep - target.timestep_id;

        match true {
            _ if target.timestep_id >= current_timestep => {
                log::error!(
                    "Cannot rollback to a future timestep: {}",
                    target.timestep_id
                );
            }
            _ if steps_to_resim == 0 => {
                log::warn!("Corrective rollback to same timestep. No action taken.");
            }
            _ if steps_to_resim > self.max_len => {
                log::error!(
                    "Cannot rollback more than the buffer length: {}",
                    self.max_len
                );
            }
            _ => match target.physics_state {
                Some(physics_state) => {
                    world.state.timestep_id = target.timestep_id;
                    self.mark_stale_after(target.timestep_id);
                    restore_snapshot(world, physics_state);

                    for _ in 0..steps_to_resim {
                        world.step();
                    }
                }
                None => {
                    log::error!("Provided BufferStep did not have attached physics state");
                }
            },
        }
    }

    /// Adds an action to the buffer at the given timestep
    /// Creates a new BufferStep if one does not exist
    pub fn insert_action(&mut self, action: Action, timestep_id: usize) {
        if let Some(step) = self.buffer.get_mut(&timestep_id) {
            step.actions.push(action);
        } else {
            let step = BufferStep {
                timestep_id,
                physics_state: None,
                actions: vec![action],
            };
            self.buffer.insert(step.timestep_id, step);
        }
    }

    /// Executes all actions in the buffer at the given timestep
    pub fn execute_actions(&mut self, timestep_id: usize) {
        if let Some(step) = self.buffer.get_mut(&timestep_id) {
            for action in step.actions.drain(..) {
                match action.operation {
                    Operation::ADD_NODE => {
                        log::info!("Adding node: {:?} {:?}", action.cuid, action.data);
                    }
                    Operation::REMOVE_NODE => {
                        log::info!("Removing node: {:?} {:?}", action.cuid, action.data);
                    }
                    Operation::CONFIGURE_NODE => {
                        log::info!("Configuring node: {:?} {:?}", action.cuid, action.data);
                    }
                    Operation::MOVE_NODE => {
                        log::info!("Modifying node: {:?} {:?}", action.cuid, action.data);
                    }
                }
            }
        }
    }

    /// Called whenever the world is stepped.
    /// Adds the next timestep's BufferStep with empty actions list.
    pub fn on_world_stepped(&mut self, next_timestep_id: usize, resulting_state: Option<Vec<u8>>) {
        if let Some(phx_state) = resulting_state {
            if let Some(existing) = self.buffer.get_mut(&next_timestep_id) {
                existing.physics_state = Some(phx_state);
            } else {
                let step = BufferStep {
                    timestep_id: next_timestep_id,
                    physics_state: Some(phx_state),
                    actions: Vec::new(),
                };
                self.buffer.insert(step.timestep_id, step);
            }
        }

        // Remove old snapshots if buffer is full
        while self.buffer.len() > self.max_len {
            self.remove_oldest();
        }
    }

    /// Removes all inner physics states from BufferSteps after the given timestep
    pub fn mark_stale_after(&mut self, timestep_id: usize) {
        let keys: Vec<usize> = self.buffer.keys().cloned().collect();
        for key in keys {
            if key > timestep_id {
                if let Some(step) = self.buffer.get_mut(&key) {
                    step.physics_state = None;
                }
            }
        }
    }

    /// Removes the oldest BufferStep from the buffer.
    fn remove_oldest(&mut self) {
        let oldest = match self.buffer.keys().min() {
            Some(oldest) => *oldest,
            None => {
                log::error!("Failed removing oldest BufferStep. Clearing entire WorldBuffer.");
                self.buffer.clear();
                return;
            }
        };
        self.buffer.remove(&oldest);
    }
}
