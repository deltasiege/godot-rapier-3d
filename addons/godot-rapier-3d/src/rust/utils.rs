use crate::Rapier3DSingleton;
use godot::builtin::Array as GArray;
use godot::builtin::Quaternion as GQuaternion;
use godot::builtin::Vector3 as GVector;
use godot::prelude::*;
use nalgebra::Quaternion as NAQuaternion;
use nalgebra::Vector3 as NAVector;
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

pub fn rot_godot_to_rapier(rot: GQuaternion) -> Rotation<Real> {
    Rotation::from_quaternion(NAQuaternion::new(rot.x, rot.y, rot.z, rot.w))
}

pub fn rot_rapier_to_godot(rot: Rotation<Real>) -> GQuaternion {
    let coords = rot.quaternion().coords;
    GQuaternion::new(coords.x, coords.y, coords.z, coords.w)
}

pub fn pos_godot_to_rapier(pos: GVector) -> NAVector<Real> {
    NAVector::new(pos.x, pos.y, pos.z)
}

pub fn pos_rapier_to_godot(pos: NAVector<Real>) -> GVector {
    GVector::new(pos.x, pos.y, pos.z)
}

pub fn rb_handle_to_id(handle: RigidBodyHandle) -> GArray<Variant> {
    let (index, generation) = handle.into_raw_parts();
    let mut id = GArray::new();
    id.push(Variant::from(index));
    id.push(Variant::from(generation));
    id
}

pub fn collider_handle_to_id(handle: ColliderHandle) -> GArray<Variant> {
    let (index, generation) = handle.into_raw_parts();
    let mut id = GArray::new();
    id.push(Variant::from(index));
    id.push(Variant::from(generation));
    id
}
