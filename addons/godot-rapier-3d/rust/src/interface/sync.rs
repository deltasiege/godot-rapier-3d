use std::collections::HashMap;

use godot::{classes::Engine, prelude::*};

use crate::{
    sync::{
        attach_network_adapter, detach_network_adapter, on_received_remote_start,
        on_received_remote_stop, ping_all_peers, record_all_advantages, record_rtt, return_ping,
        send_local_actions_to_all_peers, sync_start, sync_stop, GR3DNetworkAdapter, Peer,
    },
    world::buffer::Action,
};

use super::get_singleton;

/// Use the GR3DSync singleton to share physics + action data between network clients
#[derive(GodotClass)]
#[class(base = Object)]
pub struct GR3DSync {
    pub host_starting: bool,
    pub started: bool,
    pub tick_interval: f64,     // Time between physics ticks in seconds
    pub tick: usize, // Current tick - updated every physics frame - should match World's timestep
    pub synchronized_tick: u64, // The latest tick that has received all peer actions and matching state hashes
    pub peers: Vec<Peer>,       // List of connected peers
    pub local_actions: HashMap<usize, Vec<Action>>, // Tick -> List of local actions that have been added to local world

    #[export]
    pub network_adapter: Option<Gd<GR3DNetworkAdapter>>,
    base: Base<Object>,
}

#[godot_api]
impl IObject for GR3DSync {
    fn init(base: Base<Object>) -> Self {
        Self {
            host_starting: false,
            started: false,
            tick_interval: 0.0,
            tick: 0,
            synchronized_tick: 0,
            peers: Vec::new(),
            local_actions: HashMap::new(),
            network_adapter: None,
            base,
        }
    }
}

#[godot_api]
impl GR3DSync {
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
    pub fn physics_process(&mut self) {
        if !self.started {
            return;
        }

        record_all_advantages(self, false);

        let singleton = match get_singleton() {
            Some(singleton) => singleton,
            None => {
                log::error!("Failed to get GR3D singleton");
                return;
            }
        };

        let network_adapter = match &self.network_adapter {
            Some(adapter) => adapter,
            None => {
                log::error!("Network adapter not attached");
                return;
            }
        };

        self.tick = singleton.bind().world.state.timestep_id;
        send_local_actions_to_all_peers(&self.peers, &singleton.bind().world, network_adapter);
    }

    #[func]
    pub fn add_peer(&mut self, peer_id: i64) {
        self.peers.push(Peer::new(peer_id));
        log::debug!("Added peer: {}", peer_id);
    }

    #[func]
    pub fn remove_peer(&mut self, peer_id: i64) {
        self.peers.retain(|peer| peer.peer_id != peer_id);
        log::debug!("Removed peer: {}", peer_id);
    }

    #[func]
    pub fn start(&mut self) {
        sync_start(self);
    }

    #[func]
    pub fn stop(&mut self) {
        sync_stop(self);
    }

    fn _on_received_remote_start(&mut self) {
        on_received_remote_start(self);
    }

    fn _on_received_remote_stop(&mut self) {
        on_received_remote_stop(self);
    }

    #[func]
    pub fn _attach_network_adapter(&mut self, network_adapter: Gd<GR3DNetworkAdapter>) {
        attach_network_adapter(self, network_adapter);
    }

    #[func]
    pub fn _detach_network_adapter(&mut self, network_adapter: Gd<GR3DNetworkAdapter>) {
        detach_network_adapter(self, network_adapter);
    }

    #[func]
    pub fn _on_ping_timer_timeout(&mut self) {
        ping_all_peers(self);
    }

    /// Record a copy whenever the local world adds a new action
    pub fn record_local_action(&mut self, tick: usize, action: &Action) {
        if !self.local_actions.contains_key(&tick) {
            self.local_actions.insert(tick, Vec::new());
        }
        self.local_actions
            .get_mut(&tick)
            .unwrap()
            .push(action.clone());
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
    fn _on_received_tick_data(&mut self, sender_peer_id: i64, data: PackedByteArray) {
        godot_print!("Received tick data from peer: {}", sender_peer_id);
    }

    #[func]
    fn get_all_peer_data(&self) -> Array<Dictionary> {
        let mut peer_data = Array::new();
        for peer in &self.peers {
            let mut peer_dict = Dictionary::new();
            peer_dict.set("peer_id", peer.peer_id);
            peer_dict.set("rtt", peer.rtt as i64);
            peer_dict.set("last_ping_received", peer.last_ping_received as i64);
            peer_dict.set("time_delta", peer.time_delta);
            peer_dict.set(
                "last_remote_action_tick_received",
                peer.last_remote_action_tick_received,
            );
            peer_dict.set(
                "next_local_action_tick_requested",
                peer.next_local_action_tick_requested,
            );
            peer_dict.set(
                "last_remote_hash_tick_received",
                peer.last_remote_hash_tick_received,
            );
            peer_dict.set(
                "next_local_hash_tick_requested",
                peer.next_local_hash_tick_requested,
            );
            peer_dict.set("remote_lag", peer.remote_lag);
            peer_dict.set("local_lag", peer.local_lag);
            peer_dict.set("calculated_advantage", peer.calculated_advantage);
            peer_dict.set("advantage_list", peer.advantage_list.to_variant());
            peer_data.push(&peer_dict);
        }
        peer_data
    }
}

const NAME: &str = "GR3DSync";

pub fn register() {
    Engine::singleton().register_singleton(NAME, &GR3DSync::new_alloc());
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

pub fn get_sync_singleton() -> Option<Gd<GR3DSync>> {
    match Engine::singleton().get_singleton(NAME) {
        Some(singleton) => Some(singleton.cast::<GR3DSync>()),
        None => {
            log::error!("Failed to get {} singleton", NAME);
            None
        }
    }
}
