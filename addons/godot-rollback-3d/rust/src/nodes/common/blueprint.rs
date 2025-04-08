use godot::prelude::*;
use rapier3d::{parry::either::Either, prelude::*};
use serde::{Deserialize, Serialize};

use crate::impl_trait_for_all_nodes;
use crate::types::*;
use crate::utils::to_string_variant;

#[derive(Serialize, Deserialize, Clone, Debug)]
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

    pub fn to_variant(&self) -> Variant {
        let mut dict = Dictionary::new();
        dict.set("gruid", to_string_variant(self.gruid));
        dict.set("class", to_string_variant(&self.class));
        dict.set("snapshottable", self.snapshottable.to_variant());
        dict.set("spawn_isometry", to_string_variant(self.spawn_isometry));
        dict.set("children", to_string_variant(self.spawn_isometry));
        dict.set("parent", to_string_variant(self.spawn_isometry));
        dict.set("name", self.name.to_variant());
        dict.set("tree_path", self.tree_path.to_variant());
        dict.set("resource_path", self.resource_path.to_variant());
        dict.set("rapier_handle", to_string_variant(self.rapier_handle));
        dict.set("rapier_builder", to_string_variant(&self.rapier_builder));
        dict.to_variant()
    }
}

pub trait HasBlueprint {
    fn get_blueprint(&self) -> Option<NodeBlueprint>;
    fn set_blueprint(&mut self, blueprint: NodeBlueprint);

    fn get_blueprint_variant(&self) -> Variant {
        match self.get_blueprint() {
            Some(blueprint) => blueprint.to_variant(),
            None => Dictionary::new().to_variant(),
        }
    }
}

impl_trait_for_all_nodes!(HasBlueprint, {
    fn get_blueprint(&self) -> Option<NodeBlueprint> {
        self.blueprint.clone()
    }

    fn set_blueprint(&mut self, blueprint: NodeBlueprint) {
        self.blueprint = Some(blueprint);
    }
});
