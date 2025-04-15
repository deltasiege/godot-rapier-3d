use godot::prelude::*;
use rapier3d::parry::utils::hashmap::HashMap;
use rapier3d::prelude::{ColliderHandle, RigidBodyHandle};

use crate::interface::GR3D;
use crate::nodes::{NodeBlueprint, NodeData};
use crate::types::*;
use crate::utils::{
    get_node_by_path, isometry_to_transform, spawn_into_godot, transform_to_isometry,
};
use crate::world::PhysicsState;

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

/// Iterate over awaiting_godot queue and add/remove to/from the Godot world. Update node_db accordingly as well.
pub fn process_godot_spawns_despawns(gr3d: &mut GR3D, runtime: Gd<Node>) {
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
                    let peer_id = gr3d.network.get_peer_id(node_data.gruid.0);
                    spawned_node.set_meta("gruid", &gruid_str.to_variant());
                    spawned_node.set_multiplayer_authority(peer_id as i32);

                    if spawned_node.has_method("on_network_spawn") {
                        let mut spawn_data = Dictionary::new();
                        spawn_data.set("gruid", gruid_str);
                        spawn_data.set("peer_id", peer_id);
                        spawn_data.set("is_local", gr3d.network.is_local(node_data.gruid));
                        spawned_node.call_deferred("on_network_spawn", &[spawn_data.to_variant()]);
                    }

                    log::trace!(
                        "Spawned Godot node: '{}' under parent: '{}'",
                        node_data.blueprint.get_node_name(),
                        node_data.blueprint.get_parent_path()
                    );
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

/// Iterate over awaiting_rapier queue and add/remove to/from the Rapier world. Update node_db accordingly as well.
pub fn process_rapier_spawns_despawns(gr3d: &mut GR3D) {
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
        }
    }

    gr3d.world.node_db.awaiting_rapier.clear();
}

/// Creates appropriate Rapier object from the given blueprint and adds it to the Rapier world.
/// Returns the created RapierHandle
fn rapier_spawn_from_blueprint(
    blueprint: NodeBlueprint,
    physics: &mut PhysicsState,
) -> RapierHandle {
    let node_name = blueprint.get_node_name();
    match blueprint.rapier_builder {
        RapierBuilder::RigidBody(rb) => {
            let handle = physics.bodies.insert(rb);
            let num_child_colliders = blueprint.child_colliders.len();

            for collider in blueprint.child_colliders {
                match collider.rapier_builder {
                    RapierBuilder::Collider(collider) => {
                        physics
                            .colliders
                            .insert_with_parent(collider, handle, &mut physics.bodies);
                    }
                    _ => {
                        log::error!(
                            "Child collider {} does not have a collider builder",
                            collider
                        );
                    }
                }
            }

            let raw_parts = handle.into_raw_parts();

            log::trace!(
                "Spawned Rapier RigidBody: '{}' {:?} with {} child colliders",
                node_name,
                raw_parts,
                num_child_colliders
            );

            raw_parts
        }
        RapierBuilder::Collider(collider) => {
            let handle = physics.colliders.insert(collider);
            let raw_parts = handle.into_raw_parts();
            log::trace!("Spawned Rapier Collider: '{}' {:?}", node_name, raw_parts);
            raw_parts
        }
    }
}

/// Removes the rapier_handle specified in given node_data from the Rapier world.
fn rapier_despawn_from_node_data(node_data: NodeData, physics: &mut PhysicsState) {
    match node_data.blueprint.rapier_builder {
        RapierBuilder::RigidBody(_) => {
            let handle = RigidBodyHandle::from_raw_parts(
                node_data.rapier_handle.0,
                node_data.rapier_handle.1,
            );
            physics.bodies.remove(
                handle,
                &mut physics.islands,
                &mut physics.colliders,
                &mut physics.impulse_joints,
                &mut physics.multibody_joints,
                true,
            );
        }
        RapierBuilder::Collider(_) => {
            let handle = ColliderHandle::from_raw_parts(
                node_data.rapier_handle.0,
                node_data.rapier_handle.1,
            );
            physics
                .colliders
                .remove(handle, &mut physics.islands, &mut physics.bodies, false);
        }
    }
}

/// Begins spawn process and returns stringified GRUIDs of the nodes that will be spawned.
pub fn spawn(
    gr3d: &mut GR3D,
    peer_index: PeerIndex,
    name: String,
    parent: Gd<Node>,
    resource_path: String,
    transform: Transform3D,
) -> Array<GString> {
    let spawn_request = SpawnRequest {
        peer_index,
        name,
        parent,
        resource_path,
        transform,
    };
    match try_spawn(gr3d, spawn_request) {
        Some(gruid_strings) => gruid_strings,
        None => Array::new(),
    }
}

/// Option compatible version of spawn function.
fn try_spawn(gr3d: &mut GR3D, spawn_request: SpawnRequest) -> Option<Array<GString>> {
    if !gr3d.network.started {
        log::error!(
            "Cannot spawn node '{}' before network has started",
            spawn_request.name
        );
        return None;
    }

    if spawn_request.peer_index == 0 {
        log::error!(
            "Cannot spawn node '{}' under ambient peer (0). peer_index must be 1 or greater.",
            spawn_request.name
        );
        return None;
    }

    let gruids = gr3d.world.node_db.on_spawn_request(spawn_request)?;

    Some(gruids)
}

/// Collection of arguments needed to spawn a node.
pub struct SpawnRequest {
    pub peer_index: PeerIndex,
    pub name: String,
    pub parent: Gd<Node>,
    pub resource_path: String,
    pub transform: Transform3D,
}
