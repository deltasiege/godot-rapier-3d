use std::collections::HashMap;

/// A peer is any connected network entity, e.g. player / spectator
pub struct Peer {
    pub peer_id: i64,                          // The unique identifier for this peer
    pub is_spectator: bool,                    // Whether this peer is a spectator
    pub rtt: u128,                             // Round trip time in milliseconds
    pub last_ping_received: u128, // Unix millisecond timestamp of the last ping received
    pub time_delta: f64,          // The difference in time between this peer and us
    pub last_remote_action_tick_received: u64, // The last action tick this peer has sent us
    pub next_local_action_tick_requested: u64, // The next action tick that this peer wants from us
    pub last_remote_hash_tick_received: u64, // The last hash tick this peer has sent us
    pub next_local_hash_tick_requested: u64, // The next hash tick that this peer wants from us
    pub remote_lag: u64,          // Number of frames this peer is predicting for us
    pub local_lag: u64,           // Number of frames we are predicting for this peer
    pub calculated_advantage: f64, // How many ticks this peer is ahead of us
    pub advantage_list: Vec<u32>, // List of advantage values over time to calculate the average

    // Buffers
    pub actions: HashMap<u64, Vec<u8>>, // Tick -> serialized actions. All received remote actions of this peer
    pub state_hashes: HashMap<u64, u64>, // Tick -> state hash. All received remote hashes of this peer
}

const TICKS_TO_CALCULATE_ADVANTAGE: u32 = 60; // TODO expose as project setting

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
            actions: HashMap::new(),
            state_hashes: HashMap::new(),
        }
    }

    pub fn record_advantage(&mut self, tick: u64, force_recalculate: bool) {
        self.local_lag = tick + 1 - self.last_remote_action_tick_received;
        self.advantage_list
            .push((self.local_lag - self.remote_lag) as u32);
        if force_recalculate || (self.advantage_list.len() >= TICKS_TO_CALCULATE_ADVANTAGE as usize)
        {
            let total: u32 = self.advantage_list.iter().sum();
            self.calculated_advantage = total as f64 / self.advantage_list.len() as f64;
            self.advantage_list.clear();
        }
    }

    pub fn get_actions(&self, tick: u64) -> Option<&Vec<u8>> {
        self.actions.get(&tick)
    }

    pub fn get_state_hash(&self, tick: u64) -> Option<&u64> {
        self.state_hashes.get(&tick)
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
