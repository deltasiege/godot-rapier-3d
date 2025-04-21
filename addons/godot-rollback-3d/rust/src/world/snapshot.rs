use bincode::config::standard;
use bincode::serde::{decode_from_slice, encode_to_vec};
use rapier3d::prelude::*;
use serde::{Deserialize, Serialize};

use crate::types::*;
use crate::utils::{encode_or_none, get_hash};
use crate::World;

#[derive(Serialize, Deserialize, Clone)]
pub struct WorldSnapshot {
    pub tick: Tick,
    pub world_state: WorldState,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WorldState {
    pub broad_phase: DefaultBroadPhase,
    pub narrow_phase: NarrowPhase,
    pub island_manager: IslandManager,
    pub bodies: RigidBodySet,
    pub colliders: ColliderSet,
    pub nodes: NodeMap, // All known alive & dead nodes at the snapshot tick
    pub impulse_joints: ImpulseJointSet,
    pub multibody_joints: MultibodyJointSet,
}

impl std::fmt::Display for WorldState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "WorldState {{ island_manager: {:?}, bodies: {:?}, colliders: {:?}, nodes: {:?}, impulse_joints: {:?}, multibody_joints: {:?} }}",
            self.island_manager.active_kinematic_bodies().len() + self.island_manager.active_dynamic_bodies().len(),
            self.bodies.len(),
            self.colliders.len(),
            self.nodes.len(),
            self.impulse_joints.len(),
            self.multibody_joints.multibodies().count()
        )
    }
}

impl WorldSnapshot {
    pub fn from_world(world: &World) -> Self {
        Self {
            tick: world.time.tick,

            world_state: WorldState {
                broad_phase: world.physics.broad_phase.clone(),
                narrow_phase: world.physics.narrow_phase.clone(),
                island_manager: world.physics.islands.clone(),
                nodes: world.node_db.nodes.clone(),
                bodies: world.physics.bodies.clone(),
                colliders: get_snapshottable_colliders(world),
                impulse_joints: world.physics.impulse_joints.clone(),
                multibody_joints: world.physics.multibody_joints.clone(),
            },
        }
    }

    // Hash all fields except tick
    pub fn get_hash(&self) -> Option<u64> {
        let ser = encode_or_none(&self.world_state)?;
        log::debug!("Hashing: {}", self.world_state);
        Some(get_hash(&ser))
    }

    pub fn apply_to_world(&self, world: &mut World, overwrite_tick: bool) {
        // Overwrite world and node_db tick if requested
        if overwrite_tick {
            world.time.tick = self.tick;
        }

        // Overwrite world physics data
        world.physics.broad_phase = self.world_state.broad_phase.clone();
        world.physics.narrow_phase = self.world_state.narrow_phase.clone();
        world.physics.islands = self.world_state.island_manager.clone();
        world.physics.bodies = self.world_state.bodies.clone();
        world.physics.impulse_joints = self.world_state.impulse_joints.clone();
        world.physics.multibody_joints = self.world_state.multibody_joints.clone();

        // Don't overwrite colliders excluded from snapshots
        for (handle, collider) in self.world_state.colliders.iter() {
            if let Some(collider) = world.physics.colliders.get_mut(handle) {
                *collider = collider.clone();
            } else {
                world.physics.colliders.insert(collider.clone());
            }
        }

        // Overwrite node database with all nodes in the snapshot
        world
            .node_db
            .overwrite_nodes(self.world_state.nodes.clone());
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

/// Returns the list of current colliders that can be included in snapshots
fn get_snapshottable_colliders(world: &World) -> ColliderSet {
    let mut colliders = ColliderSet::new();
    for (_, node_data) in world.node_db.nodes.iter() {
        if !node_data.blueprint.snapshottable {
            continue;
        }

        if let Some(collider) = node_data.get_collider(world) {
            colliders.insert(collider.clone());
        }
    }
    colliders
}
