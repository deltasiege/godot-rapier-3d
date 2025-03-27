mod debugger;
mod editor_plugin;
mod network_singleton;
mod singleton;

// Interface module is responsible for all communication between Godot and Rapier

pub use network_singleton::{
    get_net_singleton, register as register_sync, unregister as unregister_sync, GR3DNet,
};
pub use singleton::{
    get_singleton, get_tree, register as register_singleton, unregister as unregister_singleton,
};

pub use editor_plugin::get_runtime;
