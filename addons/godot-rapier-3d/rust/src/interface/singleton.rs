use godot::classes::{IObject, Object};
use godot::prelude::*;

use crate::World;

/// Public API interface for Godot Rollback 3D.
#[derive(GodotClass)]
#[class(base = Object)]
pub struct GR3D {
    pub world: World,
    base: Base<Object>,
}

#[godot_api]
impl IObject for GR3D {
    fn init(base: Base<Object>) -> Self {
        Self {
            world: World::new(),
            base,
        }
    }
}
