use std::hash::{DefaultHasher, Hash, Hasher};

use godot::prelude::*;
use rapier3d::parry::utils::hashmap::HashMap;

use crate::actions::{serialize_actions, Action};

/// Represents a single tick in the buffer
pub struct BufferFrame {
    pub tick: usize,                           // The tick that this frame affects
    pub physics_state: Option<Vec<u8>>, // The state of the physics world at the beginning of this tick
    pub physics_hash: Option<u64>, // Hash of the physics state at the beginning of this tick, should always exist if physics_state exists
    pub actions: HashMap<String, Vec<Action>>, // Map of node CUIDS against their list of actions to apply during this tick // TODO maybe change the key to node path?
    pub ser_actions: Option<Vec<u8>>, // Serialized actions for this tick. May not exist if serialization fails
    pub actions_hash: Option<u64>, // Hash of the serialized actions for this tick. May not exist if serialization fails
    pub nodes: HashMap<String, Gd<Node>>, // Map of node paths against their Godot pointers for this timestemp
}

impl BufferFrame {
    pub fn new(tick: usize, physics_state: Option<Vec<u8>>, actions: Vec<Action>) -> Self {
        let nodes = extract_node_entries_from_actions(&actions);
        let ser_actions = serialize_actions(&actions);
        let actions_hash = ser_actions.as_ref().map(|actions| get_hash(actions));
        let physics_hash = physics_state.as_ref().map(|state| get_hash(state));
        let actions_map = extract_action_entries_from_actions(actions);

        Self {
            tick,
            physics_state,
            physics_hash,
            actions: actions_map,
            ser_actions,
            actions_hash,
            nodes,
        }
    }

    pub fn reserialize_actions(&mut self) {
        let actions: Vec<Action> = self.actions.values().flatten().cloned().collect();
        self.ser_actions = serialize_actions(&actions);
    }
}

/// Returns the hash of the given byte vector
fn get_hash(physics_state: &Vec<u8>) -> u64 {
    let mut hasher = DefaultHasher::new();
    physics_state.hash(&mut hasher);
    hasher.finish()
}

/// Maps Action Node Path -> Node
fn extract_node_entries_from_actions(actions: &Vec<Action>) -> HashMap<String, Gd<Node>> {
    let mut map = HashMap::default();
    for action in actions {
        map.entry(action.node.get_path().to_string())
            .or_insert(action.node.clone());
    }
    map
}

/// Maps Action Node CUID -> Vec<Action>
fn extract_action_entries_from_actions(actions: Vec<Action>) -> HashMap<String, Vec<Action>> {
    let mut map = HashMap::default();
    for action in actions {
        let existing = map.entry(action.cuid.to_string()).or_insert(Vec::new());
        existing.push(action);
    }
    map
}
