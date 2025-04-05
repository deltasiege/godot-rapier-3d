mod editor_plugin;
mod singleton;

// Interface module is responsible for all communication between Godot and Rapier

pub use singleton::{register as register_singleton, unregister as unregister_singleton, GR3D};
