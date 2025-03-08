use crate::objects::{Handle, HandleKind, RapierCharacterBody3D, RapierRigidBody3D};
use crate::utils::{isometry_to_transform, node_from_instance_id};
use crate::{LookupIdentifier, Lookups, ObjectKind};
use godot::classes::Node3D;
use godot::obj::WithBaseField;
use godot::prelude::*;
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
}

impl GR3DPhysicsPipeline {
    pub fn new() -> Self {
        Self {
            state: GR3DPhysicsState::default(),
            physics_pipeline: PhysicsPipeline::new(),
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
            self.sync_g2r(Handle::from(&handle), rb, lookups)
                .map_err(crate::handle_error)
                .ok();
        }
    }

    // Changes active RigidBody Godot node transforms to match Rapier transforms
    pub fn sync_active_g2r(&self, lookups: &Lookups) {
        let active_dynamic_bodies = self.state.island_manager.active_dynamic_bodies();
        let active_kinematic_bodies = self.state.island_manager.active_kinematic_bodies();
        let active_bodies = active_dynamic_bodies
            .iter()
            .chain(active_kinematic_bodies.iter());

        for active_body_handle in active_bodies {
            match self.state.rigid_body_set.get(*active_body_handle) {
                Some(rb) => self
                    .sync_g2r(Handle::from(active_body_handle), rb, lookups)
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

    // Changes RigidBody Godot node transform to match Rapier transform
    pub fn sync_g2r(
        &self,
        handle: Handle,
        rigid_body: &RigidBody,
        lookups: &Lookups,
    ) -> Result<(), String> {
        let id_bridge = lookups
            .get(
                ObjectKind::RigidBody,
                LookupIdentifier::Handle,
                &handle.to_string(),
            )
            .ok_or(format!("sync_g2r: No id_bridge found for {:?}", handle))?;
        let instance_id = InstanceId::from_i64(id_bridge.instance_id);
        let transform = isometry_to_transform(*rigid_body.position());
        let is_character = rigid_body.is_kinematic();
        match is_character {
            true => {
                let node_pointer = node_from_instance_id::<RapierCharacterBody3D>(instance_id)?;
                set_global_transform(node_pointer, transform);
            }
            false => {
                let node_pointer = node_from_instance_id::<RapierRigidBody3D>(instance_id)?;
                set_global_transform(node_pointer, transform);
            }
        }

        Ok(())
    }

    // Returns the shape of a given collider
    pub fn get_collider(&self, handle: Handle) -> Result<&Collider, String> {
        self.state
            .collider_set
            .get(ColliderHandle::from(&handle))
            .ok_or(format!("Could not find collider {:?} in pipeline", handle))
    }

    pub fn get_collider_mut(&mut self, handle: Handle) -> Result<&mut Collider, String> {
        self.state
            .collider_set
            .get_mut(ColliderHandle::from(&handle))
            .ok_or(format!("Could not find collider {:?} in pipeline", handle))
    }

    pub fn get_debug_info(&self) -> Result<String, String> {
        let engine = crate::engine::get_engine()?;
        let mut ret_map = HashMap::new();
        let rb_key = format!("Rigid bodies ({})", self.state.rigid_body_set.len());
        let col_key = format!("Colliders ({})", self.state.collider_set.len());

        let handles_to_entries = |handle: Handle| {
            let object_kind = ObjectKind::from(handle.kind.clone());
            let pos = self.get_object_position(handle.clone());
            if pos.is_err() {
                return DebugEntry {
                    id: "Unknown".to_string(),
                    pos: RVector::zeros(),
                    rot: (0.0, 0.0, 0.0),
                };
            }
            match engine.bind().lookups.get(
                object_kind,
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

pub fn set_global_transform<T: WithBaseField<Base = Node3D>>(
    mut node_pointer: Gd<T>,
    transform: Transform3D,
) {
    let mut bind = node_pointer.bind_mut();
    let mut base = bind.base_mut();
    base.set_notify_transform(false);
    base.set_global_transform(transform);
    base.set_notify_transform(true);
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
