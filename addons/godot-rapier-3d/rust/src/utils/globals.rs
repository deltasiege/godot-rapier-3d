use godot::{classes::Engine, obj::WithBaseField, prelude::*};

use crate::interface::GR3D;

// Get a reference to the 'GR3D' singleton
pub fn get_singleton(name: &str) -> Option<Gd<GR3D>> {
    match Engine::singleton().get_singleton(name) {
        Some(singleton) => Some(singleton.cast::<GR3D>()),
        None => {
            log::error!("Failed to get '{}' singleton", name);
            None
        }
    }
}

// Get a reference to the 'GR3DRuntime' autoload singleton
pub fn get_runtime<T>(obj: &T) -> Option<Gd<Node>>
where
    T: WithBaseField + GodotClass<Base = Node>,
{
    obj.base().get_node_or_null("/root/GR3DRuntime")
}
