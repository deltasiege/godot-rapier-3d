use godot::prelude::*;
use rapier3d::parry::utils::hashmap::HashMap;
use serde::{Deserialize, Serialize};

/*

  This module facilitates retrieving Rapier objects via UIDs
  assigned to associated Godot nodes and vice versa

  GString = Godot UID
  (u32, u32) = Rapier handle raw parts

*/

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LookupTable {
    pub godot_to_rapier: HashMap<String, (u32, u32)>,
    pub rapier_to_godot: HashMap<(u32, u32), String>,
    pub godot_to_node_path: HashMap<String, String>,
    pub rapier_to_node_path: HashMap<(u32, u32), String>,
    pub snapshot_colliders: Vec<(u32, u32)>,
}

impl LookupTable {
    pub fn new() -> Self {
        Self {
            godot_to_rapier: HashMap::<String, (u32, u32)>::default(),
            rapier_to_godot: HashMap::<(u32, u32), String>::default(),
            godot_to_node_path: HashMap::<String, String>::default(),
            rapier_to_node_path: HashMap::<(u32, u32), String>::default(),
            snapshot_colliders: Vec::new(),
        }
    }

    pub fn insert(&mut self, godot_uid: GString, rapier_handle: (u32, u32), node_path: NodePath) {
        self.godot_to_rapier
            .insert(godot_uid.to_string(), rapier_handle);
        self.rapier_to_godot
            .insert(rapier_handle, godot_uid.to_string());
        self.godot_to_node_path
            .insert(godot_uid.to_string(), node_path.to_string());
        self.rapier_to_node_path
            .insert(rapier_handle, node_path.to_string());
    }

    // Collision check
    pub fn cuid_exists(&self, cuid: &GString) -> bool {
        let g2r = self.godot_to_rapier.contains_key(cuid.to_string().as_str());
        let rapier_handle = self.godot_to_rapier.get(cuid.to_string().as_str());
        if let Some(rapier_handle) = rapier_handle {
            let r2g = self.rapier_to_godot.contains_key(rapier_handle);
            g2r || r2g
        } else {
            g2r
        }
    }

    pub fn get_rapier_handle(&self, godot_uid: &GString) -> Option<(u32, u32)> {
        self.godot_to_rapier
            .get(godot_uid.to_string().as_str())
            .cloned()
    }

    pub fn get_godot_uid(&self, rapier_handle: &(u32, u32)) -> Option<GString> {
        match self.rapier_to_godot.get(rapier_handle) {
            Some(uid) => Some(GString::from(uid)),
            None => None,
        }
    }

    // TODO - if nodes are reparented at runtime, the node path will be invalid
    // Need to update lookup table whenever the node is reparented
    pub fn get_node_from_handle(
        &self,
        rapier_handle: &(u32, u32),
        root_node: &Gd<Node>,
    ) -> Option<Gd<Node>> {
        if let Some(node_path) = self.rapier_to_node_path.get(rapier_handle) {
            root_node.get_node_or_null(node_path)
        } else {
            None
        }
    }

    pub fn get_node_from_cuid(&self, cuid: &GString, root_node: &Gd<Node>) -> Option<Gd<Node>> {
        if let Some(node_path) = self.godot_to_node_path.get(cuid.to_string().as_str()) {
            root_node.get_node_or_null(node_path)
        } else {
            None
        }
    }

    pub fn remove_by_uid(&mut self, godot_uid: &GString) -> Option<(u32, u32)> {
        if let Some(rapier_handle) = self
            .godot_to_rapier
            .swap_remove(godot_uid.to_string().as_str())
        {
            self.rapier_to_godot.swap_remove(&rapier_handle);
            Some(rapier_handle)
        } else {
            None
        }
    }

    pub fn remove_by_handle(&mut self, rapier_handle: &(u32, u32)) -> Option<GString> {
        if let Some(godot_uid) = self.rapier_to_godot.swap_remove(rapier_handle) {
            self.godot_to_rapier.swap_remove(&godot_uid);
            Some(GString::from(godot_uid))
        } else {
            None
        }
    }

    pub fn insert_snapshot_collider(&mut self, raw_handle: (u32, u32)) {
        self.snapshot_colliders.push(raw_handle);
        self.snapshot_colliders.sort();
    }

    pub fn remove_snapshot_collider(&mut self, raw_handle: &(u32, u32)) {
        self.snapshot_colliders.retain(|&x| x != *raw_handle);
        self.snapshot_colliders.sort();
    }
}
