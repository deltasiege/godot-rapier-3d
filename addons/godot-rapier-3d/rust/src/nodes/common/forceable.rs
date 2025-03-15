use godot::prelude::*;
use rapier3d::prelude::RigidBodyHandle;

use super::super::{
    pid_character::RapierPIDCharacter3D, RapierKinematicCharacter3D, RapierRigidBody3D,
};
use super::identifiable::Identifiable;
use super::IRapierObject;
use crate::interface::get_singleton;

// Trait that applies to rigid bodies - can be affected by external forces, impulses etc.

pub trait Forceable: IRapierObject + Identifiable {
    fn get_body_state(&self) -> BodyState {
        if let Some(singleton) = get_singleton() {
            if let Some(raw_handle) = self.get_handle_raw() {
                let handle = RigidBodyHandle::from_raw_parts(raw_handle.0, raw_handle.1);
                if singleton.bind().world.physics.bodies.contains(handle) {
                    let body = &singleton.bind().world.physics.bodies[handle];
                    let linvel = body.linvel();
                    let angvel = body.angvel();
                    return BodyState {
                        linvel: Vector3::new(linvel.x, linvel.y, linvel.z),
                        angvel: Vector3::new(angvel.x, angvel.y, angvel.z),
                        sleeping: body.is_sleeping(),
                        moving: body.is_moving(),
                    };
                }
            }
        }
        BodyState::empty()
    }

    // TODO apply impulses, forces etc.
}

macro_rules! impl_forceable {
    ($t:ty) => {
        impl Forceable for $t {}
    };
}

impl_forceable!(RapierKinematicCharacter3D);
impl_forceable!(RapierRigidBody3D);
impl_forceable!(RapierPIDCharacter3D);

pub struct BodyState {
    pub linvel: Vector3,
    pub angvel: Vector3,
    pub sleeping: bool,
    pub moving: bool,
}

impl BodyState {
    pub fn empty() -> Self {
        BodyState {
            linvel: Vector3::ZERO,
            angvel: Vector3::ZERO,
            sleeping: false,
            moving: false,
        }
    }
}
