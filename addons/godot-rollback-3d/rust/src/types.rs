use godot::prelude::*;
use rapier3d::parry::utils::hashmap::HashMap;
use rapier3d::prelude::*;
use serde::{Deserialize, Serialize};

use crate::nodes::NodeData;

// Indices
pub type Tick = usize; // Current timestep of the simulation
pub type GRUID = (u8, u32); // Godot Rollback unique identifier (peer_index, generation)
pub type RapierHandle = (u32, u32); // Rapier unique identifier (id, generation)

// Input
pub type InputMap = HashMap<GString, Variant>; // Input keys+values for a single tick

// Network
pub type PeerId = i64; // Godot peer ID
pub type PeerIndex = u8; // Index of the peer in the host's list of peers
pub type PeerMap = Array<PeerId>; // List of peers_ids in order of peer index
pub type TickData = PackedByteArray; // Serialized tick data sent via NetworkAdapter
pub type UnixEpoch = u128; // Unix epoch in milliseconds

// Nodes
pub type NodeMap = HashMap<GRUID, NodeData>; // Map of all nodes that have been spawned and despawned into Rapier + Godot.

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum RollbackNodeClass {
    RollbackArea3D,
    RollbackCollisionShape3D,
    RollbackKinematicCharacter3D,
    RollbackPIDCharacter3D,
    RollbackRigidBody3D,
    RollbackStaticBody3D,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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
