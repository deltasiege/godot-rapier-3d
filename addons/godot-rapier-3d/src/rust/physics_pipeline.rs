use crate::rigid_body::RapierRigidBody3D;
use godot::engine::Node3D;
use godot::obj::Base;
use godot::prelude::*;
use rapier3d::math::Vector as RVector;
use rapier3d::prelude::*;

pub struct RapierPhysicsPipeline {
    pub rigid_body_ids: Dictionary, // instance_id <-> RigidBodyHandle::into_raw_parts

    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    gravity: RVector<Real>,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    query_pipeline: QueryPipeline,
    physics_hooks: (),
    event_handler: (),
}

impl RapierPhysicsPipeline {
    pub fn new() -> Self {
        Self {
            rigid_body_ids: Dictionary::new(),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            gravity: RVector::new(0.0, -9.81, 0.0),
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),
            physics_hooks: (),
            event_handler: (),
        }
    }

    pub fn step(&mut self) {
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &self.physics_hooks,
            &self.event_handler,
        );
    }

    pub fn register_rigid_body(&mut self, rrb: &mut RapierRigidBody3D) -> RigidBodyHandle {
        let instance_id = rrb.base().instance_id().to_i64();
        let mut rigid_body: RigidBody = rrb.build();
        rigid_body.user_data = u128::try_from(instance_id).unwrap();
        let handle = self.rigid_body_set.insert(rigid_body);
        let id = crate::utils::rb_handle_to_id(handle);
        self.rigid_body_ids.set(instance_id, id);
        handle
    }

    pub fn unregister_rigid_body(&mut self, rrb: &mut RapierRigidBody3D) {
        let instance_id = rrb.base().instance_id().to_i64();
        let handle = rrb.handle; // TODO - check if this is valid somehow?
        self.rigid_body_set.remove(
            handle,
            &mut self.island_manager,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            true, // true = also remove colliders
        );
        self.rigid_body_ids.remove(instance_id);
    }
}
