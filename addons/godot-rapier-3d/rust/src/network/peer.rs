use godot::prelude::*;
use rapier3d::parry::utils::hashmap::HashMap;

use crate::{
    config::{MAX_BUFFER_LEN, TICKS_TO_CALCULATE_ADVANTAGE},
    interface::GR3DNet,
};

use super::remove_oldest;

#[derive(Debug)]
/// A peer is any connected network entity, e.g. player / spectator
pub struct Peer {
    pub peer_id: i64,                            // The unique identifier for this peer
    pub is_spectator: bool,                      // Whether this peer is a spectator
    pub rtt: u128,                               // Round trip time in milliseconds
    pub last_ping_received: u128, // Unix millisecond timestamp of the last ping received
    pub time_delta: f64,          // The difference in time between this peer and us
    pub last_remote_action_tick_received: usize, // The last action tick this peer has sent us
    pub next_local_action_tick_requested: usize, // The next action tick that this peer wants from us
    pub last_remote_hash_tick_received: usize,   // The last hash tick this peer has sent us
    pub next_local_hash_tick_requested: usize,   // The next hash tick that this peer wants from us
    pub remote_lag: i64,                         // Number of frames this peer is predicting for us
    pub local_lag: i64,                          // Number of frames we are predicting for this peer
    pub calculated_advantage: f64,               // How many ticks this peer is ahead of us
    pub advantage_list: Vec<i64>, // List of advantage values over time to calculate the average

    // Messages
    pub last_local_message_size: usize, // The size of the last message we sent to this peer
    pub last_remote_message_size: usize, // The size of the last message we received from this peer

    // Buffers
    pub actions: HashMap<usize, Vec<u8>>, // Tick -> serialized actions. All received remote actions of this peer
    pub state_hashes: HashMap<usize, u64>, // Tick -> state hash. All received remote hashes of this peer
}

impl Peer {
    pub fn new(peer_id: i64) -> Self {
        Self {
            peer_id,
            is_spectator: false,
            rtt: 0,
            last_ping_received: 0,
            time_delta: 0.0,
            last_remote_action_tick_received: 0,
            next_local_action_tick_requested: 1,
            last_remote_hash_tick_received: 0,
            next_local_hash_tick_requested: 1,
            remote_lag: 0,
            local_lag: 0,
            calculated_advantage: 0.0,
            advantage_list: Vec::new(),
            last_local_message_size: 0,
            last_remote_message_size: 0,
            actions: HashMap::default(),
            state_hashes: HashMap::default(),
        }
    }

    pub fn record_advantage(&mut self, tick: usize, force_recalculate: bool) {
        self.local_lag = (tick + 1) as i64 - (self.last_remote_action_tick_received) as i64;
        self.advantage_list.push((self.local_lag - self.remote_lag));
        if force_recalculate || (self.advantage_list.len() >= TICKS_TO_CALCULATE_ADVANTAGE as usize)
        {
            let total: i64 = self.advantage_list.iter().sum();
            self.calculated_advantage = total as f64 / self.advantage_list.len() as f64;
            self.advantage_list.clear();
        }
    }

    pub fn get_actions(&self, tick: usize) -> Option<&Vec<u8>> {
        self.actions.get(&tick)
    }

    pub fn get_state_hash(&self, tick: usize) -> Option<&u64> {
        self.state_hashes.get(&tick)
    }

    pub fn prune_buffers(&mut self) {
        while self.actions.len() > MAX_BUFFER_LEN {
            remove_oldest(&mut self.actions);
        }

        while self.state_hashes.len() > MAX_BUFFER_LEN {
            remove_oldest(&mut self.state_hashes);
        }
    }

    pub fn clear_advantage(&mut self) {
        self.calculated_advantage = 0.0;
        self.advantage_list.clear();
    }

    pub fn clear(&mut self) {
        self.rtt = 0;
        self.last_ping_received = 0;
        self.time_delta = 0.0;
        self.last_remote_action_tick_received = 0;
        self.next_local_action_tick_requested = 0;
        self.last_remote_hash_tick_received = 0;
        self.next_local_hash_tick_requested = 0;
        self.remote_lag = 0;
        self.local_lag = 0;
        self.clear_advantage();
    }
}

pub fn get_peer_debug_data(net: &GR3DNet) -> Array<Dictionary> {
    let mut peer_data = Array::new();
    for peer in net.peers.iter() {
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
        peer_dict.set(
            "last_local_message_size",
            peer.last_local_message_size as i64,
        );
        peer_dict.set(
            "last_remote_message_size",
            peer.last_remote_message_size as i64,
        );
        peer_dict.set("action_buffer_length", peer.actions.len() as i64);
        peer_dict.set("state_hash_buffer_length", peer.state_hashes.len() as i64);
        peer_data.push(&peer_dict);
    }
    peer_data
}
