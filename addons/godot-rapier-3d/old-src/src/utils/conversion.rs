use godot::builtin::{Basis, Quaternion as GQuaternion, Transform3D, Vector3 as GVector3};
use rapier3d::{
    math::{Isometry, Point, Real, Rotation, Translation},
    na::{Quaternion as RQuaternion, Vector3 as RVector3},
};

// Godot transform to Rapier isometry
pub fn transform_to_isometry(transform: Transform3D) -> Isometry<Real> {
    let pos = transform.origin;
    let quaternion = transform.basis.get_quaternion();
    Isometry::from_parts(
        Translation::new(pos.x, pos.y, pos.z),
        Rotation::from_quaternion(RQuaternion::new(
            quaternion.w,
            quaternion.x,
            quaternion.y,
            quaternion.z,
        )),
    )
}

pub fn isometry_to_transform(isometry: &Isometry<Real>) -> Transform3D {
    let translation = isometry.translation.vector;
    let rotation = isometry.rotation.quaternion();
    Transform3D {
        basis: Basis::from_quaternion(GQuaternion::new(
            rotation.i, rotation.j, rotation.k, rotation.w,
        )),
        origin: GVector3::new(translation.x, translation.y, translation.z),
    }
}

pub fn vector_to_rapier(vec: GVector3) -> RVector3<Real> {
    RVector3::<Real>::new(vec.x, vec.y, vec.z)
}

pub fn vector_to_godot(vec: RVector3<Real>) -> GVector3 {
    GVector3::new(vec.x, vec.y, vec.z)
}

pub fn vector_to_point(vec: &RVector3<Real>) -> Point<Real> {
    Point::from(vec.clone())
}

pub fn uniform_rapier_vector(value: Real) -> RVector3<Real> {
    RVector3::<Real>::new(value, value, value)
}
