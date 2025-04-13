use rapier3d::na::Vector3;
use rapier3d::prelude::*;

use crate::interface::get_gr3d;
use crate::nodes::*;

// Trait that applies to rigid bodies - can be affected by external forces, impulses etc.

pub trait Forceable: HasNodeData + RollbackNode {
    fn get_body_state(&self) -> BodyState {
        self.try_get_body_state().unwrap_or_default()
    }

    fn try_get_body_state(&self) -> Option<BodyState> {
        if let Some(gr3d) = get_gr3d() {
            let handle = self.get_node_data()?.get_rapier_handle().left()?;
            if gr3d.bind().world.physics.bodies.contains(handle) {
                let body = &gr3d.bind().world.physics.bodies[handle];
                return Some(BodyState::from_rigidbody(body));
            }
        }
        None
    }

    // TODO ability to apply impulses, forces etc.
}

impl Forceable for RollbackKinematicCharacter3D {}
impl Forceable for RollbackPIDCharacter3D {}
impl Forceable for RollbackRigidBody3D {}

pub struct BodyState {
    pub linvel: Vector3<Real>,
    pub angvel: Vector3<Real>,
    pub sleeping: bool,
    pub moving: bool,
}

impl BodyState {
    pub fn from_rigidbody(body: &RigidBody) -> Self {
        let linvel = body.linvel();
        let angvel = body.angvel();
        Self {
            linvel: Vector3::new(linvel.x, linvel.y, linvel.z),
            angvel: Vector3::new(angvel.x, angvel.y, angvel.z),
            sleeping: body.is_sleeping(),
            moving: body.is_moving(),
        }
    }
}

impl Default for BodyState {
    fn default() -> Self {
        Self {
            linvel: Vector3::new(0.0, 0.0, 0.0),
            angvel: Vector3::new(0.0, 0.0, 0.0),
            sleeping: false,
            moving: false,
        }
    }
}
