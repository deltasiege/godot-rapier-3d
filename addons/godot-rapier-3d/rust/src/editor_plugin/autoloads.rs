use crate::editor_plugin::GR3DEditorPlugin;
use godot::prelude::*;

pub const AUTOLOAD_NAMES: &'static [&'static str] = &["Rapier3D", "Rapier3DDebugger"];
pub const AUTOLOAD_PATHS: &'static [&'static str] = &[
    "res://addons/godot-rapier-3d/Rapier3D.gd",
    "res://addons/godot-rapier-3d/Rapier3DDebugger.gd",
];

pub fn add_all_autoloads(plugin: &mut GR3DEditorPlugin) {
    let singleton_list: PackedStringArray = godot::engine::Engine::singleton()
        .get_singleton_list()
        .clone();

    for idx in 0..AUTOLOAD_NAMES.len() {
        let name = AUTOLOAD_NAMES[idx];
        let path = AUTOLOAD_PATHS[idx];
        let exists = singleton_list.contains(&GString::from(name));
        if !exists {
            add_autoload(plugin, name, path);
        }
    }
}

pub fn remove_all_autoloads(plugin: &mut GR3DEditorPlugin) {
    for name in AUTOLOAD_NAMES {
        remove_autoload(plugin, name);
    }
}

fn add_autoload(plugin: &mut GR3DEditorPlugin, name: &str, path: &str) {
    // Call deferred so that Godot editor has time to detect Rust singleton first
    log::debug!("Adding autoload: {} -> {}", name, path);
    plugin.base_mut().call_deferred(
        StringName::from("add_autoload_singleton"),
        &[
            GString::from(name).to_variant(),
            GString::from(path).to_variant(),
        ],
    );
}

fn remove_autoload(plugin: &mut GR3DEditorPlugin, name: &str) {
    log::debug!("Removing autoload: {}", name);
    plugin
        .base_mut()
        .remove_autoload_singleton(GString::from(name));
}
