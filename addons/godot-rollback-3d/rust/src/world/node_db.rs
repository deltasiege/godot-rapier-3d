use godot::classes::{
    BoxShape3D, CapsuleShape3D, ConcavePolygonShape3D, CylinderShape3D, SphereShape3D,
};
use godot::prelude::*;
use rapier3d::parry::utils::hashmap::HashMap;
use rapier3d::prelude::*;

use crate::interface::GR3D;
use crate::nodes::{NodeBlueprint, NodeData, RollbackCollisionShape3D};
use crate::types::*;
use crate::utils::*;

#[derive(Debug)]
pub struct NodeDatabase {
    pub nodes: NodeMap, // Map of all nodes that have been spawned and despawned into Rapier + Godot.

    // Resource cache.
    // Should NOT be saved/loaded via snapshots. Should never be cleared since resources are expected to be static.
    pub resource_cache: HashMap<String, Vec<NodeBlueprint>>, // Cache of resource node paths -> blueprints. Used to avoid instantiating Godot nodes during rollback unnecessarily.

    // Rapier / Godot queues. Should NOT be saved/loaded via snapshots.
    // Instead, whenever a snapshot is loaded, `awaiting_rapier` should be cleared. `awaiting_godot` should be repopulated based on Rapier world state.
    pub awaiting_rapier: HashMap<GRUID, RapierAction>, // Iterated and drained at the end of every **network tick**. Used to ensure Rapier objects are created/removed in deterministic order (sorted by GRUID).
    pub awaiting_godot: HashMap<GRUID, GodotAction>, // Iterated and drained at the end of every **Godot physics_process tick**. Used to ensure Godot matches up with what already exists in Rapier.
}

#[derive(Debug, Clone)]
pub enum RapierAction {
    Spawn(NodeBlueprint),
    Despawn(NodeData),
}

#[derive(Debug, Clone)]
pub enum GodotAction {
    Spawn(NodeData),
    Despawn(NodeData),
}

impl NodeDatabase {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::default(),
            resource_cache: HashMap::default(),
            awaiting_rapier: HashMap::default(),
            awaiting_godot: HashMap::default(),
        }
    }

    /// Returns a new GRUID for the given peer index.
    pub fn create_gruid(&self, peer_index: PeerIndex) -> GRUID {
        let mut free_gen = 0;
        while self.nodes.contains_key(&(peer_index, free_gen)) {
            free_gen += 1;
        }
        (peer_index, free_gen)
    }

    /// Returns all nodes that are alive at the given tick.
    pub fn get_alive_nodes(&self, tick: Tick) -> NodeMap {
        self.nodes
            .iter()
            .filter(|(_, node)| node.is_alive(tick))
            .map(|(gruid, node)| (*gruid, node.clone()))
            .collect()
    }

    // Begins the process of fully spawning a node by adding a NodeBlueprint to the rapier_actions queue.
    // pub fn spawn_node(&mut self, peer_index: PeerIndex, blueprint: NodeBlueprint) {
    //     let gruid = self.create_gruid(peer_index);
    //     self.awaiting_rapier
    //         .insert(gruid, RapierAction::Spawn(blueprint));
    // }
}

// FINALLY PLANNED!
// 1. pub fn spawn_node_in_rapier - add to rapier_acitons quee
// 2. on_rapier_tick - iterate over rapier_actions and add to rapier world and then create NodeData and push to NodeMap, and push to godot_actions queue
// 3. on_physics_process - iterate over godot_actions and spawn node in Godot

// UP TO - spawning! UPTO

pub fn spawn(
    gr3d: &mut GR3D,
    spawner: Gd<Node>,
    name: String,
    parent_path: String,
    resource_path: String,
    transform: Transform3D,
) -> Option<Gd<Node3D>> {
    if !gr3d.network.started {
        log::error!("Cannot spawn node '{}' before network has started", name);
        return None;
    }

    let scene = load_scene(&resource_path)?;
    let blueprints = NodeBlueprint::from_spawn_request(&spawner, &resource_path, &parent_path)?;

    for bp in blueprints {
        godot_print!("AAAAAAAAAAAAAAA bp: {}", bp);
    }

    // UP TO - record blueprints in node_db

    // self.world.node_db.spawn_node(peer_index, blueprint);

    // let peer_index = gr3d.network.local_peer.metadata.clone()?.idx?;
    // let gruid = gr3d.world.node_db.create_gruid(peer_index);
    // let packed_scene = load_scene(&resource_path)?;

    // let foo = PackedScene::instantiate(&packed_scene);

    None
}
