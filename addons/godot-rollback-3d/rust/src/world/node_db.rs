use godot::prelude::*;
use rapier3d::parry::utils::hashmap::HashMap;

use crate::interface::GR3D;
use crate::nodes::NodeBlueprint;
use crate::types::*;

#[derive(Debug)]
pub struct NodeDatabase {
    pub nodes: NodeMap, // Map of all nodes that have been spawned and despawned into Rapier + Godot.

    // Both of these should NOT be saved/loaded via snapshots. Instead, when snapshot is loaded, assess the loaded `nodes` map, then clear and repopulate these maps.
    pub rapier_actions: HashMap<GRUID, NodeAction>, // Iterated and drained at the end of every **network tick**. Used to ensure Rapier objects are created/removed in deterministic order (sorted by GRUID).
    pub godot_actions: HashMap<GRUID, NodeAction>, // Iterated and drained at the end of every **Godot physics_process tick**. Used to ensure Godot matches up with what already exists in Rapier.
}

#[derive(Debug, Clone)]
pub enum NodeAction {
    Spawn(NodeBlueprint),
    Despawn(NodeBlueprint),
}

impl NodeDatabase {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::default(),
            rapier_actions: HashMap::default(),
            godot_actions: HashMap::default(),
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
}

// FINALLY PLANNED!
// 1. pub fn spawn_node_in_rapier - add to rapier_acitons quee
// 2. on_rapier_tick - iterate over rapier_actions and add to rapier world and then create NodeData and push to NodeMap, and push to godot_actions queue
// 3. on_physics_process - iterate over godot_actions and spawn node in Godot

// UP TO - spawning! UPTO

// pub fn spawn_node(
//     gr3d: &mut GR3D,
//     name: String,
//     parent_path: String,
//     resource_path: String,
// ) -> Option<Gd<Node3D>> {
//     if !gr3d.network.started {
//         log::error!("Cannot spawn node '{}' before network has started", name);
//         return None;
//     }

//     let peer_index = gr3d.network.local_peer.metadata.clone()?.idx?;
//     let gruid = gr3d.world.node_db.create_gruid(peer_index);
//     let packed_scene = load_scene(&resource_path)?;

//     let foo = PackedScene::instantiate(&packed_scene);

//     None
// }

fn load_scene(scene_path: &String) -> Option<Gd<PackedScene>> {
    load_path::<PackedScene>(scene_path)
}

fn load_path<T: Inherits<Resource>>(resource_path: &String) -> Option<Gd<PackedScene>> {
    match try_load(resource_path) {
        Ok(packed_scene) => Some(packed_scene),
        Err(err) => {
            log::error!("Failed to load packed scene '{}': {}", resource_path, err);
            None
        }
    }
}
