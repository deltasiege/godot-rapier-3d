use godot::{classes::Engine, prelude::*};

use super::GR3DNetworkAdapter;
use crate::interface::GR3DSync;

pub fn sync_start(sync: &mut GR3DSync) {
    match &sync.network_adapter {
        Some(adapter) => {
            if !adapter.bind().is_network_host() {
                log::error!("start() should only be called on the network host");
                return;
            }

            if sync.started || sync.host_starting {
                log::error!("Sync already starting or started");
                return;
            }

            sync.host_starting = true;
            let highest_peer_rtt = sync.peers.iter().map(|peer| peer.rtt).max().unwrap_or(0);

            for peer in &sync.peers {
                adapter.bind().send_remote_start(peer.peer_id);
            }

            log::debug!("Delaying host start by {:?}", highest_peer_rtt);
            std::thread::sleep(std::time::Duration::from_millis(highest_peer_rtt as u64)); // TODO this may cause Godot to hang
            on_received_remote_start(sync);
            sync.host_starting = false;
        }
        None => log::error!("Failed to start sync: no network adapter attached"),
    }
}

pub fn sync_stop(sync: &mut GR3DSync) {
    match &sync.network_adapter {
        Some(adapter) => {
            if !adapter.bind().is_network_host() {
                log::error!("stop() should only be called on the network host");
                return;
            }

            for peer in &sync.peers {
                adapter.bind().send_remote_stop(peer.peer_id);
            }

            on_received_remote_stop(sync);
        }
        None => log::error!("Failed to stop sync: no network adapter attached"),
    }
}

pub fn on_received_remote_start(sync: &mut GR3DSync) {
    // _reset()
    sync.tick_interval = 1.0 / Engine::singleton().get_physics_ticks_per_second() as f64; // TODO set physics engine integration parameters to match this
    sync.started = true;
    sync.network_adapter
        .as_ref()
        .expect("Network adapter not attached")
        .bind()
        .on_sync_start();
    // _spawn_manager.reset()
    sync.base_mut().emit_signal("sync_started", &[]);
}

pub fn on_received_remote_stop(sync: &mut GR3DSync) {
    if !(sync.started || sync.host_starting) {
        return;
    }

    sync.started = false;
    sync.host_starting = false;

    sync.peers.iter_mut().for_each(|peer| {
        peer.clear();
    });

    // _reset()
    sync.network_adapter
        .as_ref()
        .expect("Network adapter not attached")
        .bind()
        .on_sync_stop();
    // _spawn_manager.reset()
    sync.base_mut().emit_signal("sync_stopped", &[]);
}
