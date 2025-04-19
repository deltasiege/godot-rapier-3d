use godot::builtin::GString;
use rapier3d::parry::utils::hashmap::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::types::*;

pub fn get_system_time_ms() -> u128 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_millis(),
        Err(_) => {
            panic!("Invalid system time - before UNIX_EPOCH");
        }
    }
}

pub fn get_system_time_ms_gstr() -> GString {
    get_system_time_ms().to_string().into()
}

/// Returns the entry with the lowest Tick key from the given HashMap.
pub fn get_earliest_entry<T>(map: &HashMap<Tick, T>) -> Option<(Tick, &T)> {
    let mut earliest_entry: Option<(Tick, &T)> = None;
    for (tick, value) in map {
        if earliest_entry.is_none() || tick < &earliest_entry.as_ref().unwrap().0 {
            earliest_entry = Some((*tick, value));
        }
    }
    earliest_entry
}

/// Removes entries from the buffer that are older than the given tick.
pub fn prune_buffer<T>(buffer: &mut HashMap<Tick, T>, tick: Tick) {
    buffer.retain(|&key, _| key >= tick);
}
