mod debugger;
mod editor_plugin;
mod singleton;
mod world;

// Interface module is responsible for all communication between Godot and Rapier

pub use singleton::{
    get_singleton, get_tree, register as register_singleton, unregister as unregister_singleton,
};

pub use editor_plugin::get_runtime;
pub use world::{
    add_node_to_world, configure_node, move_node, remove_node_from_world, Action, Operation,
};
