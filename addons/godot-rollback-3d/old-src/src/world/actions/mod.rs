mod add_remove_nodes;
mod modify_nodes;

use godot::prelude::*;

use add_remove_nodes::*;
use modify_nodes::*;

use crate::actions::{Action, Operation};
use crate::utils::extract_from_dict;
use crate::world::PhysicsState;

pub fn apply_actions_to_world(actions: &mut Vec<Action>, physics: &mut PhysicsState) {
    actions.sort();
    for action in actions.iter() {
        match action.node.clone().try_cast::<Node3D>() {
            Ok(node) => {
                apply_action_to_world(action, node, physics);
            }
            Err(e) => {
                log::error!(
                    "Failed to cast node {:?} to Node3D. Skipping action {:?}. Error: {:?}",
                    action.node,
                    action,
                    e
                );
            }
        }
    }
}

fn apply_action_to_world(action: &Action, node: Gd<Node3D>, physics: &mut PhysicsState) {
    match action.operation {
        Operation::AddNode => {
            add_node_to_world(&node, physics);
            configure_node(&node); // TODO also configure node
        }
        Operation::RemoveNode => {
            remove_node_from_world(node, physics);
        }
        Operation::ConfigureNode => {
            configure_node(&node);
        }
        Operation::MoveNode => {
            if let Some(movement) = extract_from_dict(&action.data, "movement", true) {
                move_node(node, movement, physics);
            }
        }
        Operation::TeleportNode => {
            if let Some(position) = extract_from_dict(&action.data, "position", true) {
                teleport_node(node, position, physics);
            }
        }
    }
}
