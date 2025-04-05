use godot::classes::{Engine, IObject, Object};
use godot::prelude::*;

/*
    GR3D singleton exposes all public functions that user can call from GDScript.
*/

/// Use the GR3D singleton to interact with the Rapier physics engine
#[derive(GodotClass)]
#[class(base = Object)]
pub struct GR3D {
    base: Base<Object>,
}

#[godot_api]
impl IObject for GR3D {
    fn init(base: Base<Object>) -> Self {
        Self { base }
    }
}

#[godot_api]
impl GR3D {}

pub const NAME: &str = "GR3D";

pub fn register() {
    Engine::singleton().register_singleton(NAME, &GR3D::new_alloc());
}

pub fn unregister() {
    let mut engine = Engine::singleton();
    if let Some(my_singleton) = engine.get_singleton(NAME) {
        engine.unregister_singleton(NAME);
        my_singleton.free();
    } else {
        log::error!("Failed to get {} singleton", NAME);
    }
}
