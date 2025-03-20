use godot::classes::notify::Node3DNotification;
use godot::classes::{INode3D, Node3D};
use godot::prelude::*;
use rapier3d::control::PidController;

use super::common::Forceable;
use super::Identifiable;
use crate::nodes::IRapierObject;

#[derive(GodotClass)]
#[class(tool, base=Node3D)]
pub struct RapierRigidBody3D {
    #[var(usage_flags = [EDITOR, STORAGE, READ_ONLY])]
    pub handle: Array<u32>,
    pub controller: PidController,
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for RapierRigidBody3D {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            handle: Array::new(),
            controller: PidController::default(),
            base,
        }
    }

    fn enter_tree(&mut self) {
        self.on_enter_tree();
    }

    fn exit_tree(&mut self) {
        self.on_exit_tree();
    }

    fn physics_process(&mut self, _delta: f64) {
        self.sync();
    }

    fn on_notification(&mut self, what: Node3DNotification) {
        match what {
            Node3DNotification::PHYSICS_PROCESS => self.sync(),
            _ => {}
        }
    }
}

#[godot_api]
impl RapierRigidBody3D {
    #[func]
    fn set_uid(&mut self, cuid: GString) {
        self.set_cuid(cuid);
    }

    #[func]
    fn match_rapier(&mut self) {
        self.sync()
    }

    #[func]
    fn get_real_velocity(&self) -> Vector3 {
        self.get_body_state().linvel
    }
}
