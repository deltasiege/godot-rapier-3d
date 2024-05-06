use crate::physics_pipeline::RapierPhysicsPipeline;
use crate::singleton::Rapier3DSingleton;
use godot::engine::notify::Node3DNotification;
use godot::engine::INode3D;
use godot::engine::Node3D;
use godot::prelude::*;
use rapier3d::math::Rotation;
use rapier3d::math::Vector as RVector;
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
            additional_mass: 1.0,
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
        let ston = crate::utils::get_singleton();
        if ston.is_some() {
            let handle = ston.unwrap().bind_mut().pipeline.register_rigid_body(self);
            self.handle = handle;
            self.id = crate::utils::rb_handle_to_id(handle);
        }
    }

    fn on_exit_tree(&mut self) {
        godot_print!("RapierRigidBody3D::exit_tree()");
        let ston = crate::utils::get_singleton();
        if ston.is_some() {
            ston.unwrap()
                .bind_mut()
                .pipeline
                .unregister_rigid_body(self);
        }
    }

    fn on_transform_changed(&mut self) {
        // TODO lock this when pipeline is stepped, instead of only doing in editor
        godot_print!("RapierRigidBody3D::on_transform_changed()");
        // let transform = self.base().get_global_transform();
        // let (pos, rot) = transform_to_posrot(transform);
        // self.set_rapier_translation(pos);
        // self.set_rapier_rotation(rot);
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
