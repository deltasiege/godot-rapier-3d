use crate::objects::{
    Handle, HandleKind, RapierCharacterBody3D, RapierCollider3D, RapierRigidBody3D,
};
use crate::queue::Action;
use crate::{GR3DPhysicsPipeline, GR3DPhysicsState, LookupIdentifier, Lookups, ObjectKind};
use godot::global::godot_print;
use rapier3d::dynamics::{RigidBody, RigidBodyHandle, RigidBodyType};
use rapier3d::geometry::{Collider, ColliderHandle, Compound, Shape, SharedShape};
use rapier3d::math::{Isometry, Real, Translation, Vector};
use rapier3d::pipeline::QueryFilter;

use self::add_or_remove::*;
use self::sim::*;
use self::sync::*;
use super::Actionable;

mod add_or_remove;
mod sim;
mod sync;

pub fn process_insert_action(
    action: Action,
    state: &mut GR3DPhysicsState,
    lookups: &mut Lookups,
) -> Result<(), String> {
    let object_kind = ObjectKind::from(&action.data);
    let cuid2 = ensure_unique_cuid2(action.inner_cuid, lookups);
    let handle = insert_into_set(action.data, state)?;
    let iid = action.inner_iid.clone();
    match object_kind {
        ObjectKind::RigidBody => {
            attach_cuid2_to_node::<RapierRigidBody3D>(cuid2.clone(), iid)?;
            attach_handle_to_node::<RapierRigidBody3D>(handle.clone(), iid)?;
        }
        ObjectKind::Collider => {
            attach_cuid2_to_node::<RapierCollider3D>(cuid2.clone(), iid)?;
            attach_handle_to_node::<RapierCollider3D>(handle.clone(), iid)?;
        }
        ObjectKind::Character => {
            attach_cuid2_to_node::<RapierCharacterBody3D>(cuid2.clone(), iid)?;
            attach_handle_to_node::<RapierCharacterBody3D>(handle.clone(), iid)?;
        }
        ObjectKind::Invalid => {
            return Err("Invalid object kind".to_string());
        }
    }

    insert_lookup(object_kind, lookups, cuid2, handle.raw, iid)?;
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

            let col_position = {
                let col = state.collider_set.get(col_handle).ok_or(format!(
                    "Could not find collider {:?} in pipeline",
                    col_handle
                ))?;
                col.position().clone()
            };

            let rb_position = match rb_parent_handle {
                Some(rb_handle) => {
                    let rb = state.rigid_body_set.get(rb_handle).ok_or(format!(
                        "Could not find rigid body {:?} in pipeline",
                        rb_handle
                    ))?;
                    rb.position().clone()
                }
                None => Isometry::identity(),
            };
            let pos_wrt_parent = rb_position.inverse() * col_position;
            godot_print!("Col pos {:?}", col_position.translation.vector);
            godot_print!("RB pos {:?}", rb_position.translation.vector);
            godot_print!(
                "Collider position wrt parent: {:?}",
                pos_wrt_parent.translation.vector
            );

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
    let handle_kind = HandleKind::from(ObjectKind::from(&action.data));
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
                kind: handle_kind,
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
                kind: handle_kind,
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
            step(&mut pipeline.state, &mut pipeline.physics_pipeline);
            pipeline.sync_active_g2r(lookups)
        }
        Actionable::MoveCharacter {
            cuid2,
            controller,
            amount,
            delta_time,
        } => {
            let state = &pipeline.state.clone();
            let id_bridge = lookups
                .get(ObjectKind::Character, LookupIdentifier::ID, &cuid2)
                .ok_or(format!("No character handle for '{}' in lookups", cuid2))?;

            let rigid_body_handle = RigidBodyHandle::from(Handle::from(id_bridge.handle_raw));
            let rigid_body = get_rigid_body(rigid_body_handle, state)?;

            let colliders = rigid_body.colliders();
            match colliders.len() {
                0 => {
                    log::error!("No colliders found for character {:?}", cuid2);
                    return Ok(());
                }
                _ => {}
            }

            let shapes_vec = colliders
                .iter()
                .filter_map(|collider_handle| {
                    let collider = get_collider(collider_handle.clone(), state);
                    match collider {
                        Ok(c) => Some((c.position().clone(), c.shared_shape().clone())),
                        Err(e) => {
                            log::error!("{}", e);
                            None
                        }
                    }
                })
                .collect::<Vec<(Isometry<Real>, SharedShape)>>();

            let compound_shape = Compound::new(shapes_vec);
            let mut collisions = vec![];

            let query_filter = QueryFilter::default().exclude_rigid_body(rigid_body_handle);
            for col_handle in colliders {
                query_filter.exclude_collider(*col_handle);
            }

            let result = controller.move_shape(
                delta_time,
                &state.rigid_body_set,
                &state.collider_set,
                &state.query_pipeline,
                &compound_shape,
                rigid_body.position(),
                amount,
                query_filter,
                |collision| {
                    let handle = Handle::from(&collision.handle);
                    let id_bridge = lookups
                        .get(
                            ObjectKind::Collider,
                            LookupIdentifier::Handle,
                            &handle.to_string(),
                        )
                        .ok_or(format!("No collider handle for '{}' in lookups", handle));
                    godot_print!("Collision: {:?}", id_bridge);
                    collisions.push(collision);
                },
            );

            let rigid_body_mut = get_rigid_body_mut(rigid_body_handle, &mut pipeline.state)?;

            match rigid_body.body_type() {
                RigidBodyType::KinematicPositionBased => {
                    let mut new_pos = rigid_body.position().clone();
                    new_pos.append_translation_mut(&Translation::from(result.translation));
                    rigid_body_mut.set_next_kinematic_translation(new_pos.translation.vector);
                }
                RigidBodyType::KinematicVelocityBased => {
                    rigid_body_mut.set_linvel(result.translation / delta_time, true);
                }
                _ => {}
            }

            log::trace!("Moving Character {:?} by: {:?}", cuid2, result.translation);

            for collision in collisions.iter() {
                controller.solve_character_collision_impulses(
                    delta_time,
                    &mut pipeline.state.rigid_body_set,
                    &state.collider_set,
                    &state.query_pipeline,
                    &compound_shape,
                    compound_shape.mass_properties(1.0).mass() + rigid_body.mass(), // TODO how to get density?
                    &collision,
                    QueryFilter::default().exclude_rigid_body(rigid_body_handle),
                );
            }
        }
        _ => {
            return Err("Invalid Actionable type".to_string());
        }
    }
    Ok(())
}

fn get_collider(
    collider_handle: ColliderHandle,
    state: &GR3DPhysicsState,
) -> Result<&Collider, String> {
    state.collider_set.get(collider_handle).ok_or(format!(
        "Could not find collider {:?} in pipeline",
        collider_handle
    ))
}

fn get_rigid_body(
    rigid_body_handle: RigidBodyHandle,
    state: &GR3DPhysicsState,
) -> Result<&RigidBody, String> {
    state.rigid_body_set.get(rigid_body_handle).ok_or(format!(
        "Could not find rigid body {:?} in pipeline",
        rigid_body_handle
    ))
}

fn get_rigid_body_mut(
    rigid_body_handle: RigidBodyHandle,
    state: &mut GR3DPhysicsState,
) -> Result<&mut RigidBody, String> {
    state
        .rigid_body_set
        .get_mut(rigid_body_handle)
        .ok_or(format!(
            "Could not find rigid body {:?} in pipeline",
            rigid_body_handle
        ))
}
