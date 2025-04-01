use rapier3d::parry::utils::hashmap::HashMap;

use super::buffer_frame::BufferFrame;
use crate::{actions::action::insert_action_if_allowed, world::*, Action};

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

    /// Inserts or updates a BufferFrame into both buffers at the tick specified in the frame
    pub fn upsert_local_frame(&mut self, frame: BufferFrame) {
        _upsert_frame_into_buffer(frame.clone(), &mut self.local_buffer);
        _upsert_frame_into_buffer(frame, &mut self.buffer);
    }

    /// Inserts or updates given actions in both buffers at the given tick
    /// Creates a new BufferFrame if one does not exist
    pub fn upsert_local_actions(&mut self, tick: usize, actions: Vec<Action>) {
        let combined_buff = &mut self.buffer;
        for action in actions.clone() {
            _upsert_action_into_buffer(tick, action.clone(), combined_buff);
        }

        let local_buff = &mut self.local_buffer;
        for action in actions {
            _upsert_action_into_buffer(tick, action, local_buff);
        }

        self.prune_buffers();
    }

    /// Inserts or updates given actions in the combined buffer at the given tick
    pub fn upsert_remote_actions(&mut self, tick: usize, actions: Vec<Action>) {
        let combined_buff = &mut self.buffer;
        for action in actions.clone() {
            _upsert_action_into_buffer(tick, action.clone(), combined_buff);
        }

        self.prune_buffers();
    }

    /// Executes all actions in the buffer at the given tick
    pub fn apply_actions_to_world(
        &mut self,
        tick: usize,
        physics: &mut PhysicsState,
    ) -> Vec<Action> {
        let mut flattened: Vec<Action> = Vec::new();
        if let Some(frame) = self.buffer.get_mut(&tick) {
            flattened = frame.actions.values().flatten().cloned().collect();
            apply_actions_to_world(&mut flattened, physics);
        }
        flattened
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

fn _upsert_action_into_buffer(
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

fn _upsert_frame_into_buffer(frame: BufferFrame, buffer: &mut HashMap<usize, BufferFrame>) {
    if let Some(existing) = buffer.get_mut(&frame.tick) {
        // Overwrite physics state only if existing is empty
        if frame.physics_state.is_some() {
            if existing.physics_state.is_none() {
                existing.set_physics_state(frame.physics_state.clone().unwrap());
            } else {
                log::warn!(
                    "Physics state already exists for tick {}. Not overwriting",
                    frame.tick
                );
            }
        }

        // Insert actions into existing frame
        let flattened_actions: Vec<Action> = frame.actions.values().flatten().cloned().collect();
        for action in flattened_actions {
            let cuid = action.cuid.to_string();
            insert_action_if_allowed(action, existing.actions.entry(cuid));
        }
    } else {
        buffer.insert(frame.tick, frame);
    };
}
