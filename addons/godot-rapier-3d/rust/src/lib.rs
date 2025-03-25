use godot::prelude::*;

pub mod config;
mod interface;
mod nodes;
mod sync;
mod utils;
mod world;

pub use world::lookup::LookupTable;
pub use world::world::World;

struct GodotRapier3D;

#[gdextension]
unsafe impl ExtensionLibrary for GodotRapier3D {
    fn on_level_init(level: InitLevel) {
        if level == InitLevel::Scene {
            interface::register_singleton();
            interface::register_sync();
        }
    }

    fn on_level_deinit(level: InitLevel) {
        if level == InitLevel::Scene {
            interface::unregister_singleton();
            interface::unregister_sync();
        }
    }
}
