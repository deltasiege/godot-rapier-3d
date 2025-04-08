use godot::prelude::*;

use crate::interface::{disconnect_incoming_signals, disconnect_outgoing_signals, GR3D};

#[derive(GodotClass)]
#[class(base = Node)]
/// Base class that can be overriden by GDScript
/// Responsible for sending & receiving network messages between peers
/// via whatever transport the user desires.
pub struct GR3DNetworkAdapter {
    base: Base<Node>,
}

#[godot_api]
impl INode for GR3DNetworkAdapter {
    fn init(base: Base<Node>) -> Self {
        Self { base }
    }
}

#[godot_api]
impl GR3DNetworkAdapter {
    #[signal]
    pub fn received_ping(peer_id: i64, origin_ts: GString);
    #[signal]
    pub fn received_ping_back(peer_id: i64, origin_ts: GString, remote_ts: GString);
    #[signal]
    pub fn received_remote_start(peer_map: PackedByteArray);
    #[signal]
    pub fn received_remote_stop();
    #[signal]
    pub fn received_tick_data(peer_id: i64, data: PackedByteArray);

    #[func(virtual)]
    pub fn on_attached(&self) {
        debug_msg("attached", self);
    }

    #[func(virtual)]
    pub fn on_detached(&self) {
        debug_msg("detached", self);
    }

    #[func(virtual)]
    pub fn on_sync_start(&self) {
        debug_msg("sync started", self);
    }

    #[func(virtual)]
    pub fn on_sync_stop(&self) {
        debug_msg("sync stopped", self);
    }

    #[func(virtual)]
    pub fn send_ping(&self, _peer_id: i64, _origin_ts: GString) {
        unimpl("send_ping", self);
    }

    #[func(virtual)]
    pub fn send_ping_back(&self, _peer_id: i64, _origin_ts: GString, _local_ts: GString) {
        unimpl("send_ping_back", self);
    }

    #[func(virtual)]
    pub fn send_remote_start(&self, _peer_id: i64, _peer_map: PackedByteArray) {
        unimpl("send_remote_start", self);
    }

    #[func(virtual)]
    pub fn send_remote_stop(&self, _peer_id: i64) {
        unimpl("send_remote_stop", self);
    }

    #[func(virtual)]
    pub fn send_tick_data(&self, _peer_id: i64, _data: PackedByteArray) {
        unimpl("send_tick_data", self);
    }

    #[func(virtual)]
    pub fn is_network_host(&self) -> bool {
        unimpl("is_network_host", self);
        true
    }

    #[func(virtual)]
    pub fn is_network_authority_for_node(&self, _node: Gd<Node>) -> bool {
        unimpl("is_network_authority_for_node", self);
        true
    }

    #[func(virtual)]
    pub fn get_unique_id(&self) -> i64 {
        unimpl("get_unique_id", self);
        1
    }
}

/// Attach a network adapter to the GR3D instance and connect all signals to the GR3D singleton.
pub fn attach_network_adapter(gr3d: &mut GR3D, mut adapter: Gd<GR3DNetworkAdapter>) {
    log::debug!("Attaching NetworkAdapter: {:?}", adapter);
    adapter.bind().on_attached();

    let received_ping_cb = gr3d.base().callable("received_ping");
    adapter.connect("received_ping", &received_ping_cb);

    let received_ping_back_cb = gr3d.base().callable("received_ping_back");
    adapter.connect("received_ping_back", &received_ping_back_cb);

    let received_remote_start_cb = gr3d.base().callable("received_remote_start");
    adapter.connect("received_remote_start", &received_remote_start_cb);

    let received_remote_stop_cb = gr3d.base().callable("received_remote_stop");
    adapter.connect("received_remote_stop", &received_remote_stop_cb);

    let received_tick_data_cb = gr3d.base().callable("received_tick_data");
    adapter.connect("received_tick_data", &received_tick_data_cb);

    // Requires newer version - uncomment when godot-rust 0.2.5 is released
    // adapter
    //     .signals()
    //     .received_ping()
    //     .connect_obj(&gr3d.to_gd(), GR3D::_received_ping);
    // adapter
    //     .signals()
    //     .received_ping_back()
    //     .connect_obj(&gr3d.to_gd(), GR3D::_received_ping_back);
    // adapter
    //     .signals()
    //     .received_remote_start()
    //     .connect_obj(&gr3d.to_gd(), GR3D::_received_remote_start);
    // adapter
    //     .signals()
    //     .received_remote_stop()
    //     .connect_obj(&gr3d.to_gd(), GR3D::_received_remote_stop);
    // adapter
    //     .signals()
    //     .received_tick_data()
    //     .connect_obj(&gr3d.to_gd(), GR3D::_received_tick_data);

    gr3d.network.adapter = Some(adapter);
}

/// Detach the network adapter and disconnect all signals from the GR3D singleton.
pub fn detach_network_adapter(gr3d: &mut GR3D) {
    log::debug!("Detaching NetworkAdapter: {:?}", gr3d.network.adapter);
    if let Some(adapter) = &mut gr3d.network.adapter {
        adapter.bind().on_detached();
        disconnect_incoming_signals(adapter);
        disconnect_outgoing_signals(adapter);
    }
    gr3d.network.adapter = None;
}

fn debug_msg(msg: &str, adapter: &GR3DNetworkAdapter) {
    log::debug!("NetworkAdapter '{:?}' {}", adapter.base().get_name(), msg);
}

fn unimpl(func_name: &str, adapter: &GR3DNetworkAdapter) {
    log::error!(
        "UNIMPLEMENTED: {} on NetworkAdapter: {:?}",
        func_name,
        adapter.base().get_name()
    );
}
