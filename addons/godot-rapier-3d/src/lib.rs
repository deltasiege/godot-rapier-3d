use godot::prelude::*;

mod physics_pipeline;

struct GodotRapier3D;

#[gdextension]
unsafe impl ExtensionLibrary for GodotRapier3D {}
