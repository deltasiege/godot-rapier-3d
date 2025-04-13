use rapier3d::{parry::either::Either, prelude::*};
use serde::{Deserialize, Serialize};

use crate::nodes::NodeBlueprint;
use crate::types::*;
use crate::{impl_trait_for_all_nodes, World};

#[derive(Serialize, Deserialize, Clone, Debug)]
/// Information known about a node once it has been spawned in both Godot and Rapier.
pub struct NodeData {
    pub gruid: GRUID,
    pub rapier_handle: RapierHandle,
    pub spawn_tick: Tick,
    pub despawn_tick: Option<Tick>,
    pub blueprint: NodeBlueprint,
    // pub child_colliders: Vec<GRUID>,
}

impl NodeData {
    pub fn get_rapier_handle(&self) -> Either<RigidBodyHandle, ColliderHandle> {
        match self.blueprint.rapier_builder {
            RapierBuilder::RigidBody(_) => Either::Left(RigidBodyHandle::from_raw_parts(
                self.rapier_handle.0,
                self.rapier_handle.1,
            )),
            RapierBuilder::Collider(_) => Either::Right(ColliderHandle::from_raw_parts(
                self.rapier_handle.0,
                self.rapier_handle.1,
            )),
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
}

pub trait HasNodeData {
    fn get_node_data(&self) -> Option<NodeData>;
    fn set_node_data(&mut self, node_data: NodeData);

    fn get_rigid_body(&self, world: &World) -> Option<RigidBody> {
        let node_data = self.get_node_data()?;
        node_data.get_rigid_body(world)
    }

    fn get_collider(&self, world: &World) -> Option<Collider> {
        let node_data = self.get_node_data()?;
        node_data.get_collider(world)
    }
}

impl_trait_for_all_nodes!(HasNodeData, {
    fn get_node_data(&self) -> Option<NodeData> {
        self.node_data.clone()
    }

    fn set_node_data(&mut self, node_data: NodeData) {
        self.node_data = Some(node_data);
    }
});
