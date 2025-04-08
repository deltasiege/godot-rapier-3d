use godot::builtin::GString;
use std::time::{SystemTime, UNIX_EPOCH};

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
