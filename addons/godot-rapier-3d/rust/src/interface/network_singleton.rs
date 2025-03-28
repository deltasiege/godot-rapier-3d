use godot::{classes::Engine, prelude::*};

use crate::{actions::Operation, config::MAX_BUFFER_LEN, network::*};

/// Use the GR3DNet singleton to share physics + action data between network clients
#[derive(GodotClass)]
#[class(base = Object)]
pub struct GR3DNet {
    pub tick: usize, // Current tick - updated every physics frame - should match World's timestep
    pub tick_interval: f64, // Time between physics ticks in seconds
    pub synchronized_tick: u64, // The latest tick that has received all peer actions and matching state hashes
    pub started: bool,
    pub host_starting: bool,
    pub peers: Vec<Peer>, // List of connected peers
    pub world_buffer: WorldBuffer,

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
            synchronized_tick: 0,
            started: false,
            host_starting: false,
            peers: Vec::new(),
            world_buffer: WorldBuffer::new(MAX_BUFFER_LEN),
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

        let network_adapter = match &self.network_adapter {
            Some(adapter) => adapter,
            None => {
                log::error!("Network adapter not attached");
                return;
            }
        };

        send_local_actions_to_all_peers(self, network_adapter);

        for peer in &mut self.peers {
            peer.prune_buffers();
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
    fn _on_received_tick_data(&mut self, sender_peer_id: i64, ser_message: PackedByteArray) {
        ingest_peer_message(self, sender_peer_id, ser_message);
    }

    #[func]
    /// Consume a local action and apply it to the world immediately
    fn _ingest_local_action(&mut self, node: Gd<Node>, operation: Operation, data: Dictionary) {
        ingest_local_action(self, node, operation, data);
    }

    #[func]
    fn _get_all_peer_data(&self) -> Array<Dictionary> {
        let mut peer_data = Array::new();
        for peer in &self.peers {
            let mut peer_dict = Dictionary::new();
            peer_dict.set("peer_id", peer.peer_id);
            peer_dict.set("rtt", peer.rtt as i64);
            peer_dict.set("last_ping_received", peer.last_ping_received as i64);
            peer_dict.set("time_delta", peer.time_delta);
            peer_dict.set(
                "last_remote_action_tick_received",
                peer.last_remote_action_tick_received as i64,
            );
            peer_dict.set(
                "next_local_action_tick_requested",
                peer.next_local_action_tick_requested as i64,
            );
            peer_dict.set(
                "last_remote_hash_tick_received",
                peer.last_remote_hash_tick_received as i64,
            );
            peer_dict.set(
                "next_local_hash_tick_requested",
                peer.next_local_hash_tick_requested as i64,
            );
            peer_dict.set("remote_lag", peer.remote_lag);
            peer_dict.set("local_lag", peer.local_lag);
            peer_dict.set("calculated_advantage", peer.calculated_advantage);
            peer_dict.set("advantage_list", peer.advantage_list.to_variant());
            peer_dict.set(
                "ticks_with_actions",
                peer.actions.iter().filter(|(_, v)| !v.is_empty()).count() as i64,
            );
            peer_dict.set("action_buffer_length", peer.actions.len() as i64);
            peer_dict.set("state_hash_buffer_length", peer.state_hashes.len() as i64);
            peer_data.push(&peer_dict);
        }
        peer_data
    }

    #[func]
    fn _get_debug_data(&self) -> Dictionary {
        let mut dict = Dictionary::new();
        dict.set("tick", self.tick as i64);
        dict.set("synchronized_tick", self.synchronized_tick as i64);
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
