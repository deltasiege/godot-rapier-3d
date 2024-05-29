use godot::engine::BoxShape3D;
use godot::prelude::*;
use rapier3d::geometry::SharedShape;

use super::HasSharedShapeField;

#[derive(GodotClass, Debug)]
#[class(init, base = BoxShape3D)]
pub struct RapierBoxShape3D {
    base: Base<BoxShape3D>,
}

impl HasSharedShapeField for RapierBoxShape3D {
    fn get_shared_shape(&self) -> SharedShape {
        let size = self.base().get_size();
        SharedShape::cuboid(size.x / 2.0, size.y / 2.0, size.z / 2.0)
    }
}
