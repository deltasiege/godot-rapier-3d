use crate::Rapier3DSingleton;
use godot::builtin::Array as GArray;
use godot::builtin::Quaternion as GQuaternion;
use godot::builtin::Vector3 as GVector;
use godot::prelude::*;
use nalgebra::Quaternion as NAQuaternion;
use nalgebra::Vector3 as NAVector;
use rapier3d::math::Rotation;
use rapier3d::prelude::*;

pub const AUTOLOAD_NAME: &str = "Rapier3D";
pub const AUTOLOAD_PATH: &str = "res://addons/godot-rapier-3d/Rapier3D.gd";
pub const ENGINE_SINGLETON_NAME: &str = "Rapier3DEngine";

pub fn get_engine_singleton() -> Option<Gd<Rapier3DSingleton>> {
    let gd_pointer = godot::engine::Engine::singleton().get_singleton(get_engine_singleton_name());

    match gd_pointer {
        Some(gd_pointer) => {
            let casted = gd_pointer.cast::<Rapier3DSingleton>();
            Some(casted)
        }
        None => {
            godot_error!("Could not find Rapier3D singleton");
            None
        }
    }
}

pub fn get_engine_singleton_name() -> StringName {
    StringName::from(ENGINE_SINGLETON_NAME)
}

pub fn get_autoload_name() -> GString {
    GString::from(AUTOLOAD_NAME)
}

pub fn get_autoload_path() -> GString {
    GString::from(AUTOLOAD_PATH)
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
