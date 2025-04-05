use cuid2;
use godot::prelude::*;

use crate::nodes::{
    RapierArea3D, RapierCollisionShape3D, RapierKinematicCharacter3D, RapierPIDCharacter3D,
    RapierRigidBody3D, RapierStaticBody3D,
};

pub trait Identifiable {
    fn get_cuid(&self) -> GString;
    fn set_cuid(&mut self, cuid: GString);
    fn has_cuid(&self) -> bool;
    fn get_handle_raw(&self) -> Option<(u32, u32)>;
    fn set_handle_raw(&mut self, handle: (u32, u32));
}

macro_rules! impl_identifiable {
    ($type:ty) => {
        impl Identifiable for $type {
            fn get_cuid(&self) -> GString {
                match self.base().has_meta("cuid") {
                    true => GString::from_variant(&self.base().get_meta("cuid")),
                    false => {
                        log::error!("Could not retrieve cuid of: '{:?}' Please run 'addons/godot-rapier-3d/gd/fixup_cuids.gd' to resolve", self);
                        GString::new()
                    }
                }
            }

            fn has_cuid(&self) -> bool {
                self.base().has_meta("cuid")
            }

            fn set_cuid(&mut self, cuid: GString) {
                self.base_mut().set_meta("cuid", &cuid.to_variant());
            }

            fn get_handle_raw(&self) -> Option<(u32, u32)> {
                if self.handle.len() != 2 {
                    return None;
                }
                Some((self.handle.at(0), self.handle.at(1)))
            }

            fn set_handle_raw(&mut self, handle: (u32, u32)) {
                self.handle = Array::from(&[handle.0, handle.1]);
            }
        }
    };
}

impl_identifiable!(RapierArea3D);
impl_identifiable!(RapierKinematicCharacter3D);
impl_identifiable!(RapierCollisionShape3D);
impl_identifiable!(RapierRigidBody3D);
impl_identifiable!(RapierStaticBody3D);
impl_identifiable!(RapierPIDCharacter3D);

pub fn generate_cuid() -> GString {
    cuid2::slug().into() // 10 characters
}
