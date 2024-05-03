use godot::builtin::Transform3D;
use nalgebra::Quaternion as NAQuaternion;
use nalgebra::Vector3 as NAVector3;
use rapier3d::math::Rotation;
use rapier3d::prelude::*;

pub fn transform_to_posrot(transform: Transform3D) -> (NAVector3<Real>, Rotation<Real>) {
    let translation = transform.origin;
    let rotation = transform.basis.to_quat();
    let na_pos = NAVector3::new(translation.x, translation.y, translation.z);
    let na_rot = Rotation::from_quaternion(NAQuaternion::new(
        rotation.x, rotation.y, rotation.z, rotation.w,
    ));
    (na_pos, na_rot)
}
