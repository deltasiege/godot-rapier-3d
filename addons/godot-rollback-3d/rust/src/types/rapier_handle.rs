use rapier3d::prelude::{ColliderHandle, RigidBodyHandle};
use serde::{Deserialize, Serialize};

use crate::utils::try_parse;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RapierHandle {
    pub id: u32,
    pub generation: u32,
}

impl RapierHandle {
    pub fn new(id: u32, generation: u32) -> Self {
        Self { id, generation }
    }

    pub fn from_tuple(tuple: (u32, u32)) -> Self {
        Self::new(tuple.0, tuple.1)
    }

    pub fn try_from_string(rapier_handle: &String) -> Option<Self> {
        let parts: Vec<&str> = rapier_handle.split('-').collect();
        if parts.len() != 2 {
            log::error!("Invalid RapierHandle format: {}", rapier_handle);
            return None;
        }
        let id = try_parse::<u32>(parts[0])?;
        let generation = try_parse::<u32>(parts[1])?;
        Some(Self::new(id, generation))
    }

    pub fn to_rigid_body_handle(&self) -> RigidBodyHandle {
        RigidBodyHandle::from_raw_parts(self.id, self.generation)
    }

    pub fn from_rigid_body_handle(handle: RigidBodyHandle) -> Self {
        let raw_parts = handle.into_raw_parts();
        Self::from_tuple(raw_parts)
    }

    pub fn to_collider_handle(&self) -> ColliderHandle {
        ColliderHandle::from_raw_parts(self.id, self.generation)
    }

    pub fn from_collider_handle(handle: ColliderHandle) -> Self {
        let raw_parts = handle.into_raw_parts();
        Self::from_tuple(raw_parts)
    }
}

impl std::fmt::Display for RapierHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}-{}", self.id, self.generation)
    }
}
