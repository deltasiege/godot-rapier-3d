use godot::classes::notify::Node3DNotification;
use godot::classes::{INode3D, Node3D};
use godot::prelude::*;

use crate::nodes::common::*;

#[derive(GodotClass)]
#[class(tool, base=Node3D)]
pub struct RollbackStaticBody3D {
    pub blueprint: Option<NodeBlueprint>,
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for RollbackStaticBody3D {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            blueprint: None,
            base,
        }
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
