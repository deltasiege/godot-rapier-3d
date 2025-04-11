use godot::classes::{IObject, Object};
use godot::prelude::*;

use crate::adapters::*;
use crate::interface::*;
use crate::network::*;
use crate::nodes::RollbackNode;
use crate::types::*;
use crate::utils::try_wrap_bytes;
use crate::world::*;

/// Public API interface for Godot Rollback 3D.
#[derive(GodotClass)]
#[class(base = Object)]
pub struct GR3D {
    pub world: World,
    pub network: Network,
    pub logger: Logger,
    base: Base<Object>,
}

#[godot_api]
impl IObject for GR3D {
    fn init(base: Base<Object>) -> Self {
        Self {
            world: World::new(),
            network: Network::new(),
            logger: Logger::new(),
            base,
        }
    }
}

#[godot_api]
impl GR3D {
    // World ----------------------------------

    #[func]
    fn step(&mut self, count: i64) {
        step(self, count);
    }

    #[func]
    fn save_snapshot(&mut self) -> PackedByteArray {
        try_wrap_bytes(self.world.save_snapshot())
    }
    #[func]
    fn load_snapshot(&mut self, snapshot: PackedByteArray) {
        self.world.load_snapshot(snapshot.to_vec());
    }

    #[func]
    fn _on_physics_process(&mut self, step_world: bool) {
        on_physics_process(self, step_world);
    }

    // Nodes ---------------------------------

    #[func]
    fn spawn(
        &mut self,
        name: String,
        parent_path: String,
        resource_path: String,
    ) -> Option<Gd<Node3D>> {
        spawn_node(self, name, parent_path, resource_path);
        None
    }

    #[func]
    fn _node_editor_enter(&mut self, blueprint: Dictionary) {
        // TODO
    }
    #[func]
    fn _node_runtime_enter(&mut self, blueprint: Dictionary) {
        // TODO
    }
    #[func]
    fn _node_editor_exit(&mut self, blueprint: Dictionary) {
        // TODO
    }
    #[func]
    fn _node_runtime_exit(&mut self, blueprint: Dictionary) {
        // TODO
    }

    // Networking ----------------------------

    #[signal]
    fn sync_started();
    #[signal]
    fn sync_stopped();
    #[signal]
    fn peer_added(peer_id: i64);
    #[signal]
    fn peer_removed(peer_id: i64);
    #[signal]
    fn peer_pinged_back(peer_id: i64);
    #[signal]
    fn peer_map_changed(peer_map: PackedByteArray);

    #[func]
    fn start_sync(&mut self) {
        let _ = start_sync(self);
    }
    #[func]
    fn stop_sync(&mut self) {
        let _ = stop_sync(self);
    }
    #[func]
    fn add_peer(&mut self, peer_id: i64) {
        self.network.add_remote_peer(peer_id);
    }
    #[func]
    fn remove_peer(&mut self, peer_id: i64) {
        self.network.remove_remote_peer(peer_id);
    }

    #[func]
    fn _attach_network_adapter(&mut self, adapter: Gd<GR3DNetworkAdapter>) {
        attach_network_adapter(self, adapter);
    }
    #[func]
    fn _detach_network_adapter(&mut self) {
        detach_network_adapter(self);
    }
    #[func]
    fn _attach_input_adapter(&mut self, adapter: Gd<GR3DInputAdapter>) {
        attach_input_adapter(self, adapter);
    }
    #[func]
    fn _detach_input_adapter(&mut self) {
        detach_input_adapter(self);
    }
    #[func]
    pub fn _received_remote_start(&mut self, peer_map: PeerMap) {
        let _ = on_received_remote_start(self, peer_map);
    }
    #[func]
    pub fn _received_remote_stop(&mut self) {
        let _ = on_received_remote_stop(self);
    }
    #[func]
    pub fn _received_ping(&mut self, peer_id: i64, origin_ts: GString) {
        let _ = return_ping(&self.network, peer_id, origin_ts);
    }
    #[func]
    pub fn _received_ping_back(&mut self, peer_id: i64, origin_ts: GString, remote_ts: GString) {
        let _ = record_rtt(self, peer_id, origin_ts, remote_ts);
    }
    #[func]
    pub fn _received_tick_data(&mut self, peer_id: i64, data: TickData) {
        on_received_tick_data(self, peer_id, data);
    }
    #[func]
    fn _ping_timer_timeout(&mut self) {
        let _ = ping_all_peers(&self.network);
    }

    // Debugging -----------------------------

    #[func]
    /// Returns verbose string containing all debug data
    fn _get_debug_string(&mut self) -> GString {
        get_debug_string(self)
    }

    #[func]
    /// Returns a dictionary containing all debug data
    fn _get_debug_dictionary(&mut self) -> Dictionary {
        get_debug_dictionary(self)
    }

    #[func]
    fn _debug_signals(&mut self) {
        debug_all_signals(self);
    }

    #[func]
    /// Draw lines representing the current state of the world according to Rapier
    fn _get_debug_lines(&mut self) -> Array<Array<Variant>> {
        self.world.debugger.render(&self.world.physics)
    }
}
