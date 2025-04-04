use bincode::{
    config::standard,
    serde::{decode_from_slice, encode_to_vec},
};
use godot::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    actions::{Action, Operation},
    utils::extract_from_dict,
    LookupTable,
};

// Struct sent over the network
#[derive(Serialize, Deserialize, Debug)]
pub struct LeanAction {
    pub operation: Operation,
    pub cuid: String,
    pub vector3s: Vec<Vector3>,
}

/// Serialize the given actions vector into byte vector
pub fn serialize_actions(actions: &Vec<Action>) -> Option<Vec<u8>> {
    if actions.is_empty() {
        return None;
    }

    let actions: Vec<LeanAction> = actions
        .iter()
        .map(|action| action_to_lean_action(action))
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
    lookup_table: &LookupTable,
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
            .filter_map(|de_action| Action::deserialize(de_action, scene_root, lookup_table))
            .collect(),
    )
}

impl Action {
    pub fn deserialize_from_bytes(
        serialized: Vec<u8>,
        scene_root: &Gd<Node>,
        lookup_table: &LookupTable,
    ) -> Option<Action> {
        let de: LeanAction = match decode_from_slice(&serialized, standard()) {
            Ok(de) => de.0,
            Err(e) => {
                log::error!("Failed to decode action: {:?}", e);
                return None;
            }
        };

        Action::deserialize(&de, scene_root, lookup_table)
    }

    pub fn deserialize(
        de: &LeanAction,
        scene_root: &Gd<Node>,
        lookup_table: &LookupTable,
    ) -> Option<Action> {
        let cuid = GString::from(de.cuid.clone());
        let handle = lookup_table.get_rapier_handle(&cuid);
        let node = match lookup_table.get_node_from_cuid(&cuid, scene_root) {
            Some(node) => node,
            None => {
                log::error!(
                    "Failed to find node in scene tree when deserializing action: {:?}",
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
            handle,
            // node,
            operation: de.operation.clone(),
            data,
        };

        Some(result)
    }
}

pub fn action_to_lean_action(action: &Action) -> LeanAction {
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

    LeanAction {
        operation: action.operation.clone(),
        cuid: action.cuid.to_string(),
        vector3s,
    }
}
