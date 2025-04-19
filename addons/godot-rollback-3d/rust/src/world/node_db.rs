use godot::classes::Script;
use godot::prelude::*;
use rapier3d::parry::utils::hashmap::HashMap;

use crate::interface::GR3D;
use crate::nodes::NodeData;
use crate::types::*;
use crate::utils::*;
use crate::world::actions::*;

#[derive(Debug)]
pub struct NodeDatabase {
    pub nodes: NodeMap, // Map of all nodes that have been spawned/despawned into Rapier.

    // Static spawn-time data.
    // Should never be cleared since spawned resources are expected to be static.
    // Populated when nodes are spawned via on_spawn_request, at the same time as insertion into awaiting_rapier.

    // TODO change node_scripts to be referenced by resource path cuz they meant to be static and gruid is not static !!!!!!!
    // !!
    pub node_scripts: HashMap<GRUID, Gd<Script>>, // Map of all `on_physics_tick` functions that have been registered for each node.
    pub spawn_cache: HashMap<String, SpawnRecords>, // Cache of resource node paths -> blueprints. Used to avoid instantiating Godot nodes during rollback unnecessarily.

    // Rapier / Godot queues. Should be cleared whenever snapshots are loaded.
    // awaiting_rapier is populated whenever nodes are spawned or modified during Godot physics_process.
    // awaiting_godot is populated whenever nodes are spawned/despawned inside the Rapier world.
    // awaiting_godot must be manually populated whenever non-rollback snapshots are loaded, or rollbacks finish - in order sync Godot nodes with Rapier world.
    pub awaiting_rapier: HashMap<GRUID, RapierAction>, // Iterated and drained at the end of every **network tick**. Used to ensure Rapier objects are created/removed/modified in deterministic order (sorted by GRUID).
    pub awaiting_godot: HashMap<GRUID, GodotAction>, // Iterated and drained at the end of every **Godot physics_process tick**. Used to delay spawning/despawning of Godot nodes until Godot physics_process.
}

