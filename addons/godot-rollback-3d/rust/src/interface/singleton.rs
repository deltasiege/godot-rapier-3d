use godot::classes::{IObject, Object};
use godot::prelude::*;

use crate::adapters::*;
use crate::interface::*;
use crate::network::*;
use crate::nodes::move_by_amount;
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
    fn _on_physics_process(&mut self, runtime: Gd<Node>, step_world: bool) {
        on_physics_process(self, runtime, step_world);
    }

    // Input ---------------------------------

    #[func]
    fn get_input(&mut self, gruid: String, input_key: GString) -> Variant {
        get_input(self, gruid, input_key)
    }

    // Nodes ---------------------------------

    #[func]
    fn get_state(&mut self, gruid: String, key: String) -> Variant {
        get_state(self, gruid, key)
    }

    #[func]
    fn set_state(&mut self, gruid: String, key: String, value: Variant) -> Variant {
        set_state(self, gruid, key, value)
    }

    #[func]
    fn spawn(
        &mut self,
        peer_index: PeerIndex,
        name: String,
        parent: Gd<Node>,
        resource_path: String,
        transform: Transform3D,
    ) -> Array<GString> {
        spawn(self, peer_index, name, parent, resource_path, transform)
    }

    #[func]
    fn move_by_amount(&mut self, gruid: String, amount: Vector3) {
        move_by_amount(self, gruid, amount);
    }

    #[func]
    fn _node_editor_enter(&mut self) {
        // TODO
    }
    #[func]
    fn _node_runtime_enter(&mut self) {
        // TODO
    }
    #[func]
    fn _node_editor_exit(&mut self) {
        // TODO
    }
    #[func]
    fn _node_runtime_exit(&mut self) {
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
    fn get_local_peer_index(&mut self) -> i64 {
        match self.network.local_peer.get_peer_index() {
            Some(index) => index as i64,
            None => -1,
        }
    }
    #[func]
    fn get_peer_index(&mut self, peer_id: i64) -> i64 {
        match self.network.get_peer_index(peer_id) {
            Some(index) => index as i64,
            None => -1,
        }
    }
    #[func]
    fn get_peer_map(&mut self) -> PeerMap {
        match self.network.peer_map {
            Some(ref peer_map) => peer_map.clone(),
            None => {
                log::error!("Peer map is not set. Has sync started yet?");
                PeerMap::new()
            }
        }
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

    #[func]
    fn _get_log_file_dir(&mut self) -> GString {
        crate::interface::logger::get_log_file_dir().into()
    }
}
