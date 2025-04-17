use godot::prelude::*;
use rapier3d::parry::utils::hashmap::HashMap;

use crate::nodes::NodeData;
use crate::types::*;
use crate::utils::*;
use crate::world::actions::*;

#[derive(Debug)]
pub struct NodeDatabase {
    pub nodes: NodeMap, // Map of all nodes that have been spawned/despawned into Rapier.

    // Map of all `on_physics_tick` functions that have been registered for each node.
    // Cannot be serialized into snapshots. Must be cleared and then repopulated after snapshots are loaded.
    pub node_tick_functions: HashMap<GRUID, Callable>,

    // Cache of resource node paths -> blueprints. Used to avoid instantiating Godot nodes during rollback unnecessarily.
    // Should NOT be saved/loaded via snapshots. Should never be cleared since spawned resources are expected to be static.
    pub spawn_cache: HashMap<String, SpawnRecords>,

    // Rapier / Godot queues. Should NOT be saved/loaded via snapshots.
    // Instead, whenever a snapshot is loaded, `awaiting_rapier` should be cleared. `awaiting_godot` should be repopulated based on Rapier world state.
    pub awaiting_rapier: HashMap<GRUID, RapierAction>, // Iterated and drained at the end of every **network tick**. Used to ensure Rapier objects are created/removed in deterministic order (sorted by GRUID).
    pub awaiting_godot: HashMap<GRUID, GodotAction>, // Iterated and drained at the end of every **Godot physics_process tick**. Used to ensure Godot matches up with what already exists in Rapier.
}

impl NodeDatabase {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::default(),
            node_tick_functions: HashMap::default(),
            spawn_cache: HashMap::default(),
            awaiting_rapier: HashMap::default(),
            awaiting_godot: HashMap::default(),
        }
    }

    /// Iterate through sorted node_tick_functions and call them, providing relevant NodeData.
    pub fn process_node_tick_functions(&mut self) {
        self.node_tick_functions.sort_unstable_keys();

        for (gruid, callable) in self.node_tick_functions.iter() {
            if let Some(node_data) = self.nodes.get(gruid) {
                let args = VariantArray::new();
                args.push(node_data.to_variant());
                callable.call(&args);
            } else {
                log::warn!(
                    "NodeData missing for node_tick_function recorded for: {}",
                    gruid
                );
            }
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
        for (blueprint, tick_function) in spawn_records {
            let gruid = self.create_gruid(peer_idx);
            gruids.push(&gruid.to_string());

            self.awaiting_rapier
                .insert(gruid, RapierAction::Spawn(blueprint));

            if let Some(tick_function) = tick_function {
                self.node_tick_functions.insert(gruid, tick_function);
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
}
