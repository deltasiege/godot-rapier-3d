use godot::{classes::Engine, obj::WithBaseField, prelude::*};

use crate::interface::GR3D;

/// Get a reference to the 'GR3D' singleton
pub fn get_gr3d() -> Option<Gd<GR3D>> {
    Some(get_singleton("GR3D")?.cast::<GR3D>())
}

/// Get a reference to the 'GR3DRuntime' autoload singleton
pub fn get_runtime(
    node: &(impl WithBaseField + GodotClass<Base = impl Inherits<Node>>),
) -> Option<Gd<Node>> {
    node.base()
        .upcast_ref()
        .get_node_or_null("/root/GR3DRuntime")
}

/// Get a reference to the given singleton by name.
fn get_singleton(name: &str) -> Option<Gd<Object>> {
    let singleton = Engine::singleton().get_singleton(name);
    if singleton.is_none() {
        log::error!("Failed to get '{}' singleton", name);
    }
    singleton
}

/// Register the given singleton by name.
fn register_singleton(name: &str, instance: &Gd<impl Inherits<Object>>) {
    Engine::singleton().register_singleton(name, instance);
}

/// Unregister the given singleton by name.
fn unregister_singleton(name: &str) {
    if let Some(singleton) = get_singleton(name) {
        singleton.free();
    }
}

/// Register all singletons used by this extension
pub fn register_singletons() {
    register_singleton("GR3D", &GR3D::new_alloc());
}

/// Unregister all singletons used by this extension
pub fn unregister_singletons() {
    unregister_singleton("GR3D");
}
