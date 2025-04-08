mod debugger;
mod editor_plugin;
mod logger_singleton;
mod network_singleton;
mod singleton;

// Interface module is responsible for all communication between Godot and Rapier

pub use logger_singleton::{register as register_logger, unregister as unregister_logger};
pub use network_singleton::{
    get_net_singleton, register as register_net, unregister as unregister_net, GR3DNet,
};
pub use singleton::{
    get_singleton, get_tree, register as register_singleton, unregister as unregister_singleton,
};

pub use editor_plugin::get_runtime;
