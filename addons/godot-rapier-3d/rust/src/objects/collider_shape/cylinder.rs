use godot::classes::CylinderShape3D;
use godot::prelude::*;
use rapier3d::geometry::SharedShape;

use super::HasSharedShapeField;

#[derive(GodotClass, Debug)]
#[class(init, base = CylinderShape3D)]
pub struct RapierCylinderShape3D {
    base: Base<CylinderShape3D>,
}

/*

  Use round cyclinder because it's more performant than non-round cyclinder.
  https://rapier.rs/docs/user_guides/rust/colliders/#round-shapes

  > For algorithmic reasons, collision-detection involving round cylinders,
  | round cones, round convex polygon or round convex polyhedron will be faster
  | than collision-detection with their non-round counterparts.

*/

impl HasSharedShapeField for RapierCylinderShape3D {
    fn get_shared_shape(&self) -> SharedShape {
        SharedShape::round_cylinder(
            self.base().get_height() * 0.5,
            self.base().get_radius(),
            self.base().get_margin(),
        )
    }
}
