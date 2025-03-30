use godot::prelude::*;

mod actions;
pub mod config;
mod interface;
mod network;
mod nodes;
mod utils;
mod world;

pub use actions::Action;
pub use world::LookupTable;
pub use world::World;

struct GodotRapier3D;

#[gdextension]
unsafe impl ExtensionLibrary for GodotRapier3D {
    fn on_level_init(level: InitLevel) {
        if level == InitLevel::Scene {
            interface::register_logger();
            interface::register_singleton();
            interface::register_net();
        }
    }

    fn on_level_deinit(level: InitLevel) {
        if level == InitLevel::Scene {
            interface::unregister_logger();
            interface::unregister_singleton();
            interface::unregister_net();
        }
    }
}
