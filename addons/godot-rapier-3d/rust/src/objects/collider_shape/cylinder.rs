use godot::engine::CylinderShape3D;
use godot::prelude::*;
use rapier3d::geometry::SharedShape;

use super::HasSharedShapeField;

#[derive(GodotClass, Debug)]
#[class(init, base = CylinderShape3D)]
pub struct RapierCylinderShape3D {
    base: Base<CylinderShape3D>,
}

impl HasSharedShapeField for RapierCylinderShape3D {
    fn get_shared_shape(&self) -> SharedShape {
        SharedShape::cylinder(self.base().get_height() * 0.5, self.base().get_radius())
    }
}
