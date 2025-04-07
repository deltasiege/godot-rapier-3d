use bincode::config::standard;
use bincode::serde::{decode_from_slice, encode_to_vec};
use rapier3d::parry::utils::hashmap::HashMap;
use rapier3d::prelude::*;
use serde::{Deserialize, Serialize};

use crate::nodes::NodeBlueprint;
use crate::types::*;
use crate::World;

#[derive(Serialize, Deserialize, Clone)]
pub struct WorldSnapshot {
    pub tick: usize,
    pub broad_phase: DefaultBroadPhase,
    pub narrow_phase: NarrowPhase,
    pub island_manager: IslandManager,
    pub bodies: RigidBodySet,
    pub colliders: ColliderSet,
    pub nodes: HashMap<GRUID, Option<NodeBlueprint>>, // All known alive & dead nodes at the snapshot tick
    pub impulse_joints: ImpulseJointSet,
    pub multibody_joints: MultibodyJointSet,
}

impl WorldSnapshot {
    pub fn from_world(world: &World) -> Self {
        let nodes = get_snapshottable_nodes(world);
        let colliders = get_snapshottable_colliders(world, &nodes);

        Self {
            tick: world.time.tick,
            broad_phase: world.physics.broad_phase.clone(),
            narrow_phase: world.physics.narrow_phase.clone(),
            island_manager: world.physics.islands.clone(),
            nodes,
            bodies: world.physics.bodies.clone(),
            colliders,
            impulse_joints: world.physics.impulse_joints.clone(),
            multibody_joints: world.physics.multibody_joints.clone(),
        }
    }

    pub fn apply_to_world(&self, world: &mut World, overwrite_tick: bool) {
        // Overwrite world and node_db tick if requested
        if overwrite_tick {
            world.time.tick = self.tick;
            world.node_db.rollback_to_tick(self.tick); // Rollback node database to the snapshot tick
        }

        // Overwrite world physics data
        world.physics.broad_phase = self.broad_phase.clone();
        world.physics.narrow_phase = self.narrow_phase.clone();
        world.physics.islands = self.island_manager.clone();
        world.physics.bodies = self.bodies.clone();
        world.physics.impulse_joints = self.impulse_joints.clone();
        world.physics.multibody_joints = self.multibody_joints.clone();

        // Don't overwrite colliders excluded from snapshots
        for (handle, collider) in self.colliders.iter() {
            if let Some(collider) = world.physics.colliders.get_mut(handle) {
                *collider = collider.clone();
            } else {
                world.physics.colliders.insert(collider.clone());
            }
        }

        // Insert nodes in the snapshot into the node database
        world.node_db.set_nodes(self.nodes.clone(), self.tick);
    }

    pub fn try_to_bytes(&self) -> Option<Vec<u8>> {
        encode_snapshot_to_bytes(self)
    }

    pub fn try_from_bytes(bytes: &Vec<u8>) -> Option<Self> {
        decode_snapshot_from_bytes(bytes)
    }
}

fn encode_snapshot_to_bytes(snapshot: &WorldSnapshot) -> Option<Vec<u8>> {
    match encode_to_vec(snapshot, standard()) {
        Ok(bytes) => Some(bytes),
        Err(e) => {
            log::error!("Failed to encode snapshot: {:?}", e);
            None
        }
    }
}

fn decode_snapshot_from_bytes(bytes: &Vec<u8>) -> Option<WorldSnapshot> {
    let deserialized = decode_from_slice(&bytes, standard());
    match deserialized {
        Ok((snapshot, _size)) => Some(snapshot),
        Err(e) => {
            log::error!("Failed to decode snapshot: {:?}", e);
            None
        }
    }
}

/// Returns all nodes available on the given tick that can be included in snapshots.
/// Note that despawn markers are included in the result. We only want to exclude expensive colliders.
pub fn get_snapshottable_nodes(world: &World) -> HashMap<GRUID, Option<NodeBlueprint>> {
    let mut map = world.node_db.get_nodes(world.time.tick);
    map.retain(|_, node_blueprint| {
        if let Some(node_blueprint) = node_blueprint {
            return node_blueprint.snapshottable;
        }
        true
    });
    map
}

/// Returns the list of current colliders that can be included in snapshots
fn get_snapshottable_colliders(
    world: &World,
    snapshottable_nodes: &HashMap<GRUID, Option<NodeBlueprint>>,
) -> ColliderSet {
    let mut colliders = ColliderSet::new();
    for (_, blueprint) in snapshottable_nodes {
        if let Some(blueprint) = blueprint {
            match blueprint.rapier_builder {
                RapierBuilder::Collider(_) => {
                    let handle = ColliderHandle::from_raw_parts(
                        blueprint.rapier_handle.0,
                        blueprint.rapier_handle.1,
                    );
                    if let Some(collider) = world.physics.colliders.get(handle) {
                        colliders.insert(collider.clone());
                    }
                }
                _ => {}
            }
        }
    }
    colliders
}
