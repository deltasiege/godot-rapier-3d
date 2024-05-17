use rapier3d::dynamics::RigidBodyHandle;
use rapier3d::geometry::ColliderHandle;
use rapier3d::math::{Isometry, Real};

use crate::objects::{Handle, HandleKind};
use crate::GR3DPhysicsState;

pub fn set_object_position(
    handle: Handle,
    position: Isometry<Real>,
    wake_up: bool,
    state: &mut GR3DPhysicsState,
) -> Result<(), String> {
    match handle.kind {
        HandleKind::RigidBodyHandle => {
            let rigid_body = state
                .rigid_body_set
                .get_mut(RigidBodyHandle::from(handle))
                .ok_or("Could not find rigid body in pipeline")?;
            rigid_body.set_position(position, wake_up);
            Ok(())
        }
        HandleKind::ColliderHandle => {
            let collider = state
                .collider_set
                .get_mut(ColliderHandle::from(handle))
                .ok_or("Could not find collider in pipeline")?;
            collider.set_position(position);
            Ok(())
        }
        _ => Err("Invalid handle type".to_string()),
    }
}
