use std::hash::{DefaultHasher, Hash, Hasher};

/// Returns the hash of the given byte vector
pub fn get_hash(bytes: &Vec<u8>) -> u64 {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish()
}
