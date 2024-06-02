use crate::engine::{register_engine, unregister_engine};
use godot::prelude::*;

mod editor_plugin;
mod engine;
mod lookups;
mod objects;
mod pipeline;
mod queue;
mod utils;

pub use engine::get_engine;
pub use lookups::{IDBridge, LookupIdentifier, Lookups};
pub use objects::{ObjectKind, PhysicsObject};
pub use pipeline::{GR3DPhysicsPipeline, GR3DPhysicsState};
pub use queue::ActionQueue;
pub use utils::{cuid2, handle_error};

struct GodotRapier3D;

#[gdextension]
unsafe impl ExtensionLibrary for GodotRapier3D {
    fn on_level_init(level: InitLevel) {
        if level == InitLevel::Scene {
            register_engine();
        }
    }

    fn on_level_deinit(level: InitLevel) {
        if level == InitLevel::Scene {
            unregister_engine();
        }
    }
}
