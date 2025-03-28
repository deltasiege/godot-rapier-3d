use bincode::{config::standard, serde::decode_from_slice};
use godot::prelude::*;
use rapier3d::parry::utils::hashmap::HashMap;

use super::buffer_frame::BufferFrame;
use crate::{
    actions::{action::insert_action_if_allowed, serde::deserialize_actions},
    world::*,
    Action,
};

pub struct WorldBuffer {
    pub buffer: HashMap<usize, BufferFrame>, // tick -> BufferFrame. Contains all combined actions (local + all peers)
    pub local_buffer: HashMap<usize, BufferFrame>, // tick -> BufferFrame. Contains ONLY locally created actions
    pub max_len: usize,                            // Maximum number of frames to keep in the buffer
}

impl WorldBuffer {
    pub fn new(max_len: usize) -> Self {
        Self {
            buffer: HashMap::default(),
            local_buffer: HashMap::default(),
            max_len,
        }
    }

    /// Adds an action to the buffer at the given tick
    /// Creates a new BufferFrame if one does not exist
    pub fn insert_action(&mut self, tick: usize, action: Action) {
        let combined_buff = &mut self.buffer;
        _insert_action_into_buffer(tick, action.clone(), combined_buff);
        _reserialize_actions_in_buffer(combined_buff, tick);

        let local_buff = &mut self.local_buffer;
        _insert_action_into_buffer(tick, action, local_buff);
        _reserialize_actions_in_buffer(local_buff, tick);

        self.prune_buffers();
    }

    /// Adds given serialized actions to the buffer at the given tick
    /// Creates a new BufferFrame if one does not exist
    pub fn insert_serialized_actions(
        &mut self,
        tick: usize,
        actions: &Vec<u8>,
        scene_root: &Gd<Node>,
    ) {
        let combined_buff = &mut self.buffer;
        _insert_serialized_actions_into_buffer(tick, actions, scene_root, combined_buff);
        self.prune_buffers();
    }

    fn insert_serialized_action(&mut self, tick: usize, actions: &Vec<u8>, scene_root: &Gd<Node>) {
        let combined_buff = &mut self.buffer;
        _insert_serialized_actions_into_buffer(tick, actions, scene_root, combined_buff);

        let local_buff = &mut self.local_buffer;
        _insert_serialized_actions_into_buffer(tick, actions, scene_root, local_buff);
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

        while self.local_buffer.len() > self.max_len {
            remove_oldest(&mut self.local_buffer);
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

    /// Returns the actions in the combined buffer at the given tick
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

    /// Returns the physics state in the combined buffer at the given tick
    pub fn get_physics_state(&self, tick: usize) -> Option<Vec<u8>> {
        self.buffer
            .get(&tick)
            .and_then(|frame| frame.physics_state.clone())
    }

    /// Returns the hash of the physics state in the combined buffer at the given tick
    pub fn get_physics_state_hash(&self, tick: usize) -> Option<u64> {
        self.buffer.get(&tick).and_then(|frame| frame.physics_hash)
    }
}

/// Remove the smallest key from the given hashmap
pub fn remove_oldest<T>(map: &mut HashMap<usize, T>) {
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

fn _insert_action_into_buffer(
    tick: usize,
    action: Action,
    buffer: &mut HashMap<usize, BufferFrame>,
) {
    if let Some(frame) = buffer.get_mut(&tick) {
        let cuid = action.cuid.to_string();
        insert_action_if_allowed(action, frame.actions.entry(cuid));
    } else {
        let frame = BufferFrame::new(tick, None, vec![action]);
        buffer.insert(tick, frame);
    }
}

fn _reserialize_actions_in_buffer(buffer: &mut HashMap<usize, BufferFrame>, tick: usize) {
    if let Some(frame) = buffer.get_mut(&tick) {
        frame.reserialize_actions();
    } else {
        log::error!(
            "Failed to reserialize actions in buffer. Tick {} not found",
            tick
        );
    }
}

fn _insert_serialized_actions_into_buffer(
    tick: usize,
    actions: &Vec<u8>,
    scene_root: &Gd<Node>,
    buffer: &mut HashMap<usize, BufferFrame>,
) {
    if let Some(deserialized) = deserialize_actions(actions.clone(), &scene_root) {
        for action in deserialized {
            _insert_action_into_buffer(tick, action, buffer);
        }

        _reserialize_actions_in_buffer(buffer, tick);
    };
}
