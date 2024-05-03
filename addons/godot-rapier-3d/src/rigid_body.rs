use crate::utils::transform_to_posrot;
use godot::engine::notify::Node3DNotification;
use godot::engine::INode3D;
use godot::engine::Node3D;
use godot::prelude::*;
use nalgebra::Vector3 as NAVector3;
use rapier3d::math::Rotation;
use rapier3d::prelude::*;

#[derive(GodotClass)]
#[class(base=Node3D)]
pub struct RapierRigidBody3D {
    pub handle: RigidBodyHandle,
    pub rigid_body: RigidBody,
    #[export]
    body_type: RigidBodyType,
    #[export]
    pub mass: Real,
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for RapierRigidBody3D {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            handle: RigidBodyHandle::invalid(),
            rigid_body: RigidBodyBuilder::dynamic().build(),
            body_type: RigidBodyType::Dynamic,
            mass: 1.0,
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
    }

    fn on_exit_tree(&mut self) {
        godot_print!("RapierRigidBody3D::exit_tree()");
        // TODO remove self from physics pipeline rigid_body_set and remove colliders too
    }

    fn on_transform_changed(&mut self) {
        let transform = self.base().get_global_transform();
        let (pos, rot) = transform_to_posrot(transform);
        self.set_rapier_translation(pos);
        self.set_rapier_rotation(rot);
    }

    fn set_rapier_translation(&mut self, translation: NAVector3<Real>) {
        self.rigid_body.set_translation(translation, false); // TODO wakeup (second arg) is needed?
    }

    fn set_rapier_rotation(&mut self, rotation: Rotation<Real>) {
        self.rigid_body.set_rotation(rotation, false); // TODO wakeup (second arg) is needed?
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
