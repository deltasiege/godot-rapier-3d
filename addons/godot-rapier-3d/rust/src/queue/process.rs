use rapier3d::dynamics::RigidBodyHandle;
use rapier3d::geometry::ColliderHandle;

use self::add_or_remove::*;
use self::sim::*;
use self::sync::*;
use crate::objects::{Handle, ObjectBridge};
use crate::queue::Action;
use crate::GR3DPhysicsPipeline;
use crate::LookupIdentifier;
use crate::ObjectKind;
use crate::{GR3DPhysicsState, Lookups};

use super::Actionable;

mod add_or_remove;
mod sim;
mod sync;

pub fn process_insert_action(
    action: Action,
    state: &mut GR3DPhysicsState,
    lookups: &mut Lookups,
) -> Result<(), String> {
    let object_bridge = ObjectBridge::from(&action.data);
    let cuid2 = ensure_unique_cuid2(action.inner_cuid, lookups);
    let handle = insert_into_set(action.data, state)?;
    let iid = action.inner_iid.clone();
    attach_handle_to_node(object_bridge.object_kind.clone(), handle.clone(), iid)?;
    insert_lookup(object_bridge.object_kind, lookups, cuid2, handle.raw, iid)?;
    Ok(())
}

pub fn process_remove_action(
    action: Action,
    state: &mut GR3DPhysicsState,
    lookups: &mut Lookups,
) -> Result<(), String> {
    remove_from_set(action.data, state)?;
    remove_lookup(action.inner_cuid, lookups)?;
    Ok(())
}

pub fn process_parent_action(
    action: Action,
    state: &mut GR3DPhysicsState,
    lookups: &Lookups,
) -> Result<bool, String> {
    match action.data {
        Actionable::ColliderIDWithParentID(col_id, rb_parent_id) => {
            let col_handle = match lookups.get(ObjectKind::Collider, LookupIdentifier::ID, &col_id)
            {
                Some(id_bridge) => ColliderHandle::from(Handle::from(id_bridge.handle_raw)),
                None => {
                    return Err(format!(
                        "Could not find collider handle for '{}' in lookups",
                        col_id
                    ));
                }
            };

            let rb_parent_handle = match rb_parent_id {
                Some(parent_id) => {
                    match lookups.get(ObjectKind::RigidBody, LookupIdentifier::ID, &parent_id) {
                        Some(id_bridge) => {
                            Some(RigidBodyHandle::from(Handle::from(id_bridge.handle_raw)))
                        }
                        None => {
                            return Err(format!(
                                "Could not find rigid body handle for '{}' in lookups",
                                parent_id
                            ));
                        }
                    }
                }
                None => None,
            };

            log::trace!(
                "[AQ]: Setting parent for collider: {:?} {:?}",
                col_handle,
                rb_parent_handle
            );
            state
                .collider_set
                .set_parent(col_handle, rb_parent_handle, &mut state.rigid_body_set);
            Ok(rb_parent_handle.is_some())
        }
        _ => Err(format!(
            "[AQ]: Invalid Actionable passed when trying to set collider parent: '{}'",
            action.data
        )),
    }
}

pub fn process_sync_action(
    action: Action,
    state: &mut GR3DPhysicsState,
    lookups: &mut Lookups,
) -> Result<(), String> {
    let object_bridge = ObjectBridge::from(&action.data);
    match action.data {
        Actionable::NodePos(object_kind, position) => {
            let id_bridge = lookups
                .get(
                    object_kind.clone(),
                    crate::LookupIdentifier::ID,
                    &action.inner_cuid,
                )
                .ok_or("Could not find handle in lookups")?;

            let handle = Handle {
                kind: object_bridge.handle_kind,
                raw: id_bridge.handle_raw,
            };

            set_object_position(handle, position, false, state)?;
        }
        Actionable::ColliderShape(shape) => {
            let id_bridge = lookups
                .get(
                    ObjectKind::Collider,
                    crate::LookupIdentifier::ID,
                    &action.inner_cuid,
                )
                .ok_or("Could not find handle in lookups")?;

            let handle = Handle {
                kind: object_bridge.handle_kind,
                raw: id_bridge.handle_raw,
            };

            set_collider_shape(handle, shape, state)?;
        }
        _ => {
            return Err("Invalid Actionable type".to_string());
        }
    }
    Ok(())
}

pub fn process_sim_action(
    action: Action,
    pipeline: &mut GR3DPhysicsPipeline,
    lookups: &Lookups,
) -> Result<(), String> {
    match action.data {
        Actionable::Step => {
            step(pipeline);
            pipeline.sync_active_g2r(lookups)
        }
        _ => {
            return Err("Invalid Actionable type".to_string());
        }
    }
    Ok(())
}
