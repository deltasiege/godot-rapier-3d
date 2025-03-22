use std::cmp::Ordering;

use godot::prelude::*;

use crate::{
    nodes::{
        Identifiable, RapierArea3D, RapierCollisionShape3D, RapierKinematicCharacter3D,
        RapierPIDCharacter3D, RapierRigidBody3D, RapierStaticBody3D,
    },
    World,
};

#[derive(GodotConvert, Var, Export, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[godot(via = GString)]
pub enum Operation {
    AddNode,
    ConfigureNode,
    RemoveNode,
    MoveNode,
}

#[derive(Debug, Clone)]
pub struct Action {
    pub cuid: GString,
    pub node: Gd<Node3D>,
    pub operation: Operation,
    pub data: Dictionary,
    pub replicate: bool, // Copy forward during rollbacks to predict future actions
}

impl Action {
    pub fn new(cuid: GString, node: Gd<Node3D>, operation: Operation, data: Dictionary) -> Self {
        Self {
            cuid,
            node,
            operation,
            data,
            replicate: false,
        }
    }

    pub fn new_full(
        cuid: GString,
        node: Gd<Node3D>,
        operation: Operation,
        data: Dictionary,
        replicate: bool,
    ) -> Self {
        Self {
            cuid,
            node,
            operation,
            data,
            replicate,
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
// sorting = by cuid -> by operation -> (within move operations, magnitude of movement)
pub fn sort_actions(actions: Vec<&Action>) -> Vec<&Action> {
    let mut sorted_actions = actions;
    sorted_actions.sort();
    sorted_actions
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

/// Sort actions by CUID and then by operation and then by magnitude of movement
impl Ord for Action {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.cuid != other.cuid {
            self.cuid.cmp(&other.cuid)
        } else if self.operation != other.operation {
            self.operation.cmp(&other.operation)
        } else if self.operation == Operation::MoveNode && other.operation == Operation::MoveNode {
            if !self.data.contains_key("movement") || !other.data.contains_key("movement") {
                return Ordering::Equal;
            }
            let self_movement = extract_vec3(&self.data, "movement", false);
            let other_movement = extract_vec3(&other.data, "movement", false);
            if let Some(self_movement) = self_movement {
                if let Some(other_movement) = other_movement {
                    self_movement
                        .length()
                        .partial_cmp(&other_movement.length())
                        .unwrap()
                } else {
                    Ordering::Greater
                }
            } else {
                Ordering::Less
            }
        } else {
            Ordering::Equal
        }
    }
}

pub fn extract_vec3(data: &Dictionary, key: &str, silent: bool) -> Option<Vector3> {
    if let Some(variant) = data.get(key) {
        match Vector3::try_from_variant(&variant) {
            Ok(vec3) => Some(vec3),
            Err(e) => {
                if !silent {
                    log::error!(
                        "Expecting Vector3 at key '{}' in data {:?}. Error: {}",
                        key,
                        data,
                        e
                    );
                }
                None
            }
        }
    } else {
        if !silent {
            log::error!("Missing Vector3 at key '{}' in data: {:?}", key, data);
        }
        None
    }
}

impl PartialOrd for Action {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Action {}
impl PartialEq for Action {
    fn eq(&self, other: &Self) -> bool {
        self.cuid == other.cuid && self.operation == other.operation && self.data == other.data
    }
}
