use godot::engine::notify::Node3DNotification;
use godot::engine::INode3D;
use godot::engine::Node3D;
use godot::prelude::*;
use rapier3d::prelude::*;

#[derive(GodotClass)]
#[class(base=Node3D)]
pub struct RapierRigidBody3D {
    #[var]
    pub id: Array<Variant>, // RigidBodyHandle::into_raw_parts
    pub handle: RigidBodyHandle,
    #[export]
    body_type: RigidBodyType,
    #[export]
    pub additional_mass: Real,
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for RapierRigidBody3D {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            id: Array::new(),
            handle: RigidBodyHandle::invalid(),
            body_type: RigidBodyType::Dynamic,
            additional_mass: 0.0,
            base,
        }
    }

    fn on_notification(&mut self, what: Node3DNotification) {
        match what {
            Node3DNotification::EnterTree => self.on_enter_tree(),
            Node3DNotification::ExitTree => self.on_exit_tree(),
            Node3DNotification::TransformChanged => self.on_transform_changed(),
            _ => {}
        };
    }
}

#[godot_api]
impl RapierRigidBody3D {
    fn on_enter_tree(&mut self) {
        self.base_mut().set_notify_transform(true);
        let ston = crate::utils::get_engine_singleton();
        if ston.is_some() {
            let mut singleton = ston.unwrap();
            let pipeline = &mut singleton.bind_mut().pipeline;
            let handle = pipeline.register_rigid_body(self);
            self.handle = handle;
            self.id = crate::utils::rb_handle_to_id(handle);

            let rigid_body = pipeline.get_rigid_body_mut(self.handle);

            match rigid_body {
                Some(rigid_body) => {
                    self.sync_transforms_to_godot(rigid_body, false);
                }
                None => {
                    godot_error!("RapierRigidBody3D::on_enter_tree - Could not find rigid body");
                    return;
                }
            }
        }
    }

    fn on_exit_tree(&mut self) {
        let ston = crate::utils::get_engine_singleton();
        if ston.is_some() {
            ston.unwrap()
                .bind_mut()
                .pipeline
                .unregister_rigid_body(self);
        }
    }

    fn on_transform_changed(&mut self) {
        let ston = crate::utils::get_engine_singleton();
        if ston.is_some() {
            let mut singleton = ston.unwrap();
            let pipeline = &mut singleton.bind_mut().pipeline;
            let rigid_body = pipeline.get_rigid_body_mut(self.handle);

            match rigid_body {
                Some(rigid_body) => {
                    self.sync_transforms_to_godot(rigid_body, false);
                }
                None => {
                    godot_error!(
                        "RapierRigidBody3D::on_transform_changed - Could not find rigid body"
                    );
                    return;
                }
            }
        }
    }

    // Changes rapier transforms to match godot transforms
    fn sync_transforms_to_godot(&mut self, rigid_body: &mut RigidBody, wakeup: bool) {
        if self.base().is_inside_tree() {
            let translation = self.base().get_global_position();
            let rotation = self.base().get_quaternion();
            let r_pos = crate::utils::pos_godot_to_rapier(translation);
            let r_rot = crate::utils::rot_godot_to_rapier(rotation);
            rigid_body.set_translation(r_pos, wakeup);
            rigid_body.set_rotation(r_rot, wakeup);
        }
    }

    pub fn build(&self) -> RigidBody {
        let rb = match self.body_type {
            RigidBodyType::Dynamic => RigidBodyBuilder::dynamic(),
            RigidBodyType::Fixed => RigidBodyBuilder::fixed(),
            RigidBodyType::KinematicPositionBased => RigidBodyBuilder::kinematic_position_based(),
            RigidBodyType::KinematicVelocityBased => RigidBodyBuilder::kinematic_velocity_based(),
        };
        rb.additional_mass(self.additional_mass).build()
    }

    #[func]
    pub fn print_colliders(&self) {
        let ston = crate::utils::get_engine_singleton();
        if ston.is_some() {
            let mut singleton = ston.unwrap();
            let pipeline = &mut singleton.bind_mut().pipeline;
            let rigid_body = pipeline.get_rigid_body_mut(self.handle);
            match rigid_body {
                Some(rigid_body) => {
                    godot_print!("Colliders: {:?}", rigid_body.colliders());
                }
                None => {
                    godot_error!("Could not find rigid body {:?}", self.handle);
                    return;
                }
            }
        }
    }
}

#[derive(GodotConvert, Var, Export)]
#[godot(via = GString)]
// https://docs.rs/rapier3d/latest/rapier3d/dynamics/enum.RigidBodyType.html
pub enum RigidBodyType {
    Dynamic,
    Fixed,
    KinematicPositionBased,
    KinematicVelocityBased,
}
