use crate::objects::{Handle, HandleKind};
use crate::ObjectKind;
use std::collections::HashMap;
use std::fmt::Debug;

mod id_bridge;
pub use id_bridge::IDBridge;

#[derive(Eq, PartialEq, Debug, Hash)]
pub enum LookupIdentifier {
    ID,
    Handle,
    InstanceID,
}

// Provides means of accessing all currently registered physics objects
pub struct Lookups {
    pub inner: HashMap<ObjectKind, HashMap<LookupIdentifier, HashMap<String, IDBridge>>>,
}

impl Lookups {
    pub fn new() -> Self {
        Self {
            inner: HashMap::from([
                (
                    ObjectKind::RigidBody,
                    HashMap::from([
                        (LookupIdentifier::ID, HashMap::new()), // CUID2 <-> IDBridge
                        (LookupIdentifier::Handle, HashMap::new()), // Rapier handle raw parts <-> IDBridge
                        (LookupIdentifier::InstanceID, HashMap::new()), // Godot instance ID <-> IDBridge
                    ]),
                ),
                (
                    ObjectKind::Collider,
                    HashMap::from([
                        (LookupIdentifier::ID, HashMap::new()), // CUID2 <-> IDBridge
                        (LookupIdentifier::Handle, HashMap::new()), // Rapier handle raw parts <-> IDBridge
                        (LookupIdentifier::InstanceID, HashMap::new()), // Godot instance ID <-> IDBridge
                    ]),
                ),
            ]),
        }
    }

    // Returns true if the given CUID2 is unique across all physics objects
    pub fn is_cuid2_unique(&self, cuid2: &str) -> bool {
        self.inner
            .values()
            .all(|map| map.values().all(|map| !map.contains_key(cuid2)))
    }

    pub fn get(
        &self,
        object_kind: ObjectKind,
        identifier: LookupIdentifier,
        key: &str,
    ) -> Option<&IDBridge> {
        let _object_kind = match object_kind {
            ObjectKind::Character => ObjectKind::RigidBody,
            _ => object_kind,
        };
        self.inner.get(&_object_kind)?.get(&identifier)?.get(key)
    }

    pub fn insert(&mut self, object_kind: ObjectKind, id_bridge: IDBridge) -> Result<(), String> {
        let _object_kind = match object_kind {
            ObjectKind::Character => ObjectKind::RigidBody,
            _ => object_kind,
        };
        let object_map = self.inner.get_mut(&_object_kind).ok_or(format!(
            "Could not find object kind '{}' in lookups",
            _object_kind
        ))?;

        for (ident, map) in object_map.into_iter() {
            match ident {
                LookupIdentifier::ID => {
                    map.insert(id_bridge.clone().cuid2, id_bridge.clone());
                }
                LookupIdentifier::Handle => {
                    let handle = Handle {
                        kind: HandleKind::from(&_object_kind),
                        raw: id_bridge.clone().handle_raw,
                    };
                    map.insert(handle.to_string(), id_bridge.clone());
                }
                LookupIdentifier::InstanceID => {
                    map.insert(id_bridge.clone().instance_id.to_string(), id_bridge.clone());
                }
            }
        }
        Ok(())
    }

    pub fn remove(&mut self, cuid2: String) -> Result<(), String> {
        for (_, object_map) in self.inner.iter_mut() {
            for (ident, map) in object_map.iter_mut() {
                match ident {
                    LookupIdentifier::ID => {
                        map.remove(&cuid2);
                    }
                    _ => map.retain(|_, id_bridge| id_bridge.cuid2 != cuid2),
                }
            }
        }
        Ok(())
    }

    pub fn get_all_cuids(&self) -> HashMap<String, Vec<String>> {
        let mut ret_map = HashMap::new();
        for (object_kind, object_map) in self.inner.iter() {
            let mut cuids = Vec::new();
            let cuid_map = object_map.get(&LookupIdentifier::ID);
            for map in cuid_map.iter() {
                cuids.extend(map.keys().cloned());
            }
            ret_map.insert(object_kind.to_string(), cuids);
        }
        ret_map
    }
}
