use godot::builtin::GString;
use rapier3d::parry::utils::hashmap::HashMap;
use serde::{Deserialize, Serialize};
// use std::collections::HashMap;
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
    pub snapshot_colliders: Vec<(u32, u32)>,
}

impl LookupTable {
    pub fn new() -> Self {
        Self {
            godot_to_rapier: HashMap::<String, (u32, u32)>::default(),
            rapier_to_godot: HashMap::<(u32, u32), String>::default(),
            snapshot_colliders: Vec::new(),
        }
    }

    pub fn insert(&mut self, godot_uid: GString, rapier_handle: (u32, u32)) {
        self.godot_to_rapier
            .insert(godot_uid.to_string(), rapier_handle);
        self.rapier_to_godot
            .insert(rapier_handle, godot_uid.to_string());
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

    pub fn get_rapier_handle(&self, godot_uid: &GString) -> Option<&(u32, u32)> {
        self.godot_to_rapier.get(godot_uid.to_string().as_str())
    }

    pub fn get_godot_uid(&self, rapier_handle: &(u32, u32)) -> Option<GString> {
        match self.rapier_to_godot.get(rapier_handle) {
            Some(uid) => Some(GString::from(uid)),
            None => None,
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
