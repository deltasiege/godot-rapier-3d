use crate::objects::{Handle, HandleKind, ObjectBridge, RapierRigidBody3D};
use crate::utils::{handle_to_instance_id, isometry_to_transform, node_from_instance_id};
use crate::{LookupIdentifier, Lookups};
use godot::obj::WithBaseField;
use rapier3d::math::Vector as RVector;
use rapier3d::prelude::*;
use std::collections::HashMap;
use std::fmt;

mod debug_render_pipeline;
mod state;
pub use state::GR3DPhysicsState;

pub struct GR3DPhysicsPipeline {
    pub state: GR3DPhysicsState,
    pub physics_pipeline: PhysicsPipeline,
    pub physics_hooks: (),
    pub event_handler: (),
}

impl GR3DPhysicsPipeline {
    pub fn new() -> Self {
        Self {
            state: GR3DPhysicsState::default(),
            physics_pipeline: PhysicsPipeline::new(),
            physics_hooks: (),
            event_handler: (),
        }
    }

    // Returns the rapier position of a given object
    pub fn get_object_position(&self, handle: Handle) -> Result<Isometry<Real>, String> {
        match handle.kind {
            HandleKind::RigidBodyHandle => {
                let rigid_body = self
                    .state
                    .rigid_body_set
                    .get(RigidBodyHandle::from(handle))
                    .ok_or("Could not find rigid body in pipeline")?;
                Ok(*rigid_body.position())
            }
            HandleKind::ColliderHandle => {
                let collider = self
                    .state
                    .collider_set
                    .get(ColliderHandle::from(handle))
                    .ok_or("Could not find collider in pipeline")?;
                Ok(*collider.position())
            }
            _ => Err("Invalid handle type".to_string()),
        }
    }

    // Changes all RigidBody Godot node transforms to match Rapier transforms
    pub fn sync_all_g2r(&self, lookups: &Lookups) {
        let dynamic_bodies = self.state.rigid_body_set.iter();
        for (handle, rb) in dynamic_bodies {
            self.sync_g2r(handle, rb, lookups)
                .map_err(crate::handle_error)
                .ok();
        }
    }

    // Changes active RigidBody Godot node transforms to match Rapier transforms
    pub fn sync_active_g2r(&self, lookups: &Lookups) {
        let active_dynamic_bodies = self.state.island_manager.active_dynamic_bodies();
        for active_body_handle in active_dynamic_bodies {
            match self.state.rigid_body_set.get(*active_body_handle) {
                Some(rb) => self
                    .sync_g2r(*active_body_handle, rb, lookups)
                    .map_err(crate::handle_error)
                    .ok(),
                None => {
                    log::error!(
                        "Pipeline: could not find active body {:?}",
                        active_body_handle
                    );
                    continue;
                }
            };
        }
    }

    // Changes specific RigidBody Godot node transform to match Rapier transform
    pub fn sync_g2r(
        &self,
        handle: RigidBodyHandle,
        rigid_body: &RigidBody,
        lookups: &Lookups,
    ) -> Result<(), String> {
        let transform = isometry_to_transform(*rigid_body.position());
        let instance_id = handle_to_instance_id(Handle::from(&handle), lookups)?;
        let mut node = node_from_instance_id::<RapierRigidBody3D>(instance_id)?;
        node.bind_mut().base_mut().set_notify_transform(false);
        node.bind_mut().base_mut().set_global_transform(transform);
        node.bind_mut().base_mut().set_notify_transform(true);
        Ok(())
    }

    // Returns the shape of a given collider
    pub fn get_collider(&self, handle: Handle) -> Result<&Collider, String> {
        self.state
            .collider_set
            .get(ColliderHandle::from(handle.clone()))
            .ok_or(format!("Could not find collider {:?} in pipeline", handle))
    }

    pub fn get_collider_mut(&mut self, handle: Handle) -> Result<&mut Collider, String> {
        self.state
            .collider_set
            .get_mut(ColliderHandle::from(handle.clone()))
            .ok_or(format!("Could not find collider {:?} in pipeline", handle))
    }

    pub fn get_debug_info(&self) -> Result<String, String> {
        let engine = crate::engine::get_engine()?;
        let mut ret_map = HashMap::new();
        let rb_key = format!("Rigid bodies ({})", self.state.rigid_body_set.len());
        let col_key = format!("Colliders ({})", self.state.collider_set.len());

        let handles_to_entries = |handle: Handle| {
            let object_bridge = ObjectBridge::from(handle.kind.clone());
            let pos = self.get_object_position(handle.clone());
            if pos.is_err() {
                return DebugEntry {
                    id: "Unknown".to_string(),
                    pos: RVector::zeros(),
                    rot: (0.0, 0.0, 0.0),
                };
            }
            match engine.bind().lookups.get(
                object_bridge.object_kind,
                LookupIdentifier::Handle,
                &handle.to_string(),
            ) {
                Some(result) => DebugEntry {
                    id: result.cuid2.clone(),
                    pos: pos.clone().unwrap().translation.vector,
                    rot: pos.clone().unwrap().rotation.euler_angles(),
                },
                None => DebugEntry {
                    id: "Unknown".to_string(),
                    pos: RVector::zeros(),
                    rot: (0.0, 0.0, 0.0),
                },
            }
        };

        let rigid_body_entries = self
            .state
            .rigid_body_set
            .iter()
            .map(|(handle, _)| handles_to_entries(Handle::from(&handle)))
            .collect::<Vec<DebugEntry>>();

        let collider_entries = self
            .state
            .collider_set
            .iter()
            .map(|(handle, _)| handles_to_entries(Handle::from(&handle)))
            .collect::<Vec<DebugEntry>>();

        ret_map.insert(rb_key, rigid_body_entries);
        ret_map.insert(col_key, collider_entries);

        Ok(format!("{:#?}", ret_map))
    }
}

pub struct DebugEntry {
    pub id: String,
    pub pos: RVector<f32>,
    pub rot: (f32, f32, f32),
}

impl fmt::Debug for DebugEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: pos: {:?}, rot: {:?}", self.id, self.pos, self.rot)
    }
}
