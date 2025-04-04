use godot::prelude::*;
use rapier3d::parry::utils::hashmap::HashMap;

use crate::actions::{serialize_actions, Action};
use crate::utils::get_hash;

/// Represents a single tick in the buffer
#[derive(Clone)]
pub struct BufferFrame {
    pub tick: usize,                           // The tick that this frame affects
    pub physics_state: Option<Vec<u8>>, // The state of the physics world at the beginning of this tick
    pub physics_hash: Option<u64>, // Hash of the physics state at the beginning of this tick, should always exist if physics_state exists
    pub actions: HashMap<String, Vec<Action>>, // Map of node CUIDS against their list of actions to apply during this tick // TODO maybe change the key to node path?
    pub nodes: HashMap<String, Gd<Node>>, // Map of node paths against their Godot pointers for this frame
}

impl BufferFrame {
    pub fn new(tick: usize, physics_state: Option<Vec<u8>>, actions: Vec<Action>) -> Self {
        let nodes = extract_node_entries_from_actions(&actions);
        let physics_hash = physics_state.as_ref().map(|state| get_hash(state));
        let actions_map = extract_action_entries_from_actions(actions);

        Self {
            tick,
            physics_state,
            physics_hash,
            actions: actions_map,
            nodes,
        }
    }

    pub fn new_from_physics_state(tick: usize, physics_state: Option<Vec<u8>>) -> Self {
        let physics_hash = physics_state.as_ref().map(|state| get_hash(state));
        Self {
            tick,
            physics_state,
            physics_hash,
            actions: HashMap::default(),
            nodes: HashMap::default(),
        }
    }

    pub fn set_physics_state(&mut self, physics_state: Vec<u8>) {
        self.physics_hash = Some(get_hash(&physics_state));
        self.physics_state = Some(physics_state);
    }

    pub fn get_serialized_actions(&self) -> Option<Vec<u8>> {
        let flat: Vec<Action> = self.actions.values().flatten().cloned().collect();
        serialize_actions(&flat)
    }
}

/// Converts a flat list of actions to a map of node path -> node pointer
fn extract_node_entries_from_actions(actions: &Vec<Action>) -> HashMap<String, Gd<Node>> {
    let mut map = HashMap::default();
    for action in actions {
        map.entry(action.node.get_path().to_string())
            .or_insert(action.node.clone());
    }
    map
}

/// Converts a flat list of actions to a map of node CUID -> Vec<Action> that apply to the node
fn extract_action_entries_from_actions(actions: Vec<Action>) -> HashMap<String, Vec<Action>> {
    let mut map = HashMap::default();
    for action in actions {
        let existing = map.entry(action.cuid.to_string()).or_insert(Vec::new());
        existing.push(action);
    }
    map
}
