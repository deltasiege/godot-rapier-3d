use godot::classes::Script;
use godot::prelude::*;

use crate::nodes::*;
use crate::types::*;
use crate::utils::*;
use crate::world::SpawnRequest;

pub fn get_spawn_records_from_spawn_request(
    spawn_request: SpawnRequest,
) -> Option<Vec<SpawnRecord>> {
    let mut spawned_node = spawn_into_godot(
        &spawn_request.parent,
        &spawn_request.name,
        &spawn_request.parent.get_path().to_string(),
        &spawn_request.resource_path,
        spawn_request.transform,
    )?;

    let records = get_records(&spawned_node.clone().upcast(), &spawn_request.resource_path);
    spawned_node.queue_free();
    Some(records)
}

/// Recursively returns a vector of NodeBlueprints for all rollback nodes under and including the given node.
fn get_records(root: &Gd<Node>, resource_path: &String) -> Vec<SpawnRecord> {
    let rigidbodies = get_children_with_class(root, RollbackNodeClass::RollbackRigidBody3D);
    let kin_chars = get_children_with_class(root, RollbackNodeClass::RollbackKinematicCharacter3D);
    let pid_chars = get_children_with_class(root, RollbackNodeClass::RollbackPIDCharacter3D);
    let combined = rigidbodies
        .iter_shared()
        .chain(kin_chars.iter_shared())
        .chain(pid_chars.iter_shared());

    let mut records = Vec::new();
    let root_bp = node_to_records(root, resource_path, true);
    let child_bps = iter_to_records(combined, resource_path, true);

    records.extend(root_bp);
    records.extend(child_bps);

    records
}

/// Recursively returns SpawnRecords for all colliders under the given node.
fn get_collider_records(root: &Gd<Node>, resource_path: &String) -> Vec<SpawnRecord> {
    let colliders = get_children_with_class(root, RollbackNodeClass::RollbackCollisionShape3D);
    iter_to_records(colliders.iter_shared(), resource_path, false)
}

/// Returns SpawnRecords for all nodes in the given iterator.
fn iter_to_records(
    iter: impl Iterator<Item = Gd<Node>>,
    resource_path: &String,
    get_child_colliders: bool,
) -> Vec<SpawnRecord> {
    let mut records = Vec::new();
    for node in iter {
        if let Some(blueprint) = node_to_records(&node, resource_path, get_child_colliders) {
            records.push(blueprint);
        }
    }
    records
}

/// Returns a SpawnRecord for the given node, without attempting to get child colliders.
fn node_to_records(
    node: &Gd<Node>,
    resource_path: &String,
    get_child_colliders: bool,
) -> Option<SpawnRecord> {
    let casted = &node.clone().try_cast::<Node3D>().ok()?;
    let class = RollbackNodeClass::try_from_pointer(node, true)?;
    let spawn_isometry = get_spawn_isometry(casted, &class);
    let rapier_builder = get_rapier_builder(casted, &class)?;

    let child_colliders = match get_child_colliders {
        true => get_collider_records(node, resource_path),
        false => Vec::new(),
    }
    .iter()
    .map(|(bp, _)| bp.clone())
    .collect::<Vec<NodeBlueprint>>();

    let rapier_pid_controller = match class {
        RollbackNodeClass::RollbackPIDCharacter3D => {
            let casted = node.clone().cast::<RollbackPIDCharacter3D>();
            let settings = casted.bind().get_controller_settings();
            Some(settings)
        }
        _ => None,
    };

    let rapier_kinematic_controller = match class {
        RollbackNodeClass::RollbackKinematicCharacter3D => {
            let casted = node.clone().cast::<RollbackKinematicCharacter3D>();
            let controller = casted.bind().get_controller();
            Some(controller)
        }
        _ => None,
    };

    let bp = NodeBlueprint {
        class,
        snapshottable: true,
        spawn_isometry,
        child_colliders,
        tree_path: node.get_path().to_string(),
        resource_path: resource_path.clone(),
        rapier_builder,
        rapier_pid_controller,
        rapier_kinematic_controller,
    };

    let script = get_root_script(node);
    Some((bp, script))
}

fn get_root_script(node: &Gd<Node>) -> Option<Gd<Script>> {
    get_node_script(node)
}

fn get_node_script(node: &Gd<Node>) -> Option<Gd<Script>> {
    let variant = node.get_script();
    if variant.is_nil() {
        return None;
    }
    match variant.try_to::<Gd<Script>>() {
        Ok(script) => Some(script),
        Err(e) => {
            log::error!(
                "Failed to get script from node: {}. Error: {}",
                node.get_name(),
                e
            );
            None
        }
    }
}
