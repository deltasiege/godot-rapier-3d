use godot::prelude::*;

#[derive(GodotClass)]
#[class(base = Node)]
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
    fn received_ping(peer_id: i64, origin_time: GString);

    #[signal]
    fn received_ping_back(peer_id: i64, local_time: GString, remote_time: GString);

    #[signal]
    fn received_remote_start();

    #[signal]
    fn received_remote_stop();

    #[signal]
    fn received_tick_data(peer_id: i64, data: PackedByteArray);

    #[func(virtual)]
    pub fn on_attached(&self) {
        log::debug!("Net adapter '{:?}' attached", self.base().get_name());
    }

    #[func(virtual)]
    pub fn on_detached(&self) {
        log::debug!("Net adapter '{:?}' detached", self.base().get_name());
    }

    #[func(virtual)]
    pub fn on_sync_start(&self) {
        log::debug!("Net adapter '{:?}' sync started", self.base().get_name());
    }

    #[func(virtual)]
    pub fn on_sync_stop(&self) {
        log::debug!("Net adapter '{:?}' sync stopped", self.base().get_name());
    }

    #[func(virtual)]
    pub fn send_ping(&self, _peer_id: i64, _origin_time: GString) {
        log::error!(
            "UNIMPLEMENTED: send_ping on Network adapter: {:?}",
            self.base().get_name()
        );
    }

    #[func(virtual)]
    pub fn send_ping_back(&self, _peer_id: i64, _origin_time: GString, _local_time: GString) {
        log::error!(
            "UNIMPLEMENTED: send_ping_back on Network adapter: {:?}",
            self.base().get_name()
        );
    }

    #[func(virtual)]
    pub fn send_remote_start(&self, _peer_id: i64) {
        log::error!(
            "UNIMPLEMENTED: send_remote_start on Network adapter: {:?}",
            self.base().get_name()
        );
    }

    #[func(virtual)]
    pub fn send_remote_stop(&self, _peer_id: i64) {
        log::error!(
            "UNIMPLEMENTED: send_remote_stop on Network adapter: {:?}",
            self.base().get_name()
        );
    }

    #[func(virtual)]
    pub fn send_tick_data(&self, _peer_id: i64, _data: PackedByteArray) {
        log::error!(
            "UNIMPLEMENTED: send_tick_data on Network adapter: {:?}",
            self.base().get_name()
        );
    }

    #[func(virtual)]
    pub fn is_network_host(&self) -> bool {
        log::error!(
            "UNIMPLEMENTED: is_network_host on Network adapter: {:?}",
            self.base().get_name()
        );
        true
    }

    #[func(virtual)]
    pub fn is_network_authority_for_node(&self, _node: Gd<Node>) -> bool {
        log::error!(
            "UNIMPLEMENTED: is_network_authority_for_node on Network adapter: {:?}",
            self.base().get_name()
        );
        true
    }

    #[func(virtual)]
    pub fn get_unique_id(&self) -> i64 {
        log::error!(
            "UNIMPLEMENTED: get_unique_id on Network adapter: {:?}",
            self.base().get_name()
        );
        1
    }
}
