use crate::objects::{Handle, ObjectBridge};
use crate::{LookupIdentifier, Lookups};
#[derive(Debug, Clone)]
pub struct IDBridge {
    pub cuid2: String,        // CUID2
    pub handle_raw: [u32; 2], // Rapier handle raw parts
    pub instance_id: i64,     // Godot node instance_id
}

impl IDBridge {
    pub fn new(cuid2: String, handle_raw: [u32; 2], instance_id: i64) -> Self {
        Self {
            cuid2,
            handle_raw,
            instance_id,
        }
    }

    pub fn invalid() -> IDBridge {
        Self {
            cuid2: String::new(),
            handle_raw: [u32::MAX, u32::MAX],
            instance_id: i64::MAX,
        }
    }

    pub fn is_valid(&self) -> Result<(), String> {
        if self.cuid2.len() == 0 {
            return Err("CUID2 is empty".to_string());
        }
        if self.handle_raw == [u32::MAX, u32::MAX] {
            return Err("Handle is invalid".to_string());
        }
        if self.instance_id == i64::MAX {
            return Err("Instance ID is invalid".to_string());
        }
        Ok(())
    }

    pub fn from_handle(handle: Handle, lookups: &Lookups) -> Self {
        let object_bridge = ObjectBridge::from(handle.kind.clone());
        let found = match lookups.get(
            object_bridge.object_kind,
            LookupIdentifier::Handle,
            &handle.to_string(),
        ) {
            Some(found) => found,
            None => {
                return Self::invalid();
            }
        };

        Self {
            cuid2: found.cuid2.to_string(),
            handle_raw: handle.raw,
            instance_id: found.instance_id,
        }
    }
}
