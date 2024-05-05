use godot::engine::EditorPlugin;
use godot::engine::IEditorPlugin;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(tool, init, editor_plugin, base=EditorPlugin)]
struct GodotRapier3DEditorPlugin {
    base: Base<EditorPlugin>,
}

#[godot_api]
impl IEditorPlugin for GodotRapier3DEditorPlugin {
    fn enter_tree(&mut self) {
        self.base_mut().add_autoload_singleton(
            GString::from("Rapier3D"),
            GString::from("res://addons/godot-rapier-3d/Rapier3D.gd"),
        )
    }

    fn exit_tree(&mut self) {
        self.base_mut()
            .remove_autoload_singleton(GString::from("Rapier3D"))
    }
}