impl NodeDatabase {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::default(),
            node_scripts: HashMap::default(),
            spawn_cache: HashMap::default(),
            awaiting_rapier: HashMap::default(),
            awaiting_godot: HashMap::default(),
        }
    }

    /// Returns a new GRUID for the given peer index.
    pub fn create_gruid(&self, peer_index: PeerIndex) -> GRUID {
        let mut free_gen = 0;
        while self.nodes.contains_key(&GRUID::new(peer_index, free_gen)) {
            free_gen += 1;
        }
        GRUID::new(peer_index, free_gen)
    }

    /// Returns all nodes that are alive at the given tick.
    pub fn get_alive_nodes(&self, tick: Tick) -> NodeMap {
        self.nodes
            .iter()
            .filter(|(_, node)| node.is_alive(tick))
            .map(|(gruid, node)| (*gruid, node.clone()))
            .collect()
    }

    /// Inserts a new modify action into the awaiting_rapier queue.
    pub fn ingest_modify_action(
        &mut self,
        node_data: NodeData,
        operation: NodeOperation,
        args: Vec<Variant>,
    ) {
        let gruid = node_data.gruid;
        let action = RapierAction::Modify(node_data, operation, args);
        self.awaiting_rapier.insert(gruid, action);
    }

    /// Fetches blueprints for the given spawn request and inserts them into the awaiting_rapier queue.
    /// Returns a vector of stringified GRUIDs for the nodes that are going to be spawned.
    pub fn on_spawn_request(&mut self, spawn_request: SpawnRequest) -> Option<Array<GString>> {
        let peer_idx = spawn_request.peer_index;
        let spawn_records = self.get_spawn_records(spawn_request)?;

        let mut gruids = Array::new();
        for (blueprint, script) in spawn_records {
            let gruid = self.create_gruid(peer_idx);
            gruids.push(&gruid.to_string());

            self.awaiting_rapier
                .insert(gruid, RapierAction::Spawn(blueprint));

            if let Some(script) = script {
                if script.has_method("on_physics_tick") {
                    self.node_scripts.insert(gruid, script);
                }
            }
        }

        Some(gruids)
    }

    /// Extracts spawn records from the given spawn request,
    /// either from the spawn_cache or by actually instantiating the node in Godot and analyzing it.
    fn get_spawn_records(&mut self, spawn_request: SpawnRequest) -> Option<SpawnRecords> {
        let res_path = spawn_request.resource_path.clone();

        match self.spawn_cache.get(&res_path) {
            Some(spawn_records) => {
                log::trace!("Resource path '{}' found in spawn_cache", res_path);

                // TODO Overriding blueprint properties here probably won't work for a Godot scene containing multiple sibling Rollback nodes.
                // Need to somehow deal with that without making cache useless.
                let mut cached = spawn_records.clone();
                for (bp, _) in cached.iter_mut() {
                    bp.spawn_isometry = transform_to_isometry(spawn_request.transform);
                    bp.tree_path =
                        format!("{}/{}", spawn_request.parent.get_path(), spawn_request.name);
                }

                Some(cached)
            }
            None => {
                log::trace!("Resource path '{}' not found in spawn_cache", res_path);
                let spawn_records = get_spawn_records_from_spawn_request(spawn_request)?;
                self.spawn_cache.insert(res_path, spawn_records.clone());
                Some(spawn_records)
            }
        }
    }

    /// Iterate through sorted node_tick_functions and call them, providing relevant NodeData.
    pub fn process_node_tick_functions(&mut self, local_peer_id: PeerId) {
        log::trace!("Processing {} node tick functions", self.node_scripts.len());

        self.node_scripts.sort_unstable_keys();

        for (gruid, script) in self.node_scripts.iter_mut() {
            if let Some(node_data) = self.nodes.get(gruid) {
                script.call_deferred(
                    "on_physics_tick",
                    &[
                        local_peer_id.to_variant(),
                        node_data.gruid.to_variant(),
                        node_data.get_state_dictionary().to_variant(),
                    ],
                );
            } else {
                log::trace!(
                    "NodeData missing for {} node tick function. Call skipped.",
                    gruid
                );
            }
        }
    }

    /// Overwrites the node map, clears await queues and repopulates the
    /// awaiting_godot queue based on changes to the node map.
    pub fn overwrite_nodes(&mut self, new_nodes: NodeMap) {
        self.awaiting_rapier.clear();
        self.awaiting_godot.clear();

        let diff = hash_map_diff(&self.nodes, &new_nodes);
        let additions = diff.added.len();
        let removals = diff.removed.len();

        for (gruid, node_data) in diff.added {
            self.awaiting_godot
                .insert(gruid, GodotAction::Spawn(node_data));
        }

        for (gruid, node_data) in diff.removed {
            self.awaiting_godot
                .insert(gruid, GodotAction::Despawn(node_data));
        }

        log::trace!("{} nodes added, {} nodes removed.", additions, removals);
        self.nodes = new_nodes;
    }
}

/// Returns the given key's state on a node's NodeData.
pub fn get_state(gr3d: &mut GR3D, gruid: String, key: String) -> Variant {
    match try_get_state(gr3d, gruid, key) {
        Some(result) => result,
        None => Variant::nil(),
    }
}

fn try_get_state(gr3d: &mut GR3D, gruid: String, key: String) -> Option<Variant> {
    let gruid = GRUID::try_from_string(&gruid)?;
    let node_data = match gr3d.world.node_db.nodes.get(&gruid) {
        Some(node) => node,
        None => {
            log::error!(
                "Can't get state ({}). Node {} not found in node_db.",
                key,
                gruid
            );
            return None;
        }
    };

    let value = node_data.node_state.get(&GString::from(key))?;
    Some(value.to_variant())
}

/// Sets the given key=value state on a node's NodeData.
pub fn set_state(gr3d: &mut GR3D, gruid: String, key: String, value: Variant) -> Variant {
    match try_set_state(gr3d, gruid, key, value) {
        Some(result) => result,
        None => Variant::nil(),
    }
}

fn try_set_state(gr3d: &mut GR3D, gruid: String, key: String, value: Variant) -> Option<Variant> {
    let gruid = GRUID::try_from_string(&gruid)?;
    let node_data = match gr3d.world.node_db.nodes.get_mut(&gruid) {
        Some(node) => node,
        None => {
            log::error!(
                "Can't set state ({}). Node {} not found in node_db.",
                key,
                gruid
            );
            return None;
        }
    };

    let previous_value = node_data
        .node_state
        .insert(key.into(), SerdeVar::from_variant(value)?);

    Some(previous_value?.to_variant())
}
