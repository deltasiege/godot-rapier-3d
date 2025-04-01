use rapier3d::parry::utils::hashmap::HashMap;

pub struct NodeCache {
    pub node_paths: HashMap<u32, String>, // index -> node_path. Node paths cached by us, for use when sending node paths to peers inside actions
    pub node_paths_reverse: HashMap<String, u32>, // node_path -> index. Reverse lookup of node_paths
}

impl NodeCache {
    pub fn new() -> Self {
        Self {
            node_paths: HashMap::default(),
            node_paths_reverse: HashMap::default(),
        }
    }

    /// Adds a node path to the cache and returns the index
    pub fn add_or_get_node_path(&mut self, node_path: String) -> u32 {
        let index = self
            .node_paths_reverse
            .get(&node_path)
            .copied()
            .unwrap_or(self.node_paths.len() as u32);

        self.node_paths.insert(index, node_path.clone());
        self.node_paths_reverse.insert(node_path.clone(), index);
        log::trace!("Cached node path {} to index {}", node_path, index);
        index
    }

    pub fn get_node_path_or_empty(&self, node_cache_idx: u32) -> String {
        let retrieved = self.node_paths.get(&node_cache_idx).cloned();
        match retrieved {
            Some(node_path) => node_path,
            None => {
                log::error!("Failed to retrieve node path for index {}", node_cache_idx);
                String::new()
            }
        }
    }
}
