use crate::rigid_body::RapierRigidBody3D;
use godot::builtin::Quaternion as GQuaternion;
use godot::builtin::Vector3 as GVector;
use godot::engine::Node3D;
use godot::obj::Base;
use godot::prelude::*;
use rapier3d::math::Vector as RVector;
use rapier3d::prelude::*;

pub struct RapierPhysicsPipeline {
    pub rigid_body_ids: Dictionary, // gd node instance_id <-> rapier rb_handle_to_id()

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

        self.sync_active_body_positions();
    }

    // Syncs Godot position to Rapier position
    pub fn sync_active_body_positions(&mut self) {
        let active_dynamic_bodies = self.island_manager.active_dynamic_bodies();

        for active_body_handle in active_dynamic_bodies {
            let rb = match self.rigid_body_set.get(*active_body_handle) {
                Some(rb) => rb,
                None => {
                    godot_error!("Could not find active body {:?}", active_body_handle);
                    continue;
                }
            };

            let rb_pos = rb.translation();
            let rb_rot = rb.rotation().quaternion().coords;
            let g_pos = GVector::new(rb_pos.x, rb_pos.y, rb_pos.z);
            let g_rot = GQuaternion::new(rb_rot.x, rb_rot.y, rb_rot.z, rb_rot.w);

            godot_print!("Syncing body {:?} to {:?}", rb_rot, g_rot);

            let id = crate::utils::rb_handle_to_id(*active_body_handle);
            let instance_id_var: Variant = match self.rigid_body_ids.find_key_by_value(id.clone()) {
                Some(instance_id) => instance_id,
                None => {
                    godot_error!(
                        "Could not find instance_id for active body {:?}",
                        id.clone()
                    );
                    continue;
                }
            };

            let instance_id_int = instance_id_var.to_string().parse::<i64>().unwrap();
            let instance_id = InstanceId::from_i64(instance_id_int);

            let mut node: Gd<RapierRigidBody3D> = match Gd::try_from_instance_id(instance_id) {
                Ok(node) => node,
                _ => {
                    godot_error!("Could not find node for active body {:?}", instance_id);
                    continue;
                }
            };

            node.bind_mut().base_mut().set_notify_transform(false);
            node.bind_mut().base_mut().set_global_position(g_pos);
            node.bind_mut().base_mut().set_quaternion(g_rot);
            node.bind_mut().base_mut().set_notify_transform(true);
        }
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
