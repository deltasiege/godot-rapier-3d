use godot::prelude::*;

// Pulls a value from a dictionary and logs an error if it fails
pub fn extract_from_dict<T>(dict: &Dictionary, key: &str, log_error: bool) -> Option<T>
where
    T: FromGodot,
{
    match dict.get(key) {
        Some(value) => match value.try_to() {
            Ok(value) => Some(value),
            Err(e) => {
                if log_error {
                    log::error!("Failed to extract '{}' from dictionary: {:?}", key, e);
                }
                None
            }
        },
        None => {
            if log_error {
                log::error!("Missing '{}' in dictionary", key);
            }
            None
        }
    }
}
