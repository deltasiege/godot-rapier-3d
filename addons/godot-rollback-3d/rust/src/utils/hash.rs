use std::hash::{DefaultHasher, Hash, Hasher};

use rapier3d::parry::utils::hashmap::HashMap;

/// Returns the hash of the given value
pub fn get_hash(value: &impl Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

/// Returns hashmap entries that were added or removed between two hashmaps
pub fn hash_map_diff<K, V>(lhs: &HashMap<K, V>, rhs: &HashMap<K, V>) -> HashMapDiff<K, V>
where
    K: Clone + Eq + std::hash::Hash,
    V: Clone,
{
    let mut removed: HashMap<K, V> = HashMap::default();
    for (key, value) in lhs.iter() {
        if !rhs.contains_key(key) {
            removed.insert(key.clone(), value.clone());
        }
    }

    let mut added: HashMap<K, V> = HashMap::default();
    for (key, value) in rhs.iter() {
        if !lhs.contains_key(key) {
            added.insert(key.clone(), value.clone());
        }
    }

    HashMapDiff { added, removed }
}

#[derive(Eq, PartialEq, Debug)]
pub struct HashMapDiff<K, V>
where
    K: Eq + std::hash::Hash,
{
    pub added: HashMap<K, V>,
    pub removed: HashMap<K, V>,
}
