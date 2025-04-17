use godot::prelude::*;
use rapier3d::control::KinematicCharacterController;
use rapier3d::prelude::*;
use serde::{Deserialize, Serialize};

use crate::nodes::*;
use crate::types::*;
use crate::utils::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
/// Information known at the time of the initial spawn request.
pub struct NodeBlueprint {
    pub class: RollbackNodeClass,
    pub snapshottable: bool, // If any child is not snapshottable, this must be false. TODO: change to is_snapshottable function.
    pub spawn_isometry: Isometry<Real>,

    pub child_colliders: Vec<NodeBlueprint>,

    // Godot specific
    pub tree_path: String,
    pub resource_path: String, // Note: always refers to the root spawned node, even for children.

    // Rapier specific
    pub rapier_builder: RapierBuilder,
    pub rapier_pid_controller: Option<PDControllerSettings>, // TODO: potentially stateful
    pub rapier_kinematic_controller: Option<KinematicCharacterController>, // TODO: potentially stateful
}

impl NodeBlueprint {
    pub fn get_node_name(&self) -> String {
        let mut name = self.tree_path.clone();
        if let Some(last_slash) = name.rfind('/') {
            name = name[last_slash + 1..].to_string();
        }
        name
    }

    pub fn get_parent_path(&self) -> String {
        let mut path = self.tree_path.clone();
        if let Some(last_slash) = path.rfind('/') {
            path = path[..last_slash].to_string();
        }
        path
    }

    pub fn to_variant(&self) -> Variant {
        let mut dict = Dictionary::new();
        dict.set("class", to_string_variant(&self.class));
        dict.set("snapshottable", self.snapshottable.to_variant());
        dict.set("spawn_isometry", to_string_variant(self.spawn_isometry));
        dict.set("tree_path", self.tree_path.to_variant());
        dict.set("resource_path", self.resource_path.to_variant());
        dict.set("rapier_builder", to_string_variant(&self.rapier_builder));
        dict.to_variant()
    }
}

impl std::fmt::Display for NodeBlueprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "NodeBlueprint {{ class: {:?}, snapshottable: {}, spawn_isometry: {}, child_colliders: {}, tree_path: {}, resource_path: {}, rapier_builder: {} }}",
            self.class,
            self.snapshottable,
            self.spawn_isometry,
            self.child_colliders.len(),
            self.tree_path,
            self.resource_path,
            self.rapier_builder,
        )
    }
}
