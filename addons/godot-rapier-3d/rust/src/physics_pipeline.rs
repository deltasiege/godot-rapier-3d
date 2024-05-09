use crate::collider::RapierCollider3D;
use crate::physics_state::GR3DPhysicsState;
use crate::rigid_body::RapierRigidBody3D;
use godot::prelude::*;
use rapier3d::prelude::*;

pub struct GR3DPhysicsPipeline {
    pub rigid_body_ids: Dictionary, // gd node instance_id <-> rapier rb_handle_to_id()
    pub collider_ids: Dictionary,   // gd node instance_id <-> rapier collider_handle_to_id()
    physics_pipeline: PhysicsPipeline,
    pub state: GR3DPhysicsState,
    physics_hooks: (),
    event_handler: (),
}

impl GR3DPhysicsPipeline {
    pub fn new() -> Self {
        Self {
            rigid_body_ids: Dictionary::new(),
            collider_ids: Dictionary::new(),
            physics_pipeline: PhysicsPipeline::new(),
            state: GR3DPhysicsState::default(),
            physics_hooks: (),
            event_handler: (),
        }
    }

    pub fn step(&mut self) {
        self.physics_pipeline.step(
            &self.state.gravity,
            &self.state.integration_parameters,
            &mut self.state.island_manager,
            &mut self.state.broad_phase,
            &mut self.state.narrow_phase,
            &mut self.state.rigid_body_set,
            &mut self.state.collider_set,
            &mut self.state.impulse_joint_set,
            &mut self.state.multibody_joint_set,
            &mut self.state.ccd_solver,
            Some(&mut self.state.query_pipeline),
            &self.physics_hooks,
            &self.event_handler,
        );

        self.sync_active_body_positions();
    }

    // Syncs Godot position to Rapier position
    pub fn sync_active_body_positions(&mut self) {
        let active_dynamic_bodies = self.state.island_manager.active_dynamic_bodies();

        for active_body_handle in active_dynamic_bodies {
            let rb = match self.state.rigid_body_set.get(*active_body_handle) {
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
        let handle = self.state.rigid_body_set.insert(rigid_body);
        let id = crate::utils::rb_handle_to_id(handle);
        self.rigid_body_ids.set(instance_id, id);
        handle
    }

    pub fn unregister_rigid_body(&mut self, class: &mut RapierRigidBody3D) {
        let instance_id = class.base().instance_id().to_i64();
        let handle = class.handle;
        self.state.rigid_body_set.remove(
            handle,
            &mut self.state.island_manager,
            &mut self.state.collider_set,
            &mut self.state.impulse_joint_set,
            &mut self.state.multibody_joint_set,
            true, // true = also remove colliders
        );
        self.rigid_body_ids.remove(instance_id);
    }

    pub fn get_rigid_body_mut(&mut self, handle: RigidBodyHandle) -> Option<&mut RigidBody> {
        self.state.rigid_body_set.get_mut(handle)
    }

    pub fn register_collider(&mut self, class: &mut RapierCollider3D) -> ColliderHandle {
        if self.collider_is_registered(class) {
            return class.handle;
        }
        let instance_id = class.base().instance_id().to_i64();
        let mut collider: Collider = class.build();
        collider.user_data = u128::try_from(instance_id).unwrap();
        let handle = self.state.collider_set.insert(collider);
        let id = crate::utils::collider_handle_to_id(handle);
        self.collider_ids.set(instance_id, id);
        handle
    }

    pub fn collider_is_registered(&self, class: &RapierCollider3D) -> bool {
        let instance_id = class.base().instance_id().to_i64();
        let result = self.collider_ids.contains_key(instance_id);
        if result {
            godot_print!("RapierCollider3D '{:?}' already registered", class.handle);
        }
        result
    }

    // Parent rigid_body must already exist in rigid_body_set when calling this
    pub fn register_collider_with_parent(
        &mut self,
        class: &mut RapierCollider3D,
        parent_handle: RigidBodyHandle,
    ) -> ColliderHandle {
        if self.collider_is_registered(class) {
            return class.handle;
        }
        let instance_id = class.base().instance_id().to_i64();
        let mut collider: Collider = class.build();
        collider.user_data = u128::try_from(instance_id).unwrap();
        let handle = self.state.collider_set.insert_with_parent(
            collider,
            parent_handle,
            &mut self.state.rigid_body_set,
        );
        let id = crate::utils::collider_handle_to_id(handle);
        self.collider_ids.set(instance_id, id);
        handle
    }

    pub fn set_collider_parent(
        &mut self,
        collider: ColliderHandle,
        parent: Option<RigidBodyHandle>,
    ) {
        self.state
            .collider_set
            .set_parent(collider, parent, &mut self.state.rigid_body_set);
    }

    pub fn unregister_collider(&mut self, class: &mut RapierCollider3D) {
        let instance_id = class.base().instance_id().to_i64();
        let handle = class.handle;
        self.state.collider_set.remove(
            handle,
            &mut self.state.island_manager,
            &mut self.state.rigid_body_set,
            false,
        ); // false = don't wakeup parent rigid_body
        self.collider_ids.remove(instance_id);
    }

    pub fn get_collider_mut(&mut self, handle: ColliderHandle) -> Option<&mut Collider> {
        self.state.collider_set.get_mut(handle)
    }
}
