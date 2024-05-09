use crate::singleton::{register_engine, unregister_engine};
use godot::prelude::*;

mod collider;
mod debug_render_pipeline;
mod editor_plugin;
mod physics_pipeline;
mod physics_state;
mod rigid_body;
mod singleton;
mod utils;

struct GodotRapier3D;

#[gdextension]
unsafe impl ExtensionLibrary for GodotRapier3D {
    fn on_level_init(level: InitLevel) {
        if level == InitLevel::Scene {
            godot_print!("InitLevel::Scene");
            register_engine();
        }
    }

    fn on_level_deinit(level: InitLevel) {
        if level == InitLevel::Scene {
            unregister_engine();
        }
    }
}
