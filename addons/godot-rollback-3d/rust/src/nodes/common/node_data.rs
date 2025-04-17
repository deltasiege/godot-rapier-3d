use godot::obj::WithBaseField;
use godot::prelude::*;
use rapier3d::{parry::either::Either, prelude::*};
use serde::{Deserialize, Serialize};

use crate::nodes::NodeBlueprint;
use crate::types::*;
use crate::utils::*;
use crate::{impl_trait_for_all_nodes, World};

#[derive(Serialize, Deserialize, Clone, Debug)]
/// Information known about a node once it has been spawned in both Godot and Rapier.
pub struct NodeData {
    pub gruid: GRUID,
    pub rapier_handle: RapierHandle,
    pub spawn_tick: Tick,
    pub despawn_tick: Option<Tick>,
    pub blueprint: NodeBlueprint,
    pub node_state: Vector3,
}

// UP TO - need to allow saving arbitrary node state into node data from GDScript

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableEntry {
    pub key: String,
    pub value: SerializableVariant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializableVariant {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Color(Color),
    Vector2(Vector2),
    Vector3(Vector3),
}

impl NodeData {
    pub fn new(
        gruid: GRUID,
        rapier_handle: RapierHandle,
        spawn_tick: Tick,
        blueprint: NodeBlueprint,
    ) -> Self {
        Self {
            gruid,
            rapier_handle,
            spawn_tick,
            despawn_tick: None,
            blueprint,
            node_state: Vector3::ZERO,
        }
    }

    pub fn get_rapier_handle(&self) -> Either<RigidBodyHandle, ColliderHandle> {
        match self.blueprint.rapier_builder {
            RapierBuilder::RigidBody(_) => Either::Left(self.rapier_handle.to_rigid_body_handle()),
            RapierBuilder::Collider(_) => Either::Right(self.rapier_handle.to_collider_handle()),
        }
    }

    pub fn is_alive(&self, tick: Tick) -> bool {
        if let Some(despawn_tick) = self.despawn_tick {
            return self.spawn_tick <= tick && tick < despawn_tick;
        }
        self.spawn_tick <= tick
    }

    pub fn get_rigid_body(&self, world: &World) -> Option<RigidBody> {
        let handle = self.get_rapier_handle().left()?;
        let body = world.physics.bodies.get(handle)?;
        Some(body.clone())
    }

    pub fn get_collider(&self, world: &World) -> Option<Collider> {
        let handle = self.get_rapier_handle().right()?;
        let collider = world.physics.colliders.get(handle)?;
        Some(collider.clone())
    }

    pub fn serialize(&self) -> PackedByteArray {
        encode_to_packed_byte_array(self)
    }

    pub fn from_packed_byte_array(data: PackedByteArray) -> Option<Self> {
        decode_from_packed_byte_array(&data)
    }

    pub fn get_from_node(node: &Gd<Node3D>) -> Option<Self> {
        let packed_byte_array = Self::get_ser_from_node(node);
        Self::from_packed_byte_array(packed_byte_array)
    }

    pub fn get_ser_from_node(node: &Gd<Node3D>) -> PackedByteArray {
        if !node.has_meta("gr3d_data") {
            return PackedByteArray::new();
        }
        let variant = node.get_meta("gr3d_data");
        match variant.try_to::<PackedByteArray>().ok() {
            Some(packed_byte_array) => packed_byte_array,
            None => PackedByteArray::new(),
        }
    }

    pub fn set_on_node(&self, node: &mut Gd<Node3D>) {
        node.set_meta("gr3d_data", &self.serialize().to_variant());
    }
}

pub trait HasNodeData: WithBaseField + GodotClass<Base = Node3D> {
    fn get_node_data(&self) -> Option<NodeData> {
        NodeData::get_from_node(&self.base())
    }

    fn get_ser_node_data(&self) -> PackedByteArray {
        NodeData::get_ser_from_node(&self.base())
    }

    fn set_node_data(&mut self, node_data: NodeData) {
        node_data.set_on_node(&mut self.base_mut())
    }

    fn get_rigid_body(&self, world: &World) -> Option<RigidBody> {
        let node_data = self.get_node_data()?;
        node_data.get_rigid_body(world)
    }

    fn get_collider(&self, world: &World) -> Option<Collider> {
        let node_data = self.get_node_data()?;
        node_data.get_collider(world)
    }
}

impl_trait_for_all_nodes!(HasNodeData, {});
