use godot::builtin::{Basis, Quaternion as GQuaternion, Transform3D, Vector3 as GVector3};
use godot::prelude::*;
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

pub fn gstr_to_millis(gstr: GString) -> u128 {
    gstr.to_string().parse::<u128>().unwrap_or_else(|_| {
        log::error!("Failed to parse GString to u128: {}", gstr);
        0
    })
}

pub fn to_string_variant(value: impl std::fmt::Debug) -> Variant {
    format!("{:?}", value).to_variant()
}

pub fn option_to_variant(value: Option<impl ToGodot>) -> Variant {
    match value {
        Some(v) => v.to_variant(),
        None => Variant::nil(),
    }
}

pub fn stringify_option(value: Option<impl std::fmt::Debug>) -> String {
    match value {
        Some(v) => format!("{:?}", v),
        None => "None".to_string(),
    }
}

pub fn try_parse<T: std::str::FromStr>(value: &str) -> Option<T> {
    match value.parse::<T>() {
        Ok(parsed) => Some(parsed),
        Err(_) => {
            log::error!(
                "Failed to parse value: {} as {}",
                value,
                std::any::type_name::<T>()
            );
            None
        }
    }
}

pub fn variant_to<T>(variant: &Variant) -> Option<T>
where
    T: FromGodot,
{
    match variant.try_to::<T>() {
        Ok(value) => Some(value),
        Err(err) => {
            log::error!(
                "Failed to convert Variant to {}: {}",
                std::any::type_name::<T>(),
                err
            );
            None
        }
    }
}
