use rapier3d::{parry::either::Either, prelude::*};
use serde::{Deserialize, Serialize};

use crate::{impl_trait_for_all_nodes, types::*};

#[derive(Serialize, Deserialize, Clone)]
pub struct NodeBlueprint {
    pub gruid: GRUID,
    pub class: RollbackNodeClass,
    pub snapshottable: bool,
    pub spawn_isometry: Isometry<Real>,
    pub children: Vec<GRUID>,
    pub parent: Option<GRUID>,

    // Godot specific
    pub name: String,
    pub tree_path: String,
    pub resource_path: String,

    // Rapier specific
    pub rapier_handle: RapierHandle,
    pub rapier_builder: RapierBuilder,
}

impl NodeBlueprint {
    pub fn get_parent_path(&self) -> String {
        self.tree_path
            .split('/')
            .filter(|&s| s != self.name)
            .collect::<Vec<&str>>()
            .join("/")
    }

    pub fn get_rapier_handle(&self) -> Either<RigidBodyHandle, ColliderHandle> {
        match self.rapier_builder {
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
}

pub trait HasBlueprint {
    fn get_blueprint(&self) -> Option<NodeBlueprint>;
    fn set_blueprint(&mut self, blueprint: NodeBlueprint);
}

impl_trait_for_all_nodes!(HasBlueprint, {
    fn get_blueprint(&self) -> Option<NodeBlueprint> {
        self.blueprint.clone()
    }

    fn set_blueprint(&mut self, blueprint: NodeBlueprint) {
        self.blueprint = Some(blueprint);
    }
});
