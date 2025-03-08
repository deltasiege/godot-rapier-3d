use godot::classes::CapsuleShape3D;
use godot::prelude::*;
use rapier3d::geometry::SharedShape;

use super::HasSharedShapeField;

#[derive(GodotClass, Debug)]
#[class(init, base = CapsuleShape3D)]
pub struct RapierCapsuleShape3D {
    base: Base<CapsuleShape3D>,
}

impl HasSharedShapeField for RapierCapsuleShape3D {
    fn get_shared_shape(&self) -> SharedShape {
        SharedShape::capsule_y(self.base().get_height() / 4.0, self.base().get_radius())
    }
}
