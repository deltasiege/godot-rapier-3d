use godot::prelude::*;

use crate::interface::GR3D;
use crate::nodes::{rapier_move_node, rapier_teleport_node, NodeBlueprint, NodeData};
use crate::utils::*;
use crate::world::actions::*;

#[derive(Debug, Clone)]

pub enum RapierAction {
    Spawn(NodeBlueprint),
    Despawn(NodeData),
    Modify(NodeData, NodeOperation, Array<Variant>),
}

#[derive(Debug, Clone)]
pub enum GodotAction {
    Spawn(NodeData),
    Despawn(NodeData),
}

/// Iterate over awaiting_rapier queue and add/remove to/from the Rapier world. Update node_db accordingly as well.
pub fn process_rapier_actions(gr3d: &mut GR3D) {
    let mut sorted = gr3d
        .world
        .node_db
        .awaiting_rapier
        .iter()
        .map(|(gruid, action)| (gruid, action))
        .collect::<Vec<_>>();

    sorted.sort_by(|(gruid_a, _), (gruid_b, _)| {
        let (peer_a, gen_a) = gruid_a;
        let (peer_b, gen_b) = gruid_b;
        if peer_a == peer_b {
            gen_a.cmp(gen_b)
        } else {
            peer_a.cmp(peer_b)
        }
    });

    for (gruid, action) in sorted {
        match action {
            RapierAction::Spawn(bp) => {
                let handle = rapier_spawn_from_blueprint(bp.clone(), &mut gr3d.world.physics);
                let node_data = NodeData::new(*gruid, handle, gr3d.world.time.tick, bp.clone());
                gr3d.world.node_db.nodes.insert(*gruid, node_data.clone());
                gr3d.world
                    .node_db
                    .awaiting_godot
                    .insert(*gruid, GodotAction::Spawn(node_data));
            }
            RapierAction::Despawn(node_data) => {
                rapier_despawn_from_node_data(node_data.clone(), &mut gr3d.world.physics);
                gr3d.world.node_db.nodes.swap_remove(&node_data.gruid);
                gr3d.world
                    .node_db
                    .awaiting_godot
                    .insert(*gruid, GodotAction::Despawn(node_data.clone()));
            }
            RapierAction::Modify(node_data, operation, array) => match operation {
                NodeOperation::MoveByAmount => {
                    rapier_move_node(&node_data, array.get(0), &mut gr3d.world.physics);
                }
                NodeOperation::TeleportToPosition => {
                    rapier_teleport_node(&node_data, array.get(0), &mut gr3d.world.physics);
                }
            },
        }
    }

    gr3d.world.node_db.awaiting_rapier.clear();
}

/// Iterate over awaiting_godot queue and add/remove to/from the Godot world. Update node_db accordingly as well.
pub fn process_godot_actions(gr3d: &mut GR3D, runtime: Gd<Node>) {
    for (_, action) in gr3d.world.node_db.awaiting_godot.iter() {
        match action {
            GodotAction::Spawn(node_data) => {
                if let Some(mut spawned_node) = spawn_into_godot(
                    &runtime,
                    &node_data.blueprint.get_node_name(),
                    &node_data.blueprint.get_parent_path(),
                    &node_data.blueprint.resource_path,
                    isometry_to_transform(&node_data.blueprint.spawn_isometry),
                ) {
                    let gruid_str = gruid_to_string(node_data.gruid);
                    if let Some(peer_id) = gr3d.network.get_peer_id(node_data.gruid.0) {
                        spawned_node.set_meta("gruid", &gruid_str.to_variant());
                        spawned_node.set_multiplayer_authority(peer_id as i32);

                        // Call on_network_spawn if it exists
                        if spawned_node.has_method("on_network_spawn") {
                            let mut spawn_data = Dictionary::new();
                            spawn_data.set("gruid", gruid_str);
                            spawn_data.set("peer_id", peer_id);
                            spawn_data.set(
                                "is_local",
                                gr3d.network.local_peer.is_local_gruid(node_data.gruid),
                            );
                            spawned_node
                                .call_deferred("on_network_spawn", &[spawn_data.to_variant()]);
                        }

                        NodeData::set_on_node(node_data, &mut spawned_node);

                        log::trace!(
                            "Spawned Godot node: '{}' under parent: '{}'",
                            node_data.blueprint.get_node_name(),
                            node_data.blueprint.get_parent_path()
                        );
                    }
                }
            }
            GodotAction::Despawn(node_data) => {
                match get_node_by_path(&runtime, &node_data.blueprint.tree_path) {
                    Some(mut node) => {
                        node.queue_free();
                        log::trace!(
                            "Despawned Godot node: '{}' under parent: '{}'",
                            node_data.blueprint.get_node_name(),
                            node_data.blueprint.get_parent_path()
                        );
                    }
                    None => {
                        log::error!(
                            "Cannot despawn missing node at path: '{}'.",
                            node_data.blueprint.tree_path
                        );
                    }
                }
            }
        }
    }

    gr3d.world.node_db.awaiting_godot.clear();
}

#[derive(Debug, Clone, GodotConvert)]
#[godot(via = i64)]
pub enum NodeOperation {
    MoveByAmount,
    TeleportToPosition,
}

impl TryFrom<i64> for NodeOperation {
    type Error = &'static str;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(NodeOperation::MoveByAmount),
            1 => Ok(NodeOperation::TeleportToPosition),
            _ => Err("Invalid NodeOperation index"),
        }
    }
}
