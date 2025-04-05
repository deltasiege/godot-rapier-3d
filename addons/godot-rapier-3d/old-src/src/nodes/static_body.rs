use godot::classes::notify::Node3DNotification;
use godot::classes::{INode3D, Node3D};
use godot::prelude::*;

use crate::nodes::{generate_cuid, IRapierObject};

#[derive(GodotClass)]
#[class(tool, base=Node3D)]
pub struct RapierStaticBody3D {
    #[var(usage_flags = [EDITOR, STORAGE, READ_ONLY])]
    pub cuid: GString,
    #[var(usage_flags = [EDITOR, STORAGE, READ_ONLY])]
    pub handle: Array<u32>,
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for RapierStaticBody3D {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            cuid: generate_cuid(),
            handle: Array::new(),
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
impl RapierStaticBody3D {
    #[func]
    fn set_uid(&mut self, cuid: GString) {
        self.set_cuid(cuid);
    }

    #[func]
    fn match_rapier(&mut self) {
        self.sync()
    }
}
