use godot::classes::notify::Node3DNotification;
use godot::classes::{INode3D, Node3D};
use godot::prelude::*;

use crate::nodes::common::*;
use crate::utils::vector_to_godot;

#[derive(GodotClass)]
#[class(tool, base=Node3D)]
pub struct RollbackRigidBody3D {
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for RollbackRigidBody3D {
    fn init(base: Base<Node3D>) -> Self {
        Self { base }
    }

    fn enter_tree(&mut self) {
        self.on_enter_tree();
    }

    fn exit_tree(&mut self) {
        self.on_exit_tree();
    }

    fn on_notification(&mut self, what: Node3DNotification) {
        match what {
            Node3DNotification::PHYSICS_PROCESS => self.sync(),
            _ => {}
        }
    }
}

#[godot_api]
impl RollbackRigidBody3D {
    #[func]
    fn get_gruid(&self) -> Variant {
        self.on_get_gruid()
    }

    #[func]
    fn get_real_velocity(&self) -> Vector3 {
        vector_to_godot(self.get_body_state().linvel)
    }
}
