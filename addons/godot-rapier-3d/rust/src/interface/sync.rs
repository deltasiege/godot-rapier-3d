use godot::{classes::Engine, prelude::*};

use crate::sync::{ping_all_peers, record_rtt, return_ping, GR3DNetworkAdapter, Peer};

/// Use the GR3DSync singleton to share physics + input data between network clients
#[derive(GodotClass)]
#[class(base = Object)]
pub struct GR3DSync {
    host_starting: bool,
    started: bool,
    pub input_tick: i64,
    pub peers: Vec<Peer>,

    #[export]
    pub network_adapter: Option<Gd<GR3DNetworkAdapter>>,
    base: Base<Object>,
}

#[godot_api]
impl IObject for GR3DSync {
    fn init(base: Base<Object>) -> Self {
        Self {
            host_starting: false,
            started: false,
            input_tick: 0,
            peers: Vec::new(),
            network_adapter: None,
            base,
        }
    }
}

#[godot_api]
impl GR3DSync {
    #[signal]
    fn sync_started(&self);

    #[signal]
    fn sync_stopped(&self);

    #[signal]
    fn peer_added(&self, peer_id: i64);

    #[signal]
    fn peer_removed(&self, peer_id: i64);

    #[signal]
    fn peer_pinged_back(&self, peer: i64);

    // signal sync_started ()
    // signal sync_stopped ()
    // signal sync_lost ()
    // signal sync_regained ()
    // signal sync_error (msg)

    // signal skip_ticks_flagged (count)
    // signal rollback_flagged (tick)
    // signal prediction_missed (tick, peer_id, local_input, remote_input)
    // signal remote_state_mismatch (tick, peer_id, local_hash, remote_hash)

    // signal peer_added (peer_id)
    // signal peer_removed (peer_id)
    // signal peer_pinged_back (peer)

    // signal state_loaded (_rollback_ticks)
    // signal tick_finished (is_rollback)
    // signal tick_retired (tick)
    // signal tick_input_complete (tick)

    #[func]
    pub fn add_peer(&mut self, peer_id: i64) {
        self.peers.push(Peer::new(peer_id));
        log::debug!("Added peer: {}", peer_id);
    }

    #[func]
    pub fn remove_peer(&mut self, peer_id: i64) {
        self.peers.retain(|peer| peer.peer_id != peer_id);
        log::debug!("Removed peer: {}", peer_id);
    }

    #[func]
    pub fn start(&mut self) {
        match &self.network_adapter {
            Some(adapter) => {
                if !adapter.bind().is_network_host() {
                    log::error!("start() should only be called on the network host");
                    return;
                }

                if self.started || self.host_starting {
                    log::error!("Sync already starting or started");
                    return;
                }

                self.host_starting = true;
                let highest_peer_rtt = self.peers.iter().map(|peer| peer.rtt).max().unwrap_or(0);

                for peer in &self.peers {
                    adapter.bind().send_remote_start(peer.peer_id);
                }

                log::debug!("Delaying host start by {:?}", highest_peer_rtt);
                std::thread::sleep(std::time::Duration::from_millis(highest_peer_rtt as u64)); // TODO this may cause Godot to hang
                self._on_received_remote_start();
                self.host_starting = false;
            }
            None => log::error!("Failed to start sync: no network adapter attached"),
        }
    }

    #[func]
    pub fn stop(&mut self) {
        match &self.network_adapter {
            Some(adapter) => {
                if !adapter.bind().is_network_host() {
                    log::error!("stop() should only be called on the network host");
                    return;
                }

                for peer in &self.peers {
                    adapter.bind().send_remote_stop(peer.peer_id);
                }

                self._on_received_remote_stop();
            }
            None => log::error!("Failed to stop sync: no network adapter attached"),
        }
    }

    fn _on_received_remote_start(&mut self) {
        // _reset()
        // _tick_time = (1.0 / Engine.physics_ticks_per_second)
        self.started = true;
        self.network_adapter
            .as_ref()
            .expect("Network adapter not attached")
            .bind()
            .on_sync_start();
        // _spawn_manager.reset()
        self.base_mut().emit_signal("sync_started", &[]);
    }

    fn _on_received_remote_stop(&mut self) {
        if !(self.started || self.host_starting) {
            return;
        }

        self.started = false;
        self.host_starting = false;

        self.peers.iter_mut().for_each(|peer| {
            peer.clear();
        });

        // _reset()
        self.network_adapter
            .as_ref()
            .expect("Network adapter not attached")
            .bind()
            .on_sync_stop();
        // _spawn_manager.reset()
        self.base_mut().emit_signal("sync_stopped", &[]);
    }

    #[func]
    pub fn _attach_network_adapter(&mut self, mut network_adapter: Gd<GR3DNetworkAdapter>) {
        log::debug!("Attaching network adapter: {:?}", network_adapter);
        network_adapter.bind().on_attached();
        let ping_cb = self.to_gd().callable("_on_received_ping");
        let ping_back_cb = self.to_gd().callable("_on_received_ping_back");
        network_adapter.connect("received_ping", &ping_cb);
        network_adapter.connect("received_ping_back", &ping_back_cb);
        self.network_adapter = Some(network_adapter);
    }

    #[func]
    pub fn _detach_network_adapter(&mut self, network_adapter: Gd<GR3DNetworkAdapter>) {
        log::debug!("Detaching network adapter: {:?}", network_adapter);
        network_adapter.bind().on_detached();
        self.network_adapter = None;
    }

    #[func]
    pub fn _on_ping_timer_timeout(&mut self) {
        ping_all_peers(self);
    }

    #[func]
    fn _on_received_ping(&mut self, peer_id: i64, origin_time: GString) {
        log::debug!("Received ping from peer: {} at {}", peer_id, origin_time);
        return_ping(peer_id, origin_time, &self);
    }

    #[func]
    fn _on_received_ping_back(&mut self, peer_id: i64, origin_ts: String, remote_ts: String) {
        log::debug!("Received ping back from peer: {} at {}", peer_id, remote_ts);
        let local_time = origin_ts.parse().expect("Failed to parse origin_time");
        let remote_time = remote_ts.parse().expect("Failed to parse remote_time");
        record_rtt(self, peer_id, local_time, remote_time);
    }
}

const NAME: &str = "GR3DSync";

pub fn register() {
    Engine::singleton().register_singleton(NAME, &GR3DSync::new_alloc());
}

pub fn unregister() {
    let mut engine = Engine::singleton();
    if let Some(my_singleton) = engine.get_singleton(NAME) {
        engine.unregister_singleton(NAME);
        my_singleton.free();
    } else {
        log::error!("Failed to get {} singleton", NAME);
    }
}
