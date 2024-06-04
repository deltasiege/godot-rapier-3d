use godot::builtin::{Basis, Quaternion as GQuaternion, Transform3D, Vector3 as GVector};
use nalgebra::geometry::Quaternion as NAQuaternion;
use rapier3d::math::{Isometry, Real, Rotation, Translation, Vector as RVector};
use rapier3d::prelude::*;

mod id;
mod logger;

pub use id::*;
pub use logger::*;

// Godot transform to Rapier isometry
pub fn transform_to_isometry(transform: Transform3D) -> Isometry<Real> {
    let pos = transform.origin;
    let quat = transform.basis.to_quat();
    let translation = Translation::new(pos.x, pos.y, pos.z);
    let rotation = Rotation::from_quaternion(NAQuaternion::new(
        -1.0 * quat.z,
        quat.y,
        -1.0 * quat.x,
        quat.w,
    ));
    Isometry::from_parts(translation, rotation)
}

// Rapier isometry to Godot transform
pub fn isometry_to_transform(isometry: Isometry<Real>) -> Transform3D {
    let vec = isometry.translation.vector;
    let quat = isometry.rotation.quaternion();
    Transform3D {
        basis: Basis::from_quat(GQuaternion::new(
            quat.coords.x,
            quat.coords.y,
            quat.coords.z,
            quat.coords.w,
        )),
        origin: GVector::new(vec.x, vec.y, vec.z),
    }
}

// Converts Godot Vector3 to Rapier Vector
pub fn vec_g2r(vec: GVector) -> RVector<Real> {
    RVector::new(vec.x, vec.y, vec.z)
}
