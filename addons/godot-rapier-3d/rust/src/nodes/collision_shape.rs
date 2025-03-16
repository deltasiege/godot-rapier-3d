use godot::classes::notify::Node3DNotification;
use godot::classes::{CollisionShape3D, INode3D, Node3D, Shape3D};
use godot::prelude::*;

use crate::nodes::IRapierObject;

#[derive(GodotClass)]
#[class(tool, base=Node3D)]
pub struct RapierCollisionShape3D {
    #[var(usage_flags = [EDITOR, STORAGE, READ_ONLY])]
    pub handle: Array<u32>,
    #[export]
    pub col_shape: Option<Gd<CollisionShape3D>>,
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for RapierCollisionShape3D {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            handle: Array::new(),
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
impl RapierCollisionShape3D {
    #[func]
    fn match_rapier(&mut self) {
        self.sync()
    }

    #[func]
    pub fn get_shape(&self) -> Option<Gd<Shape3D>> {
        match &self.col_shape {
            Some(col_shape) => match col_shape.get_shape() {
                Some(shape) => Some(shape),
                None => None,
            },
            None => None,
        }
    }
}
