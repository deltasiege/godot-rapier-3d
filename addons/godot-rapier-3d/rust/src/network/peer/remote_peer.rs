use crate::config::*;
use crate::network::*;
use crate::types::*;

/// A remote peer is a peer in the network that is not us.
pub struct RemotePeer {
    pub metadata: PeerMetadata,
    pub buffers: PeerBuffers,

    pub rtt: u128,                          // Round trip time in milliseconds
    pub last_ping_received: u128,           // Unix millisecond timestamp of the last ping received
    pub time_delta: f64,                    // The difference in time between this peer and us
    pub latest_remote_tick_received: usize, // The latest tick this peer has sent us. Same as received_remote_ticks.keys().max()
    pub latest_local_tick_requested: usize, // The next tick that this peer wants from us. Same as requested_local_ticks.max()
    pub remote_lag: i64,                    // Number of frames this peer is predicting for us
    pub local_lag: i64,                     // Number of frames we are predicting for this peer
    pub calculated_advantage: f64,          // How many ticks this peer is ahead of us
    pub advantage_list: Vec<i64>, // List of advantage values over time to calculate the average
}

impl RemotePeer {
    pub fn new(metadata: PeerMetadata) -> Self {
        Self {
            metadata,
            buffers: PeerBuffers::default(),

            rtt: 0,
            last_ping_received: 0,
            time_delta: 0.0,
            latest_remote_tick_received: 0,
            latest_local_tick_requested: 0,
            remote_lag: 0,
            local_lag: 0,
            calculated_advantage: 0.0,
            advantage_list: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.rtt = 0;
        self.last_ping_received = 0;
        self.time_delta = 0.0;
        self.latest_remote_tick_received = 0;
        self.latest_local_tick_requested = 0;
        self.remote_lag = 0;
        self.local_lag = 0;
        self.calculated_advantage = 0.0;
        self.advantage_list.clear();
    }

    pub fn record_advantage(&mut self, tick: Tick, force_recalculate: bool) {
        self.local_lag = (tick + 1) as i64 - (self.latest_remote_tick_received) as i64;
        self.advantage_list.push(self.local_lag - self.remote_lag);
        if force_recalculate || (self.advantage_list.len() >= TICKS_TO_CALCULATE_ADVANTAGE as usize)
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
}
