use godot::builtin::GString;
use std::collections::HashMap;
/*

  This module facilitates retrieving Rapier objects via UIDs
  assigned to associated Godot nodes and vice versa

  GString = Godot UID
  (u32, u32) = Rapier handle raw parts

*/

pub struct LookupTable {
    pub godot_to_rapier: HashMap<GString, (u32, u32)>,
    pub rapier_to_godot: HashMap<(u32, u32), GString>,
}

impl LookupTable {
    pub fn new() -> Self {
        Self {
            godot_to_rapier: HashMap::new(),
            rapier_to_godot: HashMap::new(),
        }
    }

    pub fn insert(&mut self, godot_uid: GString, rapier_handle: (u32, u32)) {
        self.godot_to_rapier
            .insert(godot_uid.clone(), rapier_handle);
        self.rapier_to_godot
            .insert(rapier_handle, godot_uid.clone());
    }

    // Collision check
    pub fn cuid_exists(&self, cuid: &GString) -> bool {
        let g2r = self.godot_to_rapier.contains_key(cuid);
        let rapier_handle = self.godot_to_rapier.get(cuid);
        if let Some(rapier_handle) = rapier_handle {
            let r2g = self.rapier_to_godot.contains_key(rapier_handle);
            g2r || r2g
        } else {
            g2r
        }
    }

    pub fn get_rapier_handle(&self, godot_uid: &GString) -> Option<&(u32, u32)> {
        self.godot_to_rapier.get(godot_uid)
    }

    pub fn get_godot_uid(&self, rapier_handle: &(u32, u32)) -> Option<&GString> {
        self.rapier_to_godot.get(rapier_handle)
    }

    pub fn remove_by_uid(&mut self, godot_uid: &GString) -> Option<(u32, u32)> {
        if let Some(rapier_handle) = self.godot_to_rapier.remove(godot_uid) {
            self.rapier_to_godot.remove(&rapier_handle);
            Some(rapier_handle)
        } else {
            None
        }
    }

    pub fn remove_by_handle(&mut self, rapier_handle: &(u32, u32)) -> Option<GString> {
        if let Some(godot_uid) = self.rapier_to_godot.remove(rapier_handle) {
            self.godot_to_rapier.remove(&godot_uid);
            Some(godot_uid)
        } else {
            None
        }
    }
}
