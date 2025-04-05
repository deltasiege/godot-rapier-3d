use godot::classes::{EditorPlugin, IEditorPlugin};
use godot::prelude::*;

use crate::nodes::IRapierObject;

/*
    The editor plugin is only responsible for attaching the runtime autoload
*/

#[derive(GodotClass)]
#[class(tool, base = EditorPlugin)]
pub struct GR3DEditor {
    base: Base<EditorPlugin>,
}

#[godot_api]
impl IEditorPlugin for GR3DEditor {
    fn init(base: Base<EditorPlugin>) -> Self {
        Self { base }
    }

    fn enter_tree(&mut self) {
        self.base_mut().call_deferred(
            "add_autoload_singleton",
            &[
                Variant::from("GR3DRuntime"),
                Variant::from("res://addons/godot-rapier-3d/gd/GR3DRuntime.gd"),
            ],
        );
        log::debug!("Registered");
    }

    fn exit_tree(&mut self) {
        self.base_mut().remove_autoload_singleton("GR3DRuntime");
        log::debug!("Unregistered");
    }
}

// Runtime = internal functions / godot side orchestration
pub fn get_runtime(obj: &impl IRapierObject) -> Option<Gd<Node>> {
    obj.base().get_node_or_null("/root/GR3DRuntime")
}
