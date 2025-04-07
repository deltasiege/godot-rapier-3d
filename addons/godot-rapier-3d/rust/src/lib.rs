use godot::prelude::*;

mod adapters;
pub mod config;
mod interface;
mod nodes;
pub mod types;
mod utils;
mod world;

use interface::{register_singletons, unregister_singletons};
pub use world::World;

struct GodotRapier3D;

#[gdextension]
unsafe impl ExtensionLibrary for GodotRapier3D {
    fn on_level_init(level: InitLevel) {
        if level == InitLevel::Scene {
            register_singletons();
        }
    }

    fn on_level_deinit(level: InitLevel) {
        if level == InitLevel::Scene {
            unregister_singletons();
        }
    }
}
