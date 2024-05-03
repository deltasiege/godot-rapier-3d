use godot::prelude::*;

mod collider;
mod physics_pipeline;
mod rigid_body;
mod utils;

struct GodotRapier3D;

#[gdextension]
unsafe impl ExtensionLibrary for GodotRapier3D {}
