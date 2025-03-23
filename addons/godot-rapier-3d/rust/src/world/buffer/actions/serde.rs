use bincode::config::standard;
use godot::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    utils::extract_from_dict,
    world::buffer::{Action, Operation, WorldBuffer},
};

// Struct sent over the network
#[derive(Serialize, Deserialize, Debug)]
pub struct DeserializedAction {
    pub operation: Operation,
    pub handle: Option<(u32, u32)>,
    pub vector3s: Vec<Vector3>,
    pub strings: Vec<String>,
}

pub fn serialize_actions(buffer: &WorldBuffer, timestep_id: usize) -> Option<(Vec<u8>, usize)> {
    let actions: Vec<DeserializedAction> = match buffer.get_actions(timestep_id) {
        Some(actions) => actions
            .iter()
            .map(|action| DeserializedAction::from(action))
            .collect(),
        None => {
            // No actions to serialize
            return None;
        }
    };

    let count = actions.len();

    match bincode::serde::encode_to_vec(actions, standard()) {
        Ok(serialized) => Some((serialized, count)),
        Err(e) => {
            log::error!(
                "Failed serializing actions on timestep: {} Error: {:?}",
                timestep_id,
                e
            );
            None
        }
    }
}

pub fn deserialize_actions(serialized: Vec<u8>, scene_root: Gd<Node>) -> Vec<Action> {
    let de: Vec<DeserializedAction> =
        match bincode::serde::decode_from_slice(&serialized, standard()) {
            Ok(de) => de.0,
            Err(e) => {
                log::error!("Failed to decode actions: {:?}", e);
                return Vec::new();
            }
        };

    de.iter()
        .filter_map(|de_action| Action::deserialize(de_action, scene_root.clone()))
        .collect()
}

impl Action {
    pub fn serialize(&self) -> Result<Vec<u8>, bincode::error::EncodeError> {
        Ok(bincode::serde::encode_to_vec(
            DeserializedAction::from(self),
            standard(),
        )?)
    }

    pub fn deserialize_from_bytes(serialized: Vec<u8>, scene_root: Gd<Node>) -> Option<Action> {
        let de: DeserializedAction =
            match bincode::serde::decode_from_slice(&serialized, standard()) {
                Ok(de) => de.0,
                Err(e) => {
                    log::error!("Failed to decode action: {:?}", e);
                    return None;
                }
            };

        Action::deserialize(&de, scene_root)
    }

    pub fn deserialize(de: &DeserializedAction, scene_root: Gd<Node>) -> Option<Action> {
        match scene_root.get_node_or_null(&de.strings[0]) {
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

impl From<&Action> for DeserializedAction {
    fn from(action: &Action) -> Self {
        let mut vector3s = Vec::<Vector3>::new();
        let mut strings = Vec::<String>::new();

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

        strings.push(action.node.get_path().to_string());

        DeserializedAction {
            operation: action.operation.clone(),
            handle: action.handle,
            vector3s,
            strings,
        }
    }
}
