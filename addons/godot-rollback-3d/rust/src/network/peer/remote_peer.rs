use godot::prelude::*;
use rapier3d::parry::utils::hashmap::HashMap;

use crate::adapters::*;
use crate::config::*;
use crate::network::*;
use crate::types::*;
use crate::utils::get_earliest_entry;

/// A remote peer is a peer in the network that is not us.
#[derive(Debug, Clone)]
pub struct RemotePeer {
    pub metadata: PeerMetadata,
    pub buffers: PeerBuffers,

    pub combined_inputs: HashMap<Tick, (InputMap, bool)>, // Map of all known and predicted inputs we have for this remote peer. Bool represents whether the input map was predicted.
    pub combined_input_hashes: HashMap<Tick, (u64, bool)>, // Map of hashes of this remote peer's combined inputs. Bool represents whether the hash comes from an input map that was predicted.

    pub rtt: UnixEpoch,                // Round trip time in milliseconds
    pub last_ping_received: UnixEpoch, // Unix millisecond timestamp of the last ping received
    pub time_delta: f64,               // The difference in time between this peer and us
    pub remote_lag: i64,               // Number of frames this peer is predicting for us
    pub local_lag: i64,                // Number of frames we are predicting for this peer
    pub calculated_advantage: f64,     // How many ticks this peer is ahead of us
    pub advantage_list: Vec<i64>, // List of advantage values over time to calculate the average

    pub requested_frames: Vec<Tick>, // List of ticks that the remote peer would like to receive local UpdateMessages for
    pub received_frames: HashMap<Tick, UpdateFrame>, // List of update messages we have received from this remote peer
}

impl RemotePeer {
    pub fn new(metadata: PeerMetadata) -> Self {
        Self {
            metadata,
            buffers: PeerBuffers::default(),

            combined_inputs: HashMap::default(),
            combined_input_hashes: HashMap::default(),

            rtt: 0,
            last_ping_received: 0,
            time_delta: 0.0,
            remote_lag: 0,
            local_lag: 0,
            calculated_advantage: 0.0,
            advantage_list: Vec::new(),

            requested_frames: Vec::new(),
            received_frames: HashMap::default(),
        }
    }

    pub fn get_latest_requested_tick(&self) -> Tick {
        self.requested_frames.last().unwrap_or(&0).clone()
    }

    pub fn get_lastest_received_tick(&self) -> Tick {
        self.received_frames.keys().last().unwrap_or(&0).clone()
    }

    // Predict missing inputs up to the current tick after the most recent input
    // Silently returns if there are no inputs to predict from
    pub fn predict_inputs_until_tick(
        &mut self,
        until_tick: Tick,
        input_adapter: &mut Gd<GR3DInputAdapter>,
    ) {
        self.combined_inputs.clear();
        self.combined_input_hashes.clear();

        let mut predictions = 0;

        if let Some((start_tick, start_input)) = get_earliest_entry(&self.buffers.inputs) {
            for tick in start_tick..=until_tick {
                if let Some(input) = self.buffers.inputs.get(&tick) {
                    self.combined_inputs.insert(tick, (input.clone(), false)); // If we have a known input, add it to the combined inputs
                } else {
                    // If we don't have a known input, predict it based on the previous input (predicted or not)
                    let previous_input = match tick == start_tick {
                        true => start_input,
                        false => &self.combined_inputs.get(&(tick - 1)).unwrap().0,
                    };
                    let predicted_input = input_adapter
                        .bind_mut()
                        .get_predicted_inputs(previous_input);

                    self.combined_input_hashes
                        .insert(tick, (get_input_hash(&predicted_input), true));
                    self.combined_inputs.insert(tick, (predicted_input, true));

                    predictions += 1;
                }
            }
        }

        log::trace!(
            "Predicted {}/{} inputs for remote peer: {} until tick: {}",
            predictions,
            self.combined_inputs.len(),
            self.metadata.id,
            until_tick
        );
    }

    pub fn record_update_message(
        &mut self,
        update_message: &UpdateMessage,
        input_adapter: &mut Gd<GR3DInputAdapter>,
    ) {
        log::trace!(
            "Recording received message: {} from remote peer: {}",
            update_message,
            self.metadata.id
        );

        // Record all relevant buffer data
        for frame in &update_message.frames {
            self.received_frames.insert(frame.tick, frame.clone());
            self.buffers.record_update_frame(frame, input_adapter);
        }

        // Remove any requested_frames records that are earlier than the earliest requested tick of the current message
        // (because the remote peer is now confirming that it's no longer interested in them)
        let earliest_requested_tick = update_message
            .requested_frames
            .iter()
            .min()
            .unwrap_or(&0)
            .clone();

        self.requested_frames
            .retain(|tick| tick >= &earliest_requested_tick);

        log::trace!(
            "Remote peer: {:?} is requesting ticks from us: {:?}",
            self.metadata.id,
            update_message.requested_frames
        );

        for tick in &update_message.requested_frames {
            if !self.requested_frames.contains(tick) {
                self.requested_frames.push(*tick);
            }
        }
    }

    pub fn reset(&mut self) {
        self.rtt = 0;
        self.last_ping_received = 0;
        self.time_delta = 0.0;
        self.remote_lag = 0;
        self.local_lag = 0;
        self.calculated_advantage = 0.0;
        self.advantage_list.clear();
    }

    pub fn record_advantage(&mut self, tick: Tick, force_recalculate: bool) {
        self.local_lag = (tick + 1) as i64 - (self.get_lastest_received_tick()) as i64;
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
