use godot::{classes::Engine, prelude::*};
use rapier3d::parry::utils::{hashmap::HashMap, hashset::HashSet};

use crate::{actions::Operation, config::MAX_BUFFER_LEN, network::*};

/// Use the GR3DNet singleton to share physics + action data between network clients
#[derive(GodotClass)]
#[class(base = Object)]
pub struct GR3DNet {
    pub tick: usize, // Current tick - updated every physics frame - should match World's timestep
    pub tick_interval: f64, // Time between physics ticks in seconds
    pub started: bool,
    pub host_starting: bool,
    pub peers: Vec<Peer>, // List of connected peers
    pub world_buffer: WorldBuffer,

    // Sync
    pub rollback_flags: Vec<usize>, // Tick -> rollback flag. Used to flag ticks that need to be rolled back to and resimulated. Checked once per physics frame
    pub action_complete_peers: HashMap<usize, HashSet<i64>>, // Tick -> peer_ids. List of peers that have sent actions for this tick. Used to calculate the action_complete_tick
    pub action_complete_tick: usize, // The latest tick that has received all peer actions
    pub physics_hash_complete_peers: HashMap<usize, HashMap<i64, u64>>, // Tick -> (peer_id -> hash). List of peer hashes that have been sent for the given tick. Used to calculate the physics_hash_complete_tick
    pub physics_hash_complete_tick: usize, // The latest tick that has received all peer state hashes
    pub synchronized_tick: usize, // The latest tick that is action complete and physics hashes are complete and equal for all peers

    #[export]
    pub network_adapter: Option<Gd<GR3DNetworkAdapter>>,
    base: Base<Object>,
}

#[godot_api]
impl IObject for GR3DNet {
    fn init(base: Base<Object>) -> Self {
        Self {
            tick: 0,
            tick_interval: 0.0,
            started: false,
            host_starting: false,
            peers: Vec::new(),
            world_buffer: WorldBuffer::new(MAX_BUFFER_LEN),
            rollback_flags: Vec::new(),
            action_complete_peers: HashMap::default(),
            action_complete_tick: 0,
            physics_hash_complete_peers: HashMap::default(),
            physics_hash_complete_tick: 0,
            synchronized_tick: 0,
            network_adapter: None,
            base,
        }
    }
}

#[godot_api]
impl GR3DNet {
    #[signal]
    fn sync_started(&self);

    #[signal]
    fn sync_stopped(&self);

    #[signal]
    fn peer_added(&self, peer_id: i64);

    #[signal]
    fn peer_removed(&self, peer_id: i64);

    #[signal]
    fn peer_pinged_back(&self, peer: i64);

    // signal sync_started ()
    // signal sync_stopped ()
    // signal sync_lost ()
    // signal sync_regained ()
    // signal sync_error (msg)

    // signal skip_ticks_flagged (count)
    // signal rollback_flagged (tick)
    // signal prediction_missed (tick, peer_id, local_input, remote_input)
    // signal remote_state_mismatch (tick, peer_id, local_hash, remote_hash)

    // signal peer_added (peer_id)
    // signal peer_removed (peer_id)
    // signal peer_pinged_back (peer)

    // signal state_loaded (_rollback_ticks)
    // signal tick_finished (is_rollback)
    // signal tick_retired (tick)
    // signal tick_input_complete (tick)

    #[func]
    pub fn on_physics_process(&mut self) {
        if !self.started {
            return;
        }

        record_all_advantages(self, false);
        send_local_actions_to_all_peers(self);

        for peer in &mut self.peers {
            peer.prune_buffers();
        }

        while self.action_complete_peers.len() > MAX_BUFFER_LEN {
            remove_oldest(&mut self.action_complete_peers);
        }

        while self.physics_hash_complete_peers.len() > MAX_BUFFER_LEN {
            remove_oldest(&mut self.physics_hash_complete_peers);
        }
    }

    #[func]
    fn add_peer(&mut self, peer_id: i64) {
        self.peers.push(Peer::new(peer_id));
        log::debug!("Added peer: {}", peer_id);
    }

    #[func]
    fn remove_peer(&mut self, peer_id: i64) {
        self.peers.retain(|peer| peer.peer_id != peer_id);
        log::debug!("Removed peer: {}", peer_id);
    }

    #[func]
    fn start(&mut self) {
        sync_start(self);
    }

    #[func]
    fn stop(&mut self) {
        sync_stop(self);
    }

    #[func]
    fn _on_received_remote_start(&mut self) {
        on_received_remote_start(self);
    }

    #[func]
    fn _on_received_remote_stop(&mut self) {
        on_received_remote_stop(self);
    }

    #[func]
    fn _attach_network_adapter(&mut self, network_adapter: Gd<GR3DNetworkAdapter>) {
        attach_network_adapter(self, network_adapter);
    }

    #[func]
    fn _detach_network_adapter(&mut self, network_adapter: Gd<GR3DNetworkAdapter>) {
        detach_network_adapter(self, network_adapter);
    }

    #[func]
    fn _on_ping_timer_timeout(&mut self) {
        ping_all_peers(self);
    }

    #[func]
    fn _on_received_ping(&mut self, peer_id: i64, origin_time: GString) {
        log::trace!("Received ping from peer: {} at {}", peer_id, origin_time);
        return_ping(peer_id, origin_time, &self);
    }

    #[func]
    fn _on_received_ping_back(&mut self, peer_id: i64, origin_ts: String, remote_ts: String) {
        log::trace!("Received ping back from peer: {} at {}", peer_id, remote_ts);
        let local_time = origin_ts.parse().expect("Failed to parse origin_time");
        let remote_time = remote_ts.parse().expect("Failed to parse remote_time");
        record_rtt(self, peer_id, local_time, remote_time);
    }

    #[func]
    /// Consume a remote action and record it against the peer
    fn _on_received_tick_data(
        &mut self,
        sender_peer_id: i64,
        ser_message: PackedByteArray,
        scene_root: Gd<Node>,
    ) {
        ingest_peer_message(self, sender_peer_id, ser_message, scene_root);
    }

    #[func]
    /// Consume a local action and apply it to the world immediately
    fn _ingest_local_action(&mut self, node: Gd<Node>, operation: Operation, data: Dictionary) {
        ingest_local_action(self, node, operation, data);
    }

    #[func]
    fn _get_all_peer_data(&self) -> Array<Dictionary> {
        get_peer_debug_data(self)
    }

    #[func]
    fn _get_debug_data(&self) -> Dictionary {
        let mut dict = Dictionary::new();
        dict.set("tick", self.tick as i64);
        dict.set("action_complete_tick", self.action_complete_tick as i64);
        dict.set("started", self.started);
        dict.set("host_starting", self.host_starting);
        dict.set("peers", self.peers.len() as i64);
        dict.set("local_actions", self.world_buffer.local_buffer.len() as i64);
        dict
    }
}

const NAME: &str = "GR3DNet";

pub fn register() {
    Engine::singleton().register_singleton(NAME, &GR3DNet::new_alloc());
}

pub fn unregister() {
    let mut engine = Engine::singleton();
    if let Some(my_singleton) = engine.get_singleton(NAME) {
        engine.unregister_singleton(NAME);
        my_singleton.free();
    } else {
        log::error!("Failed to get {} singleton", NAME);
    }
}

pub fn get_net_singleton() -> Option<Gd<GR3DNet>> {
    match Engine::singleton().get_singleton(NAME) {
        Some(singleton) => Some(singleton.cast::<GR3DNet>()),
        None => {
            log::error!("Failed to get {} singleton", NAME);
            None
        }
    }
}
