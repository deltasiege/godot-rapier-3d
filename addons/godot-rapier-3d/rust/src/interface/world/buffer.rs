use godot::prelude::*;

use crate::{
    nodes::{
        Identifiable, RapierArea3D, RapierCollisionShape3D, RapierKinematicCharacter3D,
        RapierPIDCharacter3D, RapierRigidBody3D, RapierStaticBody3D,
    },
    World,
};

#[derive(GodotConvert, Var, Export, Debug, Clone)]
#[godot(via = GString)]
pub enum Operation {
    AddNode,
    RemoveNode,
    MoveNode,
    ConfigureNode,
}

#[derive(Clone)]
pub struct Action {
    pub cuid: GString,
    pub node: Gd<Node3D>,
    pub operation: Operation,
    pub data: Dictionary,
}

impl Action {
    pub fn new(cuid: GString, node: Gd<Node3D>, operation: Operation, data: Dictionary) -> Self {
        Self {
            cuid,
            node,
            operation,
            data,
        }
    }
}

/// Constructs a new action and then adds it to the world buffer at the current timestep
pub fn ingest_action(node: Gd<Node3D>, operation: Operation, data: Dictionary, world: &mut World) {
    if let Some(cuid) = extract_cuid(node.clone()) {
        let action = Action::new(cuid, node, operation, data);
        let timestep_id = world.state.timestep_id;
        world.buffer.insert_action(action, timestep_id);
    }
}

// NOTE sorting should happen right before stepping the world (so that it only happens once per timestep)
// sorting = by cuid -> by operation -> (within move operations, magnitude of )
pub fn sort_actions(actions: Vec<Action>) -> Vec<Action> {
    actions
}

fn extract_cuid(node: Gd<Node3D>) -> Option<GString> {
    match node.get_class().to_string().as_str() {
        "RapierArea3D" => Some(node.cast::<RapierArea3D>().bind().get_cuid()),
        "RapierCollisionShape3D" => Some(node.cast::<RapierCollisionShape3D>().bind().get_cuid()),
        "RapierKinematicCharacter3D" => {
            Some(node.cast::<RapierKinematicCharacter3D>().bind().get_cuid())
        }
        "RapierPIDCharacter3D" => Some(node.cast::<RapierPIDCharacter3D>().bind().get_cuid()),
        "RapierRigidBody3D" => Some(node.cast::<RapierRigidBody3D>().bind().get_cuid()),
        "RapierStaticBody3D" => Some(node.cast::<RapierStaticBody3D>().bind().get_cuid()),
        _ => {
            log::error!(
                "Node class not recognized: {}",
                node.get_class().to_string()
            );
            None
        }
    }
}
