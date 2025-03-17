use rapier3d::dynamics::{
    CCDSolver, ImpulseJointSet, IntegrationParameters, IslandManager, MultibodyJointSet,
    RigidBodySet,
};
use rapier3d::geometry::{ColliderSet, DefaultBroadPhase, NarrowPhase};
use rapier3d::math::{Real, Vector};
use rapier3d::pipeline::{PhysicsHooks, PhysicsPipeline, QueryPipeline};
use serde::{Deserialize, Serialize};

use crate::World;

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
}

pub fn pack_snapshot(world: &World) -> bincode::Result<Vec<u8>> {
    let raw = DeserializedPhysicsSnapshot {
        timestep_id: world.state.timestep_id,
        broad_phase: world.physics.broad_phase.clone(),
        narrow_phase: world.physics.narrow_phase.clone(),
        island_manager: world.physics.islands.clone(),
        bodies: world.physics.bodies.clone(),
        colliders: world.physics.colliders.clone(),
        impulse_joints: world.physics.impulse_joints.clone(),
        multibody_joints: world.physics.multibody_joints.clone(),
    };

    bincode::serialize(&raw)
}

fn unpack_snapshot(bytes: Vec<u8>) -> bincode::Result<DeserializedPhysicsSnapshot> {
    let deserialized: DeserializedPhysicsSnapshot = bincode::deserialize(&bytes)?;
    Ok(deserialized)
}

pub fn restore_snapshot(world: &mut World, bytes: Vec<u8>) -> bincode::Result<()> {
    let deserialized = unpack_snapshot(bytes)?;
    world.state.timestep_id = deserialized.timestep_id;
    world.physics.broad_phase = deserialized.broad_phase;
    world.physics.narrow_phase = deserialized.narrow_phase;
    world.physics.islands = deserialized.island_manager;
    world.physics.bodies = deserialized.bodies;
    world.physics.colliders = deserialized.colliders;
    world.physics.impulse_joints = deserialized.impulse_joints;
    world.physics.multibody_joints = deserialized.multibody_joints;
    Ok(())
}
