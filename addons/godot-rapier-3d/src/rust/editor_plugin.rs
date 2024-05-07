use crate::gizmos::{add_all_gizmos, remove_all_gizmos};
use godot::engine::EditorPlugin;
use godot::engine::IEditorPlugin;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(tool, init, editor_plugin, base=EditorPlugin)]
pub struct GodotRapier3DEditorPlugin {
    base: Base<EditorPlugin>,
}

#[godot_api]
impl IEditorPlugin for GodotRapier3DEditorPlugin {
    fn enter_tree(&mut self) {
        self.base_mut().add_autoload_singleton(
            crate::utils::get_autoload_name(),
            crate::utils::get_autoload_path(),
        );

        add_all_gizmos(self);
    }

    fn exit_tree(&mut self) {
        self.base_mut()
            .remove_autoload_singleton(crate::utils::get_autoload_name());

        remove_all_gizmos(self);
    }
}
