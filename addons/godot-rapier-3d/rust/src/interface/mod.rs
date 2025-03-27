mod debugger;
mod editor_plugin;
mod singleton;
mod sync;

// Interface module is responsible for all communication between Godot and Rapier

pub use singleton::{
    get_singleton, get_tree, register as register_singleton, unregister as unregister_singleton,
};
pub use sync::{
    get_sync_singleton, register as register_sync, unregister as unregister_sync, GR3DSync,
};

pub use editor_plugin::get_runtime;
