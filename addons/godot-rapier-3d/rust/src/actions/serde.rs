use bincode::{
    config::standard,
    serde::{decode_from_slice, encode_to_vec},
};
use godot::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    actions::{Action, Operation},
    network::NodeCache,
    utils::extract_from_dict,
};

// Struct sent over the network
#[derive(Serialize, Deserialize, Debug)]
pub struct LeanAction {
    pub node_index: u32, // The cached node path index of the node that this action refers to
    pub operation: Operation,
    pub handle: Option<(u32, u32)>,
    pub vector3s: Vec<Vector3>,
}

/// Serialize the given actions vector into byte vector
pub fn serialize_actions(actions: &Vec<Action>, node_cache: &mut NodeCache) -> Option<Vec<u8>> {
    if actions.is_empty() {
        return None;
    }

    let actions: Vec<LeanAction> = actions
        .iter()
        .map(|action| action_to_lean_action(action, node_cache))
        .collect();
    match encode_to_vec(actions, standard()) {
        Ok(serialized) => Some(serialized),
        Err(e) => {
            log::error!("Failed to serialize actions: {:?}", e);
            None
        }
    }
}

pub fn deserialize_actions(
    serialized: Vec<u8>,
    scene_root: &Gd<Node>,
    node_cache: &NodeCache,
) -> Option<Vec<Action>> {
    if serialized.is_empty() {
        return None;
    }

    let de: Vec<LeanAction> = match decode_from_slice(&serialized, standard()) {
        Ok(de) => de.0,
        Err(e) => {
            log::error!(
                "Failed to deserialize actions ({}): {:?}",
                serialized.len(),
                e
            );
            return None;
        }
    };

    Some(
        de.iter()
            .filter_map(|de_action| Action::deserialize(de_action, scene_root, node_cache))
            .collect(),
    )
}

impl Action {
    pub fn deserialize_from_bytes(
        serialized: Vec<u8>,
        scene_root: &Gd<Node>,
        node_cache: &NodeCache,
    ) -> Option<Action> {
        let de: LeanAction = match decode_from_slice(&serialized, standard()) {
            Ok(de) => de.0,
            Err(e) => {
                log::error!("Failed to decode action: {:?}", e);
                return None;
            }
        };

        Action::deserialize(&de, scene_root, node_cache)
    }

    pub fn deserialize(
        de: &LeanAction,
        scene_root: &Gd<Node>,
        node_cache: &NodeCache,
    ) -> Option<Action> {
        let node_path = node_cache.get_node_path_or_empty(de.node_index);
        match scene_root.get_node_or_null(&node_path) {
            Some(node) => {
                let cuid = match node.get_meta("cuid").get_type() {
                    VariantType::STRING => GString::from(node.get_meta("cuid").to_string()),
                    _ => {
                        log::error!(
                            "Failed to get cuid from node meta when deserializing action: {:?}",
                            de
                        );
                        return None;
                    }
                };

                let mut data = Dictionary::new();
                match de.operation {
                    Operation::MoveNode => {
                        if let Some(movement) = de.vector3s.get(0) {
                            data.set("movement", *movement);
                        }
                        if let Some(position) = de.vector3s.get(1) {
                            data.set("position", *position);
                        }
                    }
                    _ => {}
                }

                let result = Action {
                    cuid,
                    handle: de.handle,
                    node,
                    operation: de.operation.clone(),
                    data,
                };

                Some(result)
            }
            None => {
                log::error!(
                    "Failed to find node in scene tree when deserializing action: {:?}",
                    de
                );
                None
            }
        }
    }
}

pub fn action_to_lean_action(action: &Action, node_cache: &mut NodeCache) -> LeanAction {
    let mut vector3s = Vec::<Vector3>::new();

    match action.operation {
        Operation::MoveNode => {
            if let Some(movement) = extract_from_dict(&action.data, "movement", true) {
                vector3s.push(movement);
            }
        }
        Operation::TeleportNode => {
            if let Some(position) = extract_from_dict(&action.data, "position", true) {
                vector3s.push(position);
            }
        }
        _ => {}
    }

    let node_path = action.node.get_path().to_string();
    let node_index = node_cache.add_or_get_node_path(node_path);

    LeanAction {
        operation: action.operation.clone(),
        handle: action.handle,
        vector3s,
        node_index,
    }
}
