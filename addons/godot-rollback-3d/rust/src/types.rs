use godot::prelude::*;
use rapier3d::parry::utils::hashmap::HashMap;
use rapier3d::prelude::*;
use serde::{Deserialize, Serialize};

use crate::nodes::NodeData;

// Indices
pub type Tick = u64; // Current timestep of the simulation
pub type GRUID = (u8, u32); // Godot Rollback unique identifier (peer_index, generation)
pub type RapierHandle = (u32, u32); // Rapier unique identifier (id, generation)

// Input
pub type InputMap = HashMap<GString, Variant>; // Input keys+values for a single tick

// Network
pub type PeerId = i64; // Godot peer ID
pub type PeerIndex = u8; // Index of the peer in the host's list of peers
pub type PeerMap = Array<PeerId>; // List of peers_ids in order of peer index. 0 = ambient, 1 = host, 2+ = client
pub type TickData = PackedByteArray; // Serialized tick data sent via NetworkAdapter
pub type UnixEpoch = u128; // Unix epoch in milliseconds

// Nodes
pub type NodeMap = HashMap<GRUID, NodeData>; // Map of all nodes that have been spawned and despawned into Rapier + Godot.

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum RollbackNodeClass {
    RollbackArea3D,
    RollbackCollisionShape3D,
    RollbackKinematicCharacter3D,
    RollbackPIDCharacter3D,
    RollbackRigidBody3D,
    RollbackStaticBody3D,
}

impl std::fmt::Display for RollbackNodeClass {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum RapierBuilder {
    RigidBody(RigidBody),
    Collider(ColliderBuilder),
}

impl std::fmt::Display for RapierBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RapierBuilder::RigidBody(_) => write!(f, "RigidBody(_)"),
            RapierBuilder::Collider(_) => write!(f, "Collider(_)"),
        }
    }
}

impl TryFrom<&GString> for RollbackNodeClass {
    type Error = &'static str;

    fn try_from(value: &GString) -> Result<Self, Self::Error> {
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

impl RollbackNodeClass {
    pub fn try_from_pointer(pointer: &Gd<impl Inherits<Object>>, silent: bool) -> Option<Self> {
        let class_name = pointer.upcast_ref::<Object>().get_class();
        match RollbackNodeClass::try_from(&class_name) {
            Ok(class) => Some(class),
            Err(_) => {
                if !silent {
                    log::error!("Unknown RollbackNode class: {}", class_name);
                }
                None
            }
        }
    }
}
