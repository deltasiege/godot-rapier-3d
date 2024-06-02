use godot::engine::SphereShape3D;
use godot::prelude::*;
use rapier3d::geometry::SharedShape;

use super::HasSharedShapeField;

#[derive(GodotClass, Debug)]
#[class(init, base = SphereShape3D)]
pub struct RapierSphereShape3D {
    base: Base<SphereShape3D>,
}

impl HasSharedShapeField for RapierSphereShape3D {
    fn get_shared_shape(&self) -> SharedShape {
        SharedShape::ball(self.base().get_radius())
    }
}
