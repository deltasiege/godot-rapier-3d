use crate::collider::RapierCollider3D;
use crate::rigid_body::RapierRigidBody3D;
use godot::prelude::*;
use rapier3d::math::Vector as RVector;
use rapier3d::prelude::*;

pub struct RapierPhysicsPipeline {
    pub rigid_body_ids: Dictionary, // gd node instance_id <-> rapier rb_handle_to_id()
    pub collider_ids: Dictionary,   // gd node instance_id <-> rapier collider_handle_to_id()

    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
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
            collider_ids: Dictionary::new(),
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

            let g_pos = crate::utils::pos_rapier_to_godot(*rb.translation());
            let g_rot = crate::utils::rot_rapier_to_godot(*rb.rotation());

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

    pub fn register_rigid_body(&mut self, class: &mut RapierRigidBody3D) -> RigidBodyHandle {
        let instance_id = class.base().instance_id().to_i64();
        let mut rigid_body: RigidBody = class.build();
        rigid_body.user_data = u128::try_from(instance_id).unwrap();
        let handle = self.rigid_body_set.insert(rigid_body);
        let id = crate::utils::rb_handle_to_id(handle);
        self.rigid_body_ids.set(instance_id, id);
        handle
    }

    pub fn unregister_rigid_body(&mut self, class: &mut RapierRigidBody3D) {
        let instance_id = class.base().instance_id().to_i64();
        let handle = class.handle;
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

    pub fn get_rigid_body_mut(&mut self, handle: RigidBodyHandle) -> Option<&mut RigidBody> {
        self.rigid_body_set.get_mut(handle)
    }

    pub fn register_collider(&mut self, class: &mut RapierCollider3D) -> ColliderHandle {
        let instance_id = class.base().instance_id().to_i64();
        let mut collider: Collider = class.build();
        collider.user_data = u128::try_from(instance_id).unwrap();
        let handle = self.collider_set.insert(collider);
        let id = crate::utils::collider_handle_to_id(handle);
        self.collider_ids.set(instance_id, id);
        handle
    }

    // Parent rigid_body must already exist in rigid_body_set when calling this
    pub fn register_collider_with_parent(
        &mut self,
        class: &mut RapierCollider3D,
        parent_handle: RigidBodyHandle,
    ) -> ColliderHandle {
        let instance_id = class.base().instance_id().to_i64();
        let mut collider: Collider = class.build();
        collider.user_data = u128::try_from(instance_id).unwrap();
        let handle =
            self.collider_set
                .insert_with_parent(collider, parent_handle, &mut self.rigid_body_set);
        let id = crate::utils::collider_handle_to_id(handle);
        self.collider_ids.set(instance_id, id);
        handle
    }

    pub fn set_collider_parent(
        &mut self,
        collider: ColliderHandle,
        parent: Option<RigidBodyHandle>,
    ) {
        self.collider_set
            .set_parent(collider, parent, &mut self.rigid_body_set);
    }

    pub fn unregister_collider(&mut self, class: &mut RapierCollider3D) {
        let instance_id = class.base().instance_id().to_i64();
        let handle = class.handle;
        self.collider_set.remove(
            handle,
            &mut self.island_manager,
            &mut self.rigid_body_set,
            false,
        ); // false = don't wakeup parent rigid_body
        self.collider_ids.remove(instance_id);
    }

    pub fn get_collider_mut(&mut self, handle: ColliderHandle) -> Option<&mut Collider> {
        self.collider_set.get_mut(handle)
    }
}
