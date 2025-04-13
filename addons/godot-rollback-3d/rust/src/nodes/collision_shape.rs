use godot::classes::notify::Node3DNotification;
use godot::classes::{CollisionShape3D, INode3D, Node3D, Shape3D};
use godot::prelude::*;

use crate::nodes::common::*;

#[derive(GodotClass)]
#[class(tool, base=Node3D)]
pub struct RollbackCollisionShape3D {
    pub node_data: Option<NodeData>,
    pub blueprint: Option<NodeBlueprint>,
    #[export]
    pub col_shape: Option<Gd<CollisionShape3D>>,
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for RollbackCollisionShape3D {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            node_data: None,
            blueprint: None,
            col_shape: None,
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

#[godot_api]
impl RollbackCollisionShape3D {
    #[func]
    pub fn get_shape(&self) -> Option<Gd<Shape3D>> {
        self.col_shape.as_ref()?.get_shape()
    }
}
