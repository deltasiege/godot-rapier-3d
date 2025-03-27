use godot::prelude::*;
use rapier3d::parry::utils::hashmap::HashMap;

use crate::utils::extract_from_dict;

use super::{
    super::state::PhysicsState, actions::sort_actions, add_node_to_world, configure_node,
    modify_nodes::teleport_node, move_node, remove_node_from_world, Action, BufferStep, Operation,
};

pub struct WorldBuffer {
    pub buffer: HashMap<usize, BufferStep>, // Timestep id -> BufferStep
    pub max_len: usize,                     // Maximum number of steps to keep in the buffer
}

impl WorldBuffer {
    pub fn new(max_len: usize) -> Self {
        Self {
            buffer: HashMap::default(),
            max_len,
        }
    }

    pub fn default() -> Self {
        Self::new(20)
    }

    /// Returns the buffer step at the given timestep
    pub fn get_step(&self, timestep_id: usize) -> Option<&BufferStep> {
        self.buffer.get(&timestep_id)
    }

    /// Returns mutable ref to buffer step at the given timestep
    pub fn get_step_mut(&mut self, timestep_id: usize) -> Option<&mut BufferStep> {
        self.buffer.get_mut(&timestep_id)
    }

    /// Returns the physics state at the given timestep
    pub fn get_physics_state(&self, timestep_id: usize) -> Option<Vec<u8>> {
        self.buffer
            .get(&timestep_id)
            .and_then(|step| step.physics_state.clone())
    }

    /// Adds an action to the buffer at the given timestep
    /// Creates a new BufferStep if one does not exist
    pub fn insert_action(&mut self, action: Action, timestep_id: usize) {
        if let Some(step) = self.buffer.get_mut(&timestep_id) {
            let existing_actions = step
                .actions
                .entry(action.cuid.to_string())
                .or_insert(Vec::new());
            let already_has_op = existing_actions
                .iter()
                .any(|a| a.operation == action.operation);
            if !already_has_op {
                existing_actions.push(action);
            } else {
                // TODO merge certain operations like move character or apply forces to RB,
                // should be allowed to apply multiple of them per step/frame
                log::warn!(
                    "Not inserting action {:?} at timestep {} because it already exists",
                    action,
                    timestep_id
                );
            }
        } else {
            let step = BufferStep::new(timestep_id, None, vec![action]);
            self.buffer.insert(step.timestep_id, step);
        }
    }

    /// Executes all actions in the buffer at the given timestep
    pub fn execute_actions(&mut self, timestep_id: usize, physics: &mut PhysicsState) {
        if let Some(step) = self.buffer.get(&timestep_id) {
            let flattened = step.actions.values().flatten();
            let sorted_actions = sort_actions(flattened.collect());
            for action in sorted_actions.iter() {
                match action.node.clone().try_cast::<Node3D>() {
                    Ok(node) => {
                        self.execute_action(action, node, physics);
                    }
                    Err(e) => {
                        log::error!(
                            "Failed to cast node {:?} to Node3D. Skipping action {:?}. Error: {:?}",
                            action.node,
                            action,
                            e
                        );
                    }
                }
            }
        }
    }

    fn execute_action(&self, action: &Action, node: Gd<Node3D>, physics: &mut PhysicsState) {
        match action.operation {
            Operation::AddNode => {
                add_node_to_world(node, physics);
            }
            Operation::RemoveNode => {
                remove_node_from_world(node, physics);
            }
            Operation::ConfigureNode => {
                configure_node(node);
            }
            Operation::MoveNode => {
                if let Some(movement) = extract_from_dict(&action.data, "movement", true) {
                    move_node(node, movement, physics);
                }
            }
            Operation::TeleportNode => {
                if let Some(position) = extract_from_dict(&action.data, "position", true) {
                    teleport_node(node, position, physics);
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
                let step = BufferStep::new(next_timestep_id, Some(phx_state), Vec::new());
                self.buffer.insert(step.timestep_id, step);
            }
        }

        // Remove old snapshots if buffer is full
        while self.buffer.len() > self.max_len {
            self.remove_oldest();
        }
    }

    /// Removes all physics states from BufferSteps after the given timestep
    pub fn mark_physics_stale_after(&mut self, timestep_id: usize) {
        let keys: Vec<usize> = self.buffer.keys().cloned().collect();
        for key in keys {
            if key > timestep_id {
                if let Some(step) = self.buffer.get_mut(&key) {
                    step.physics_state = None;
                }
            }
        }
    }

    /// Removes all actions from BufferSteps after the given timestep
    /// Unused
    pub fn mark_actions_stale_after(&mut self, timestep_id: usize) {
        let keys: Vec<usize> = self.buffer.keys().cloned().collect();
        for key in keys {
            if key > timestep_id {
                if let Some(step) = self.buffer.get_mut(&key) {
                    step.actions.clear();
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
        self.buffer.swap_remove(&oldest);
    }

    /// Returns the actions in the buffer at the given timestep
    pub fn get_actions(&self, timestep_id: usize) -> Option<Vec<Action>> {
        let result: Option<Vec<Action>> = self
            .buffer
            .get(&timestep_id)
            .map(|step| step.actions.values().flatten().cloned().collect());

        match result {
            Some(actions) => {
                if actions.is_empty() {
                    return None;
                }
                Some(actions)
            }
            None => None,
        }
    }

    /// Returns the hash of the physics state in the buffer at the given timestep
    pub fn get_state_hash(&self, timestep_id: usize) -> Option<u64> {
        self.buffer
            .get(&timestep_id)
            .and_then(|step| step.physics_hash)
    }
}
