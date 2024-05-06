use crate::Rapier3DSingleton;
use godot::builtin::Array as GArray;
use godot::builtin::Transform3D;
use godot::prelude::*;
use nalgebra::Quaternion as NAQuaternion;
use nalgebra::Vector3 as NAVector3;
use rapier3d::math::Rotation;
use rapier3d::prelude::*;

pub const SINGLETON_NAME: &str = "Rapier3D";

pub fn get_singleton() -> Option<Gd<Rapier3DSingleton>> {
    let gd_pointer = godot::engine::Engine::singleton().get_singleton(get_singleton_name());

    match gd_pointer {
        Some(gd_pointer) => {
            let casted = gd_pointer.cast::<Rapier3DSingleton>();
            Some(casted)
        }
        None => {
            godot_error!("Could not find Rapier3D singleton2");
            None
        }
    }
}

pub fn get_singleton_name() -> StringName {
    StringName::from(SINGLETON_NAME)
}

pub fn transform_to_posrot(transform: Transform3D) -> (NAVector3<Real>, Rotation<Real>) {
    let translation = transform.origin;
    let rotation = transform.basis.to_quat();
    let na_pos = NAVector3::new(translation.x, translation.y, translation.z);
    let na_rot = Rotation::from_quaternion(NAQuaternion::new(
        rotation.x, rotation.y, rotation.z, rotation.w,
    ));
    (na_pos, na_rot)
}

pub fn rb_handle_to_id(handle: RigidBodyHandle) -> GArray<Variant> {
    let (index, generation) = handle.into_raw_parts();
    let mut id = GArray::new();
    id.push(Variant::from(index));
    id.push(Variant::from(generation));
    id
}
