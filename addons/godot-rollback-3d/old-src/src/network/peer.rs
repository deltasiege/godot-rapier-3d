use std::{
    fmt::{Display, Formatter},
    usize,
};

use godot::prelude::*;
use rapier3d::parry::utils::hashmap::HashMap;
use serde::{Deserialize, Serialize};

use crate::{
    config::{MAX_BUFFER_LEN, TICKS_TO_CALCULATE_ADVANTAGE},
    interface::GR3DNet,
    utils::get_hash,
};

use super::remove_oldest;

#[derive(Debug)]
/// A peer is any connected network entity, e.g. player / spectator
pub struct Peer {
    pub peer_id: i64,                       // The unique identifier for this peer
    pub is_spectator: bool,                 // Whether this peer is a spectator
    pub rtt: u128,                          // Round trip time in milliseconds
    pub last_ping_received: u128,           // Unix millisecond timestamp of the last ping received
    pub time_delta: f64,                    // The difference in time between this peer and us
    pub latest_remote_tick_received: usize, // The latest tick this peer has sent us. Same as received_remote_ticks.keys().max()
    pub latest_local_tick_requested: usize, // The next tick that this peer wants from us. Same as requested_local_ticks.max()
    pub remote_lag: i64,                    // Number of frames this peer is predicting for us
    pub local_lag: i64,                     // Number of frames we are predicting for this peer
    pub calculated_advantage: f64,          // How many ticks this peer is ahead of us
    pub advantage_list: Vec<i64>, // List of advantage values over time to calculate the average

    // Buffers
    pub requested_local_ticks: Vec<usize>, // Ticks that this peer has requested from us
    pub received_remote_ticks: HashMap<usize, PeerBufferFrame>, // tick -> PeerBufferFrame. All action + state_hash data we have received from this peer. The keys also represent acknowledged ticks that don't need to be resent by this peer
    pub received_remote_action_hashes: HashMap<usize, u64>, // tick -> u64. Contains the hash of the actions at each tick
}

impl Peer {
    pub fn new(peer_id: i64) -> Self {
        Self {
            peer_id,
            is_spectator: false,
            rtt: 0,
            last_ping_received: 0,
            time_delta: 0.0,
            latest_remote_tick_received: 0,
            latest_local_tick_requested: 0,
            remote_lag: 0,
            local_lag: 0,
            calculated_advantage: 0.0,
            advantage_list: Vec::new(),
            requested_local_ticks: Vec::new(),
            received_remote_ticks: HashMap::default(),
            received_remote_action_hashes: HashMap::default(),
        }
    }

    pub fn record_advantage(&mut self, tick: usize, force_recalculate: bool) {
        self.local_lag = (tick + 1) as i64 - (self.latest_remote_tick_received) as i64;
        self.advantage_list.push(self.local_lag - self.remote_lag);
        if force_recalculate || (self.advantage_list.len() >= TICKS_TO_CALCULATE_ADVANTAGE as usize)
        {
            let total: i64 = self.advantage_list.iter().sum();
            self.calculated_advantage = total as f64 / self.advantage_list.len() as f64;
            self.advantage_list.clear();
        }
    }

    pub fn record_received_tick(&mut self, tick: usize, frame: PeerBufferFrame) {
        self.received_remote_action_hashes
            .insert(tick, get_hash(&frame.ser_actions));
        self.received_remote_ticks.insert(tick, frame);
    }

    /// Returns the list of ticks that we need from this peer based on missing keys in the received_remote_ticks
    pub fn get_ticks_to_request_and_beyond(&self, end_tick: usize) -> Vec<usize> {
        let mut ticks = Vec::new();
        let start = self
            .received_remote_ticks
            .keys()
            .min()
            .cloned()
            .unwrap_or(0);
        for i in start..=end_tick {
            if i > (self.latest_remote_tick_received) {
                ticks.push(i);
            } else if !self.received_remote_ticks.contains_key(&i) {
                ticks.push(i);
            }
        }
        ticks
    }

    /// Returns the ticks that this tick has requested from us + all ticks beyond that up to the given end_tick
    pub fn get_requested_local_ticks_and_beyond(&self, end_tick: usize) -> Vec<usize> {
        if self.latest_local_tick_requested > end_tick {
            return self.requested_local_ticks.clone();
        }

        let mut ticks = self.requested_local_ticks.clone();
        for i in (self.latest_local_tick_requested)..=end_tick {
            ticks.push(i);
        }
        ticks
    }

    pub fn prune_buffers(&mut self) {
        while self.received_remote_ticks.len() > MAX_BUFFER_LEN {
            remove_oldest(&mut self.received_remote_ticks);
        }

        while self.received_remote_action_hashes.len() > MAX_BUFFER_LEN {
            remove_oldest(&mut self.received_remote_action_hashes);
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
        self.latest_remote_tick_received = 0;
        self.latest_local_tick_requested = 0;
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
            "latest_remote_tick_received",
            peer.latest_remote_tick_received as i64,
        );
        peer_dict.set(
            "latest_local_tick_requested",
            peer.latest_local_tick_requested as i64,
        );
        peer_dict.set("remote_lag", peer.remote_lag);
        peer_dict.set("local_lag", peer.local_lag);
        peer_dict.set("calculated_advantage", peer.calculated_advantage);
        peer_dict.set("advantage_list", peer.advantage_list.to_variant());
        peer_dict.set(
            "requested_local_ticks",
            peer.requested_local_ticks
                .iter()
                .map(|t| (*t as i64).to_variant())
                .collect::<Array<Variant>>(),
        );
        peer_dict.set(
            "received_remote_ticks",
            peer.received_remote_ticks
                .keys()
                .map(|k| (*k as i64).to_variant())
                .collect::<Array<Variant>>(),
        );
        peer_dict.set(
            "ticks_with_actions",
            peer.received_remote_ticks
                .iter()
                .filter(|(_, v)| !v.ser_actions.is_empty())
                .count() as i64,
        );
        peer_dict.set(
            "ticks_with_state_hashes",
            peer.received_remote_ticks
                .iter()
                .filter(|(_, v)| v.state_hash.is_some())
                .count() as i64,
        );
        peer_data.push(&peer_dict);
    }
    peer_data
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PeerBufferFrame {
    pub tick: usize,
    pub ser_actions: Vec<u8>,
    pub state_hash: Option<u64>,
}

impl PeerBufferFrame {
    pub fn new(tick: usize, ser_actions: Vec<u8>, state_hash: Option<u64>) -> Self {
        Self {
            tick,
            ser_actions,
            state_hash,
        }
    }
}

impl Display for PeerBufferFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PeerBufferFrame {{ ser_actions.len(): {}, state_hash: {:?} }}",
            self.ser_actions.len(),
            self.state_hash
        )
    }
}
