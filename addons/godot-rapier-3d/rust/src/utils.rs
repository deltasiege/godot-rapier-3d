use godot::builtin::{Basis, Quaternion as GQuaternion, Transform3D, Vector3 as GVector};
use godot::engine::{Node, Node3D};
use godot::obj::{Gd, Inherits, WithBaseField};
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

// Returns the immediate children of a node that are of a specific type
pub fn get_shallow_children_of_type<T: WithBaseField<Base = Node3D> + Inherits<Node>>(
    node: &Node3D,
) -> Vec<Gd<T>> {
    let mut children = Vec::new();
    for i in 0..node.get_child_count() {
        let child = match node.get_child(i) {
            Some(child) => child,
            None => continue,
        };
        match child.try_cast::<T>() {
            Ok(c) => children.push(c),
            Err(_) => continue,
        }
    }
    children
}
