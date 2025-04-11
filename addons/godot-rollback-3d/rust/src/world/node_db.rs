use godot::classes::packed_scene;
use godot::prelude::*;
use rapier3d::parry::utils::hashmap::HashMap;

use crate::interface::GR3D;
use crate::nodes::{NodeBlueprint, RollbackNode};
use crate::types::*;

#[derive(Debug)]
pub struct NodeDatabase {
    pub gruid_counter: u32,
    pub nodes: HashMap<GRUID, HashMap<Tick, Option<NodeBlueprint>>>,
}

impl NodeDatabase {
    pub fn new() -> Self {
        Self {
            gruid_counter: 0,
            nodes: HashMap::default(),
        }
    }

    pub fn create_gruid(&self, peer_index: PeerIndex) -> GRUID {
        return (peer_index, self.gruid_counter);
    }

    // TODO pull off Gd<> reference instead
    // pub fn record_spawn(
    //     &mut self,
    //     node: Gd<,
    // ) {
    //     // Insert the node blueprint into the map for the given GRUID and tick
    //     self.nodes
    //         .entry(gruid)
    //         .or_insert_with(HashMap::default)
    //         .insert(tick, Some(node_blueprint));
    // }

    /// Insert the given blueprint at the given GRUID and tick
    fn insert(&mut self, gruid: GRUID, blueprint: Option<NodeBlueprint>, tick: Tick) {
        self.nodes
            .entry(gruid)
            .or_insert_with(HashMap::default)
            .insert(tick, blueprint);
    }

    /// Returns all alive & dead nodes at the given tick.
    pub fn get_nodes(&self, tick: Tick) -> HashMap<GRUID, Option<NodeBlueprint>> {
        let mut latest_bps = HashMap::default();
        for (gruid, bp_map) in &self.nodes {
            let past_keys = bp_map.keys().filter(|&&k| k <= tick);
            if let Some(latest_tick) = past_keys.max().cloned() {
                if let Some(node_blueprint) = bp_map.get(&latest_tick) {
                    latest_bps.insert(*gruid, node_blueprint.clone());
                }
            }
        }
        latest_bps
    }

    /// Overwrite node map with the given nodes at the given tick.
    pub fn set_nodes(&mut self, nodes: HashMap<GRUID, Option<NodeBlueprint>>, tick: Tick) {
        for (gruid, blueprint) in nodes {
            self.insert(gruid, blueprint, tick);
        }
    }

    /// Erase all node entries that were created after the given tick.
    pub fn rollback_to_tick(&mut self, tick: Tick) {
        for (_, bp_map) in &mut self.nodes {
            let keys_to_remove: Vec<_> = bp_map.keys().filter(|&&k| k > tick).cloned().collect();
            for key in keys_to_remove {
                bp_map.swap_remove(&key);
            }
        }

        self.cleanup();
    }

    /// Erases all GRUID entries in the nodes map that are empty.
    /// Note that entries containing despawn markers are preserved!
    fn cleanup(&mut self) {
        let gruids_to_remove = self
            .nodes
            .iter()
            .filter(|(_, bp_map)| bp_map.is_empty())
            .map(|(gruid, _)| *gruid)
            .collect::<Vec<_>>();

        for gruid in gruids_to_remove {
            self.nodes.swap_remove(&gruid);
        }
    }
}

// May not need this! https://docs.godotengine.org/en/stable/classes/class_node.html#class-node-property-scene-file-path

// this should definitely be disabled during rollback so nodes aren't recreated
pub fn spawn_node(
    gr3d: &mut GR3D,
    name: String,
    parent_path: String,
    resource_path: String,
) -> Option<Gd<Node3D>> {
    if !gr3d.network.started {
        log::error!("Cannot spawn node '{}' before network has started", name);
        return None;
    }

    let peer_index = gr3d.network.local_peer.metadata.clone()?.idx?;
    let gruid = gr3d.world.node_db.create_gruid(peer_index);
    let packed_scene = load_scene(&resource_path)?;

    let foo = PackedScene::instantiate(&packed_scene);

    None
}

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
