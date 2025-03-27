use bincode::{Decode, Encode};
use godot::prelude::*;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

use crate::{interface::get_sync_singleton, nodes::*, World};

#[derive(
    GodotConvert,
    Var,
    Export,
    Debug,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Serialize,
    Deserialize,
    Encode,
    Decode,
)]
#[godot(via = GString)]
pub enum Operation {
    AddNode,
    ConfigureNode,
    RemoveNode,
    MoveNode,
    TeleportNode,
}

#[derive(Debug, Clone)]
/// Struct stored in world buffer. Identical to Action but
/// cuid and handle are extracted from the node reference
/// to avoid re-fetching multiple times if the action is applied multiple times
pub struct Action {
    pub cuid: GString,
    pub handle: Option<(u32, u32)>,
    pub node: Gd<Node>,
    pub operation: Operation,
    pub data: Dictionary,
}

impl Action {
    pub fn new(
        cuid: GString,
        handle: Option<(u32, u32)>,
        node: Gd<Node>,
        operation: Operation,
        data: Dictionary,
    ) -> Self {
        Self {
            cuid,
            handle,
            node,
            operation,
            data,
        }
    }
}

/// Constructs a new action and then adds it to the world buffer at the current timestep
pub fn ingest_local_action(
    node: Gd<Node>,
    operation: Operation,
    data: Dictionary,
    world: &mut World,
) {
    if let Some((cuid, handle)) = extract_ids(node.clone()) {
        let action = Action::new(cuid, handle, node, operation, data);
        let timestep_id = world.state.timestep_id;

        match get_sync_singleton() {
            Some(mut sync) => {
                sync.bind_mut().record_local_action(timestep_id, &action);
                world.buffer.insert_action(action, timestep_id);
            }
            None => {
                log::error!(
                    "Failed to ingest_local_action because GR3DSync singleton was not found"
                );
            }
        }
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
