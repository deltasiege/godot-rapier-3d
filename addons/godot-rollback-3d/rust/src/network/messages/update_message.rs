use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::network::{LocalPeer, PeerBuffers, RemotePeer};
use crate::types::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpdateMessage {
    pub tick: Tick,                  // The tick that this message was created on
    pub frames: Vec<UpdateFrame>,    // Frames that the receiving peer has previously asked for
    pub requested_frames: Vec<Tick>, // Frames that the sending peer wants to receive
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpdateFrame {
    pub tick: Tick,          // The tick that this frame refers to
    pub ser_inputs: Vec<u8>, // Serialized inputs for this frame
    pub world_hash: u64,     // Hash of the world state at this tick
}

impl Display for UpdateMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UpdateMessage {{ tick: {}, frames: [", self.tick,)?;
        for frame in &self.frames {
            write!(f, "{}, ", frame)?;
        }
        write!(f, "] requested_frames: {:?} }}", self.requested_frames)
    }
}

impl Display for UpdateFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[tick: {}, ser_inputs.len(): {}, world_hash: {}]",
            self.tick,
            self.ser_inputs.len(),
            self.world_hash
        )
    }
}

impl UpdateMessage {
    pub fn from_peers(
        local_peer: &LocalPeer,
        remote_peer: &RemotePeer,
        current_tick: Tick,
    ) -> Self {
        log::trace!(
            "Creating UpdateMessage for remote peer: {}. requested_updates: {:?}",
            remote_peer.metadata.id,
            remote_peer.requested_frames
        );
        let mut frames_requested_by_sender = Vec::new();

        let earliest_received_tick = remote_peer // The earliest update that the local peer has received from the remote peer
            .received_frames
            .keys()
            .min()
            .cloned()
            .unwrap_or(0);

        for i in earliest_received_tick..=current_tick {
            if !remote_peer.received_frames.contains_key(&i) {
                frames_requested_by_sender.push(i);
            }
        }

        Self::from_buffers(
            &local_peer.buffers,
            current_tick,
            &remote_peer.requested_frames,
            frames_requested_by_sender,
        )
    }

    pub fn from_buffers(
        sender_buffers: &PeerBuffers,
        current_tick: Tick,
        frames_requested_by_receiver: &Vec<Tick>,
        frames_requested_by_sender: Vec<Tick>,
    ) -> Self {
        let mut frames = Vec::new();
        for tick in frames_requested_by_receiver {
            if tick > &current_tick {
                continue;
            }
            if let Some(frame) = UpdateFrame::try_from_buffers(&sender_buffers, tick) {
                frames.push(frame);
            }
        }

        Self {
            tick: current_tick,
            frames,
            requested_frames: frames_requested_by_sender,
        }
    }
}

impl UpdateFrame {
    pub fn try_from_buffers(buffers: &PeerBuffers, tick: &Tick) -> Option<Self> {
        let ser_inputs = match buffers.ser_inputs.get(tick).cloned() {
            Some(ser_inputs) => ser_inputs,
            None => {
                log::error!(
                    "Failed to create UpdateFrame from buffers for tick {} because there were no serialized inputs in given buffers",
                    tick
                );
                return None;
            }
        };

        let world_hash = match buffers.world_hashes.get(tick).cloned() {
            Some(world_hash) => world_hash,
            None => {
                log::error!(
                    "Failed to create UpdateFrame from buffers for tick {} because there was no world hash in given buffers",
                    tick
                );
                return None;
            }
        };

        Some(UpdateFrame {
            tick: *tick,
            ser_inputs,
            world_hash,
        })
    }
}
