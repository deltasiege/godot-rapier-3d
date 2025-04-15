use godot::prelude::*;
use rapier3d::prelude::*;
use serde::{Deserialize, Serialize};

use crate::impl_trait_for_all_nodes;
use crate::types::*;
use crate::utils::*;
use crate::world::SpawnRequest;

#[derive(Serialize, Deserialize, Clone, Debug)]
/// Information known at the time of the initial spawn request.
pub struct NodeBlueprint {
    pub class: RollbackNodeClass,
    pub snapshottable: bool, // If any child is not snapshottable, this must be false. TODO: change to is_snapshottable function.
    pub spawn_isometry: Isometry<Real>,

    pub child_colliders: Vec<NodeBlueprint>,

    // Godot specific
    pub tree_path: String,
    pub resource_path: String,

    // Rapier specific
    pub rapier_builder: RapierBuilder,
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

    pub fn from_spawn_request(spawn_request: SpawnRequest) -> Option<Vec<Self>> {
        let mut spawned_node = spawn_into_godot(
            &spawn_request.spawner,
            &spawn_request.name,
            &spawn_request.parent_path,
            &spawn_request.resource_path,
            spawn_request.transform,
        )?;

        let blueprints =
            get_blueprints(&spawned_node.clone().upcast(), &spawn_request.resource_path);
        spawned_node.queue_free();
        Some(blueprints)
    }
}

/// Recursively returns a vector of NodeBlueprints for all rollback nodes under and including the given node.
fn get_blueprints(root: &Gd<Node>, resource_path: &String) -> Vec<NodeBlueprint> {
    let rigidbodies = get_children_with_class(root, RollbackNodeClass::RollbackRigidBody3D);
    let kin_chars = get_children_with_class(root, RollbackNodeClass::RollbackKinematicCharacter3D);
    let pid_chars = get_children_with_class(root, RollbackNodeClass::RollbackPIDCharacter3D);
    let combined = rigidbodies
        .iter_shared()
        .chain(kin_chars.iter_shared())
        .chain(pid_chars.iter_shared());

    let mut blueprints = Vec::new();
    let root_bp = node_to_blueprint(root, resource_path, true);
    let child_bps = iter_to_blueprints(combined, resource_path, true);

    blueprints.extend(root_bp);
    blueprints.extend(child_bps);
    blueprints
}

/// Recursively returns a vector of NodeBlueprints for all colliders under the given node.
fn get_collider_blueprints(root: &Gd<Node>, resource_path: &String) -> Vec<NodeBlueprint> {
    let colliders = get_children_with_class(root, RollbackNodeClass::RollbackCollisionShape3D);
    iter_to_blueprints(colliders.iter_shared(), resource_path, false)
}

/// Returns a vector of NodeBlueprints for all nodes in the given iterator.
fn iter_to_blueprints(
    iter: impl Iterator<Item = Gd<Node>>,
    resource_path: &String,
    get_child_colliders: bool,
) -> Vec<NodeBlueprint> {
    let mut blueprints = Vec::new();
    for node in iter {
        if let Some(blueprint) = node_to_blueprint(&node, resource_path, get_child_colliders) {
            blueprints.push(blueprint);
        }
    }
    blueprints
}

/// Returns a NodeBlueprint for the given node, without attempting to get child colliders.
fn node_to_blueprint(
    node: &Gd<Node>,
    resource_path: &String,
    get_child_colliders: bool,
) -> Option<NodeBlueprint> {
    let casted = &node.clone().try_cast::<Node3D>().ok()?;
    let class = RollbackNodeClass::try_from_pointer(node, true)?;
    let spawn_isometry = get_spawn_isometry(casted, &class);
    let rapier_builder = get_rapier_builder(casted, &class)?;
    let child_colliders = match get_child_colliders {
        true => get_collider_blueprints(node, resource_path),
        false => Vec::new(),
    };
    Some(NodeBlueprint {
        class,
        snapshottable: true,
        spawn_isometry,
        child_colliders,
        tree_path: node.get_path().to_string(),
        resource_path: resource_path.clone(), // Note: always points to the root spawned node, even for children.
        rapier_builder,
    })
}

pub trait HasBlueprint {
    fn get_blueprint(&self) -> Option<NodeBlueprint>;
    fn set_blueprint(&mut self, blueprint: NodeBlueprint);

    // TODO set blueprint from Gd<Rollback node>.

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
