use godot::prelude::*;

use crate::interface::GR3D;
use crate::nodes::{NodeBlueprint, NodeData};
use crate::types::*;
use crate::world::PhysicsState;

/// Begins spawn process and returns stringified GRUIDs of the nodes that will be spawned.
pub fn spawn(
    gr3d: &mut GR3D,
    peer_index: PeerIndex,
    name: String,
    parent: Gd<Node>,
    resource_path: String,
    transform: Transform3D,
) -> Array<GString> {
    let spawn_request = SpawnRequest {
        peer_index,
        name,
        parent,
        resource_path,
        transform,
    };
    match try_spawn(gr3d, spawn_request) {
        Some(gruid_strings) => gruid_strings,
        None => Array::new(),
    }
}

pub fn register_ambient_node(gr3d: &mut GR3D, node: Gd<Node>) -> Array<GString> {
    // UP TO: - refer to spawning.md ambient nodes section and note under ## Spawning / node presence management
}

/// Option compatible version of spawn function.
fn try_spawn(gr3d: &mut GR3D, spawn_request: SpawnRequest) -> Option<Array<GString>> {
    if !gr3d.network.started {
        log::error!(
            "Cannot spawn node '{}' before network has started",
            spawn_request.name
        );
        return None;
    }

    if spawn_request.peer_index == 0 {
        log::error!(
            "Cannot spawn node '{}' under ambient peer (0). peer_index must be 1 or greater.",
            spawn_request.name
        );
        return None;
    }

    let gruids = gr3d.world.node_db.on_spawn_request(spawn_request)?;
    Some(gruids)
}

/// Collection of arguments needed to spawn a node.
pub struct SpawnRequest {
    pub peer_index: PeerIndex,
    pub name: String,
    pub parent: Gd<Node>,
    pub resource_path: String,
    pub transform: Transform3D,
}

/// Creates appropriate Rapier object from the given blueprint and adds it to the Rapier world.
/// Returns the created RapierHandle
pub fn rapier_spawn_from_blueprint(
    blueprint: NodeBlueprint,
    physics: &mut PhysicsState,
) -> RapierHandle {
    let node_name = blueprint.get_node_name();
    match blueprint.rapier_builder {
        RapierBuilder::RigidBody(mut rb) => {
            rb.set_position(blueprint.spawn_isometry, true);
            let handle = physics.bodies.insert(rb);
            let num_child_colliders = blueprint.child_colliders.len();

            for collider in blueprint.child_colliders {
                match collider.rapier_builder {
                    RapierBuilder::Collider(collider) => {
                        physics
                            .colliders
                            .insert_with_parent(collider, handle, &mut physics.bodies);
                    }
                    _ => {
                        log::error!(
                            "Child collider {} does not have a collider builder",
                            collider
                        );
                    }
                }
            }

            log::trace!(
                "Spawned Rapier RigidBody: '{}' {:?} with {} child colliders at {}",
                node_name,
                handle,
                num_child_colliders,
                blueprint.spawn_isometry.translation,
            );

            RapierHandle::from_rigid_body_handle(handle)
        }
        RapierBuilder::Collider(collider) => {
            let handle = physics.colliders.insert(collider);
            log::trace!("Spawned Rapier Collider: '{}' {:?}", node_name, handle);
            RapierHandle::from_collider_handle(handle)
        }
    }
}

/// Removes the rapier_handle specified in given node_data from the Rapier world.
pub fn rapier_despawn_from_node_data(node_data: NodeData, physics: &mut PhysicsState) {
    match node_data.blueprint.rapier_builder {
        RapierBuilder::RigidBody(_) => {
            let handle = node_data.rapier_handle.to_rigid_body_handle();
            physics.bodies.remove(
                handle,
                &mut physics.islands,
                &mut physics.colliders,
                &mut physics.impulse_joints,
                &mut physics.multibody_joints,
                true,
            );
        }
        RapierBuilder::Collider(_) => {
            let handle = node_data.rapier_handle.to_collider_handle();
            physics
                .colliders
                .remove(handle, &mut physics.islands, &mut physics.bodies, false);
        }
    }
}
