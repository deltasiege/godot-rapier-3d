use godot::prelude::*;
use rapier3d::parry::utils::hashmap::HashMap;

use super::buffer_frame::BufferFrame;
use crate::{actions::Operation, interface::GR3DNet, nodes::*, world::*, Action};

pub struct WorldBuffer {
    pub buffer: HashMap<usize, BufferFrame>, // tick -> BufferFrame
    pub local_actions: HashMap<usize, Vec<Action>>, // tick -> Vec<Action>. Locally created actions
    pub max_len: usize,                      // Maximum number of frames to keep in the buffer
}

impl WorldBuffer {
    pub fn new(max_len: usize) -> Self {
        Self {
            buffer: HashMap::default(),
            local_actions: HashMap::default(),
            max_len,
        }
    }

    /// Returns the buffer frame at the given tick
    pub fn get_frame(&self, tick: usize) -> Option<&BufferFrame> {
        self.buffer.get(&tick)
    }

    /// Returns mutable ref to buffer frame at the given tick
    pub fn get_frame_mut(&mut self, tick: usize) -> Option<&mut BufferFrame> {
        self.buffer.get_mut(&tick)
    }

    /// Adds an action to the buffer at the given tick
    /// Creates a new BufferFrame if one does not exist
    pub fn insert_action(&mut self, tick: usize, action: Action) {
        if let Some(frame) = self.buffer.get_mut(&tick) {
            let existing_actions = frame
                .actions
                .entry(action.cuid.to_string())
                .or_insert(Vec::new());
            let already_has_op = existing_actions
                .iter()
                .any(|a| a.operation == action.operation);
            if !already_has_op {
                existing_actions.push(action);
                self.local_actions.insert(tick, existing_actions.clone());
            } else {
                // TODO merge certain operations like move character or apply forces to RB,
                // should be allowed to apply multiple of them per tick
                log::warn!(
                    "Not inserting action {:?} at tick {} because a matching action already exists there",
                    action,
                    tick
                );
            }
        } else {
            self.local_actions.insert(tick, vec![action.clone()]);
            let frame = BufferFrame::new(tick, None, vec![action]);
            self.buffer.insert(tick, frame);
        }

        self.prune_buffers();
    }

    /// Executes all actions in the buffer at the given tick
    pub fn apply_actions_to_world(&mut self, tick: usize, physics: &mut PhysicsState) {
        if let Some(frame) = self.buffer.get_mut(&tick) {
            let mut flattened: Vec<Action> = frame.actions.values().flatten().cloned().collect();
            apply_actions_to_world(&mut flattened, physics);
        }
    }

    /// Called whenever the world is stepped.
    /// Adds the next tick's BufferFrame with empty actions list.
    pub fn on_world_stepped(&mut self, next_tick: usize, resulting_state: Option<Vec<u8>>) {
        if let Some(phx_state) = resulting_state {
            if let Some(existing) = self.buffer.get_mut(&next_tick) {
                existing.physics_state = Some(phx_state);
            } else {
                let frame = BufferFrame::new(next_tick, Some(phx_state), Vec::new());
                self.buffer.insert(frame.tick, frame);
            }
        }

        self.prune_buffers();
    }

    /// Remove oldest entries from buffers if they exceed the max length
    pub fn prune_buffers(&mut self) {
        while self.buffer.len() > self.max_len {
            remove_oldest(&mut self.buffer);
        }

        while self.local_actions.len() > self.max_len {
            remove_oldest(&mut self.local_actions);
        }
    }

    /// Removes all physics states from BufferFrames after the given tick
    pub fn mark_physics_stale_after(&mut self, tick: usize) {
        let keys: Vec<usize> = self.buffer.keys().cloned().collect();
        for key in keys {
            if key > tick {
                if let Some(frame) = self.buffer.get_mut(&key) {
                    frame.physics_state = None;
                }
            }
        }
    }

    /// Removes all actions from BufferFrames after the given tick
    /// Unused
    pub fn mark_actions_stale_after(&mut self, tick: usize) {
        let keys: Vec<usize> = self.buffer.keys().cloned().collect();
        for key in keys {
            if key > tick {
                if let Some(frame) = self.buffer.get_mut(&key) {
                    frame.actions.clear();
                }
            }
        }
    }

    /// Returns the actions in the buffer at the given tick
    pub fn get_actions(&self, tick: usize) -> Option<Vec<Action>> {
        let result: Option<Vec<Action>> = self
            .buffer
            .get(&tick)
            .map(|frame| frame.actions.values().flatten().cloned().collect());

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

    /// Returns the physics state at the given tick
    pub fn get_physics_state(&self, tick: usize) -> Option<Vec<u8>> {
        self.buffer
            .get(&tick)
            .and_then(|frame| frame.physics_state.clone())
    }

    /// Returns the hash of the physics state in the buffer at the given tick
    pub fn get_physics_state_hash(&self, tick: usize) -> Option<u64> {
        self.buffer.get(&tick).and_then(|frame| frame.physics_hash)
    }
}

/// Constructs a new action and then adds it to the local buffer at the current tick
pub fn ingest_local_action(
    net: &mut GR3DNet,
    node: Gd<Node>,
    operation: Operation,
    data: Dictionary,
) {
    if let Some((cuid, handle)) = extract_ids(node.clone()) {
        let action = Action::new(cuid, handle, node, operation, data);
        net.world_buffer.insert_action(net.tick, action);
    }
}

fn extract_ids(node: Gd<Node>) -> Option<(GString, Option<(u32, u32)>)> {
    match node.get_class().to_string().as_str() {
        "RapierArea3D" => Some(get_ids(node.cast::<RapierArea3D>())),
        "RapierCollisionShape3D" => Some(get_ids(node.cast::<RapierCollisionShape3D>())),
        "RapierKinematicCharacter3D" => Some(get_ids(node.cast::<RapierKinematicCharacter3D>())),
        "RapierPIDCharacter3D" => Some(get_ids(node.cast::<RapierPIDCharacter3D>())),
        "RapierRigidBody3D" => Some(get_ids(node.cast::<RapierRigidBody3D>())),
        "RapierStaticBody3D" => Some(get_ids(node.cast::<RapierStaticBody3D>())),
        _ => {
            log::error!(
                "Node class not recognized: {}",
                node.get_class().to_string()
            );
            None
        }
    }
}

fn get_ids(node: Gd<impl IRapierObject>) -> (GString, Option<(u32, u32)>) {
    let cuid = node.bind().get_cuid();
    let handle = node.bind().get_handle_raw();
    (cuid, handle)
}

/// Remove the smallest key from the given hashmap
fn remove_oldest<T>(map: &mut HashMap<usize, T>) {
    let earliest_key = match map.keys().min() {
        Some(oldest) => *oldest,
        None => {
            log::error!("Failed to determine earliest_key. Clearing entire HashMap");
            map.clear();
            return;
        }
    };

    map.swap_remove(&earliest_key);
}
