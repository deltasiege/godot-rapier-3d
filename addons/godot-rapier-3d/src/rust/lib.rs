use crate::singleton::Rapier3DSingleton;
use godot::engine::Engine;
use godot::prelude::*;

// mod collider;
// mod editor_plugin;

// mod physics_pipeline;
mod singleton;
// mod rigid_body;
// mod utils;

struct GodotRapier3D;

#[gdextension]
unsafe impl ExtensionLibrary for GodotRapier3D {
    fn on_level_init(level: InitLevel) {
        if level == InitLevel::Scene {
            Engine::singleton().register_singleton(
                StringName::from("Rapier3D"),
                Rapier3DSingleton::new_alloc().upcast(),
            );
        }
    }

    fn on_level_deinit(level: InitLevel) {
        if level == InitLevel::Scene {
            let mut engine = Engine::singleton();
            let singleton_name = StringName::from("Rapier3D");

            let singleton = engine
                .get_singleton(singleton_name.clone())
                .expect("cannot retrieve the singleton");

            engine.unregister_singleton(singleton_name);
            singleton.free();
        }
    }
}
