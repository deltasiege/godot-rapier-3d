use crate::editor_plugin::GodotRapier3DEditorPlugin;
use godot::prelude::*;

pub const AUTOLOAD_NAMES: &'static [&'static str] = &["Rapier3D", "Rapier3DDebugger"];
pub const AUTOLOAD_PATHS: &'static [&'static str] = &[
    "res://addons/godot-rapier-3d/Rapier3D.gd",
    "res://addons/godot-rapier-3d/Rapier3DDebugger.gd",
];

pub fn add_all_autoloads(plugin: &mut GodotRapier3DEditorPlugin) {
    for idx in 0..AUTOLOAD_NAMES.len() {
        let name = AUTOLOAD_NAMES[idx];
        let path = AUTOLOAD_PATHS[idx];
        add_autoload(plugin, name, path);
    }
}

pub fn remove_all_autoloads(plugin: &mut GodotRapier3DEditorPlugin) {
    for name in AUTOLOAD_NAMES {
        remove_autoload(plugin, name);
    }
}

fn add_autoload(plugin: &mut GodotRapier3DEditorPlugin, name: &str, path: &str) {
    plugin
        .base_mut()
        .add_autoload_singleton(GString::from(name), GString::from(path));
}

fn remove_autoload(plugin: &mut GodotRapier3DEditorPlugin, name: &str) {
    plugin
        .base_mut()
        .remove_autoload_singleton(GString::from(name));
}
