use godot::classes::notify::Node3DNotification;
use godot::classes::{INode3D, Node3D};
use godot::prelude::*;

use crate::nodes::common::*;

#[derive(GodotClass)]
#[class(tool, init, base=Node3D)]
pub struct RollbackArea3D {
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for RollbackArea3D {
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
impl RollbackArea3D {
    #[func]
    fn get_gruid(&self) -> Variant {
        self.on_get_gruid()
    }
}
