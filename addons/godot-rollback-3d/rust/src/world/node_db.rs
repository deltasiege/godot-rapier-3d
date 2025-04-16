use godot::prelude::*;
use rapier3d::parry::utils::hashmap::HashMap;

use crate::nodes::{NodeBlueprint, NodeData};
use crate::types::*;
use crate::utils::*;
use crate::world::actions::*;

#[derive(Debug)]
pub struct NodeDatabase {
    pub nodes: NodeMap, // Map of all nodes that have been spawned/despawned into Rapier.

    // Resource cache.
    // Should NOT be saved/loaded via snapshots. Should never be cleared since resources are expected to be static.
    pub resource_cache: HashMap<String, Vec<NodeBlueprint>>, // Cache of resource node paths -> blueprints. Used to avoid instantiating Godot nodes during rollback unnecessarily.

    // Rapier / Godot queues. Should NOT be saved/loaded via snapshots.
    // Instead, whenever a snapshot is loaded, `awaiting_rapier` should be cleared. `awaiting_godot` should be repopulated based on Rapier world state.
    pub awaiting_rapier: HashMap<GRUID, RapierAction>, // Iterated and drained at the end of every **network tick**. Used to ensure Rapier objects are created/removed in deterministic order (sorted by GRUID).
    pub awaiting_godot: HashMap<GRUID, GodotAction>, // Iterated and drained at the end of every **Godot physics_process tick**. Used to ensure Godot matches up with what already exists in Rapier.
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

    /// Inserts a new modify action into the awaiting_rapier queue.
    pub fn ingest_modify_action(
        &mut self,
        ser_node_data: PackedByteArray,
        operation: i64,
        data: Array<Variant>,
    ) {
        self.try_ingest_modify_action(ser_node_data.clone(), operation, data.clone())
            .unwrap_or_else(|| {
                log::error!(
                    "Invalid ingest_modify_action: Node data: {:?}, operation: {}, data: {:?}",
                    ser_node_data,
                    operation,
                    data
                );
            });
    }

    fn try_ingest_modify_action(
        &mut self,
        ser_node_data: PackedByteArray,
        operation: i64,
        data: Array<Variant>,
    ) -> Option<()> {
        let node_data = NodeData::from_packed_byte_array(ser_node_data)?;
        let op = NodeOperation::try_from(operation).ok()?;
        let gruid = node_data.gruid;
        let action = RapierAction::Modify(node_data, op, data);
        self.awaiting_rapier.insert(gruid, action);
        Some(())
    }

    /// Fetches blueprints for the given spawn request and inserts them into the awaiting_rapier queue.
    /// Returns a vector of stringified GRUIDs for the nodes that are going to be spawned.
    pub fn on_spawn_request(&mut self, spawn_request: SpawnRequest) -> Option<Array<GString>> {
        let peer_idx = spawn_request.peer_index;
        let blueprints = self.get_blueprints(spawn_request)?;

        let mut gruids = Array::new();
        for blueprint in blueprints {
            let gruid = self.create_gruid(peer_idx);
            self.awaiting_rapier
                .insert(gruid, RapierAction::Spawn(blueprint));
            gruids.push(&gruid_to_string(gruid));
        }

        Some(gruids)
    }

    /// Returns a vector of NodeBlueprints for the given spawn request.
    /// If the resource path is already in the cache, it returns the cached blueprints.
    fn get_blueprints(&mut self, spawn_request: SpawnRequest) -> Option<Vec<NodeBlueprint>> {
        let res_path = spawn_request.resource_path.clone();
        match self.resource_cache.get(&res_path) {
            Some(blueprints) => {
                log::trace!("Resource path '{}' found in cache", res_path);

                // TODO Overriding blueprint properties here probably won't work for a Godot scene containing multiple sibling Rollback nodes.
                // Need to somehow deal with that without making cache useless.
                let mut bps = blueprints.clone();
                for bp in bps.iter_mut() {
                    bp.spawn_isometry = transform_to_isometry(spawn_request.transform);
                    bp.tree_path =
                        format!("{}/{}", spawn_request.parent.get_path(), spawn_request.name);
                }

                Some(bps)
            }
            None => {
                log::trace!("Resource path '{}' not found in cache", res_path);
                let bps = NodeBlueprint::from_spawn_request(spawn_request)?;
                self.resource_cache.insert(res_path, bps.clone());
                Some(bps)
            }
        }
    }
}
