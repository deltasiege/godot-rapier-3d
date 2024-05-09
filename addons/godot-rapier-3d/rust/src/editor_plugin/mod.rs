use crate::editor_plugin::autoloads::{add_all_autoloads, remove_all_autoloads};
use crate::editor_plugin::gizmos::{add_all_gizmos, remove_all_gizmos};
use godot::engine::EditorPlugin;
use godot::engine::IEditorPlugin;
use godot::prelude::*;

mod autoloads;
mod gizmos;

#[derive(GodotClass)]
#[class(tool, init, editor_plugin, base=EditorPlugin)]
pub struct GR3DEditorPlugin {
    base: Base<EditorPlugin>,
}

#[godot_api]
impl IEditorPlugin for GR3DEditorPlugin {
    fn enter_tree(&mut self) {
        self.register();
    }

    fn exit_tree(&mut self) {
        self.unregister();
    }
}

impl GR3DEditorPlugin {
    pub fn register(&mut self) {
        add_all_autoloads(self);
        add_all_gizmos(self);
    }

    pub fn unregister(&mut self) {
        remove_all_autoloads(self);
        remove_all_gizmos(self);
    }
}
