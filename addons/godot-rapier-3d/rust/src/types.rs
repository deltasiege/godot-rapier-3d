use godot::prelude::*;
use rapier3d::parry::utils::hashmap::HashMap;
use rapier3d::prelude::*;
use serde::{Deserialize, Serialize};

pub type Tick = usize;
pub type GRUID = (u8, u32); // Godot Rollback unique identifier (peer_index, generation)
pub type RapierHandle = (u32, u32); // Rapier unique identifier (id, generation)

pub type InputMap = HashMap<String, Variant>; // Input keys+values for a single tick

#[derive(Serialize, Deserialize, Clone)]
pub enum RollbackNodeClass {
    RollbackArea3D,
    RollbackCollisionShape3D,
    RollbackKinematicCharacter3D,
    RollbackPIDCharacter3D,
    RollbackRigidBody3D,
    RollbackStaticBody3D,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum RapierBuilder {
    RigidBody(RigidBody),
    Collider(ColliderBuilder),
}

impl TryFrom<GString> for RollbackNodeClass {
    type Error = &'static str;

    fn try_from(value: GString) -> Result<Self, Self::Error> {
        match value.to_string().as_str() {
            "RollbackArea3D" => Ok(RollbackNodeClass::RollbackArea3D),
            "RollbackCollisionShape3D" => Ok(RollbackNodeClass::RollbackCollisionShape3D),
            "RollbackKinematicCharacter3D" => Ok(RollbackNodeClass::RollbackKinematicCharacter3D),
            "RollbackPIDCharacter3D" => Ok(RollbackNodeClass::RollbackPIDCharacter3D),
            "RollbackRigidBody3D" => Ok(RollbackNodeClass::RollbackRigidBody3D),
            "RollbackStaticBody3D" => Ok(RollbackNodeClass::RollbackStaticBody3D),
            _ => Err("Unknown RollbackNode class"),
        }
    }
}
