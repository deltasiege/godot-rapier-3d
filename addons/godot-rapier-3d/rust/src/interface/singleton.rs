use godot::classes::{IObject, Object};
use godot::prelude::*;

use crate::adapters::*;
use crate::network::*;
use crate::{Network, World};

/// Public API interface for Godot Rollback 3D.
#[derive(GodotClass)]
#[class(base = Object)]
pub struct GR3D {
    pub world: World,
    pub network: Network,
    base: Base<Object>,
}

#[godot_api]
impl IObject for GR3D {
    fn init(base: Base<Object>) -> Self {
        Self {
            world: World::new(),
            network: Network::new(),
            base,
        }
    }
}

#[godot_api]
impl GR3D {
    #[func]
    fn step(&mut self, count: i64) {
        step(self, count);
    }

    #[func]
    fn _on_physics_process(&mut self) {
        network_physics_process(self);
    }

    // Networking ----------------------------

    #[signal]
    fn sync_started(peer_idx: i64);
    #[signal]
    fn sync_stopped();
    #[signal]
    fn peer_added(peer_id: i64);
    #[signal]
    fn peer_removed(peer_id: i64);
    #[signal]
    fn peer_pinged_back(peer_id: i64);

    #[func]
    fn attach_network_adapter(&mut self, adapter: Gd<GR3DNetworkAdapter>) {
        attach_network_adapter(self, adapter);
    }
    #[func]
    fn detach_network_adapter(&mut self) {
        detach_network_adapter(self);
    }
    #[func]
    fn start_sync(&mut self) {
        let _ = start_sync(self);
    }
    #[func]
    fn stop_sync(&mut self) {
        let _ = stop_sync(self);
    }
    #[func]
    pub fn _received_remote_start(&mut self, peer_idx: i64) {
        let _ = on_received_remote_start(self, peer_idx);
    }
    #[func]
    pub fn _received_remote_stop(&mut self) {
        let _ = on_received_remote_stop(self);
    }
    #[func]
    pub fn _received_ping(&mut self, peer_id: i64, origin_ts: GString) {
        self.network.on_received_ping(peer_id, origin_ts);
    }
    #[func]
    pub fn _received_ping_back(&mut self, peer_id: i64, origin_ts: GString, remote_ts: GString) {
        self.network
            .on_received_ping_back(peer_id, origin_ts, remote_ts);
    }
    #[func]
    pub fn _received_tick_data(&mut self, peer_id: i64, data: PackedByteArray) {
        self.network.on_received_tick_data(peer_id, data);
    }

    // Debugging -----------------------------

    #[func]
    /// Draw lines representing the current state of the world according to Rapier
    fn _get_debug_lines(&mut self) -> Array<Array<Variant>> {
        self.world.debugger.render(&self.world.physics)
    }
}

fn step(gr3d: &mut GR3D, count: i64) {
    for _ in 0..count {
        let tick = gr3d.world.time.tick;
        log::trace!("Executing tick: {}", tick);

        // TODO execute all inputs
        // log::trace!("Applied inputs for tick {}: {:?}", tick, inputs);

        let _resulting_state = gr3d.world.step();

        // TODO add resulting state to local_peer buffer
        log::trace!("Tick finished: {}", tick);
    }
}
