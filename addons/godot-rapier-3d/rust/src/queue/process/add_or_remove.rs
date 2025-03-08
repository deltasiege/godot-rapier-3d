use crate::objects::Handle;
use crate::queue::Actionable;
use crate::utils::{node_from_instance_id, HasCUID2Field, HasHandleField};
use crate::{GR3DPhysicsState, IDBridge, Lookups, ObjectKind};
use godot::obj::{InstanceId, WithBaseField};
use godot::prelude::*;

// Returns input cuid2 if the CUID2 is unique, otherwise generates and returns a new one
pub fn ensure_unique_cuid2(cuid2: String, lookups: &Lookups) -> String {
    match lookups.is_cuid2_unique(&cuid2) {
        true => cuid2,
        false => {
            log::warn!("CUID2 collision, generating new one for {:?}", cuid2);
            crate::cuid2()
        }
    }
}

pub fn insert_into_set(object: Actionable, state: &mut GR3DPhysicsState) -> Result<Handle, String> {
    match object {
        Actionable::RigidBody(rb) | Actionable::Character(rb) => {
            let handle = state.rigid_body_set.insert(rb);
            Ok(Handle::from(&handle))
        }
        Actionable::Collider(col) => {
            let handle = state.collider_set.insert(col);
            Ok(Handle::from(&handle))
        }
        Actionable::ColliderWithParent(col, rb_parent_handle) => {
            let handle = state.collider_set.insert_with_parent(
                col,
                rb_parent_handle,
                &mut state.rigid_body_set,
            );
            Ok(Handle::from(&handle))
        }
        _ => Err(format!("[AQ]: Could not insert object '{:?}'", object)),
    }
}

pub fn remove_from_set(object: Actionable, state: &mut GR3DPhysicsState) -> Result<(), String> {
    match object {
        Actionable::RigidBodyHandle(handle) => {
            state.rigid_body_set.remove(
                handle,
                &mut state.island_manager,
                &mut state.collider_set,
                &mut state.impulse_joint_set,
                &mut state.multibody_joint_set,
                true,
            );
            Ok(())
        }
        Actionable::ColliderHandle(handle) => {
            state.collider_set.remove(
                handle,
                &mut state.island_manager,
                &mut state.rigid_body_set,
                false,
            );
            Ok(())
        }
        Actionable::Invalid => Ok(()),
        _ => Err(format!("[AQ]: Could not remove object '{:?}'", object)),
    }
}

pub fn attach_cuid2_to_node<T: HasCUID2Field + WithBaseField<Base = Node3D>>(
    cuid2: String,
    instance_id: i64,
) -> Result<(), String> {
    let iid = InstanceId::from_i64(instance_id);
    let mut node = node_from_instance_id::<T>(iid)?;
    node.bind_mut().set_cuid2(cuid2);
    Ok(())
}

pub fn attach_handle_to_node<T: HasHandleField + WithBaseField<Base = Node3D>>(
    handle: Handle,
    instance_id: i64,
) -> Result<(), String> {
    let iid = InstanceId::from_i64(instance_id);
    let mut node = node_from_instance_id::<T>(iid)?;
    node.bind_mut().set_handle(handle);
    node.bind_mut()
        .base_mut()
        .try_call_deferred("set_notify_transform", &[Variant::from(true)])
        .map_err(|e| format!("[AQ]: Could not set notify transform on node: {:?}", e))?;
    Ok(())
}

pub fn insert_lookup(
    object_kind: ObjectKind,
    lookups: &mut Lookups,
    cuid: String,
    handle_raw: [u32; 2],
    instance_id: i64,
) -> Result<(), String> {
    let id_bridge = IDBridge::new(cuid, handle_raw, instance_id);
    id_bridge.is_valid()?;
    lookups.insert(object_kind, id_bridge)?;
    Ok(())
}

pub fn remove_lookup(cuid: String, lookups: &mut Lookups) -> Result<(), String> {
    lookups.remove(cuid)?;
    Ok(())
}
