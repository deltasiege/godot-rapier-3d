/// A peer is any connected network entity - player / spectator / etc

pub struct Peer {
    pub peer_id: i64,
    is_spectator: bool,
    pub rtt: u128,
    pub last_ping_received: u128,
    pub time_delta: f64,
    pub last_remote_input_tick_received: i64,
    next_local_input_tick_requested: i64,
    last_remote_hash_tick_received: i64,
    next_local_hash_tick_requested: i64,
    pub remote_lag: i64,
    pub local_lag: i64,
    calculated_advantage: f64,
    advantage_list: Vec<i64>,
}

impl Peer {
    pub fn new(peer_id: i64) -> Self {
        Self {
            peer_id,
            is_spectator: false,
            rtt: 0,
            last_ping_received: 0,
            time_delta: 0.0,
            last_remote_input_tick_received: 0,
            next_local_input_tick_requested: 1,
            last_remote_hash_tick_received: 0,
            next_local_hash_tick_requested: 1,
            remote_lag: 0,
            local_lag: 0,
            calculated_advantage: 0.0,
            advantage_list: Vec::new(),
        }
    }

    pub fn record_advantage(&mut self, ticks_to_calculate_advantage: i64, force_recalculate: bool) {
        self.advantage_list.push(self.local_lag - self.remote_lag);
        if force_recalculate || (self.advantage_list.len() >= ticks_to_calculate_advantage as usize)
        {
            let total: i64 = self.advantage_list.iter().sum();
            self.calculated_advantage = total as f64 / self.advantage_list.len() as f64;
            self.advantage_list.clear();
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
        self.last_remote_input_tick_received = 0;
        self.next_local_input_tick_requested = 0;
        self.last_remote_hash_tick_received = 0;
        self.next_local_hash_tick_requested = 0;
        self.remote_lag = 0;
        self.local_lag = 0;
        self.clear_advantage();
    }
}
