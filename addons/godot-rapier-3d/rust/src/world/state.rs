use rapier3d::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{LookupTable, World};

pub struct PhysicsState {
    pub islands: IslandManager,
    pub broad_phase: DefaultBroadPhase,
    pub narrow_phase: NarrowPhase,
    pub bodies: RigidBodySet,
    pub colliders: ColliderSet,
    pub impulse_joints: ImpulseJointSet,
    pub multibody_joints: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
    pub pipeline: PhysicsPipeline,
    pub query_pipeline: QueryPipeline,
    pub integration_parameters: IntegrationParameters,
    pub gravity: Vector<Real>,
    pub hooks: Box<dyn PhysicsHooks>,
    pub lookup_table: LookupTable,
}

impl Default for PhysicsState {
    fn default() -> Self {
        Self::new()
    }
}

impl PhysicsState {
    pub fn new() -> Self {
        Self {
            islands: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            impulse_joints: ImpulseJointSet::new(),
            multibody_joints: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            pipeline: PhysicsPipeline::new(),
            query_pipeline: QueryPipeline::new(),
            integration_parameters: IntegrationParameters::default(),
            gravity: Vector::y() * -9.81,
            hooks: Box::new(()),
            lookup_table: LookupTable::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DeserializedPhysicsSnapshot {
    pub timestep_id: usize,
    pub broad_phase: DefaultBroadPhase,
    pub narrow_phase: NarrowPhase,
    pub island_manager: IslandManager,
    pub bodies: RigidBodySet,
    pub colliders: ColliderSet,
    pub impulse_joints: ImpulseJointSet,
    pub multibody_joints: MultibodyJointSet,
    pub lookup_table: LookupTable,
}

pub fn pack_snapshot(world: &World) -> bincode::Result<Vec<u8>> {
    // NOTE: only cheap colliders are serialized
    let mut colliders = ColliderSet::new();
    for raw_handle in &world.physics.lookup_table.snapshot_colliders {
        let handle = ColliderHandle::from_raw_parts(raw_handle.0, raw_handle.1);
        if let Some(collider) = world.physics.colliders.get(handle) {
            colliders.insert(collider.clone());
        }
    }

    let output = DeserializedPhysicsSnapshot {
        timestep_id: world.state.timestep_id,
        broad_phase: world.physics.broad_phase.clone(),
        narrow_phase: world.physics.narrow_phase.clone(),
        island_manager: world.physics.islands.clone(),
        bodies: world.physics.bodies.clone(),
        colliders: colliders,
        impulse_joints: world.physics.impulse_joints.clone(),
        multibody_joints: world.physics.multibody_joints.clone(),
        lookup_table: world.physics.lookup_table.clone(),
    };

    bincode::serialize(&output)
}

fn unpack_snapshot(bytes: Vec<u8>) -> Option<DeserializedPhysicsSnapshot> {
    let deserialized: bincode::Result<DeserializedPhysicsSnapshot> = bincode::deserialize(&bytes);
    match deserialized {
        Ok(snapshot) => Some(snapshot),
        Err(e) => {
            log::error!("Failed to unpack snapshot: {:?}", e);
            None
        }
    }
}

/// Overwrite the current state of the given world to the given snapshot state
pub fn restore_snapshot(world: &mut World, bytes: Vec<u8>) {
    if let Some(deserialized) = unpack_snapshot(bytes) {
        world.state.timestep_id = deserialized.timestep_id;
        world.physics.broad_phase = deserialized.broad_phase;
        world.physics.narrow_phase = deserialized.narrow_phase;
        world.physics.islands = deserialized.island_manager;
        world.physics.bodies = deserialized.bodies;
        world.physics.impulse_joints = deserialized.impulse_joints;
        world.physics.multibody_joints = deserialized.multibody_joints;

        // Carefully handle colliders to not overwrite expensive "eternal" ones
        for (handle, collider) in deserialized.colliders.iter() {
            if let Some(collider) = world.physics.colliders.get_mut(handle) {
                *collider = collider.clone();
            } else {
                world.physics.colliders.insert(collider.clone());
            }
        }

        world.physics.lookup_table = deserialized.lookup_table;
    }
}
