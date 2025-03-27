use crate::{
    interface::GR3DSync,
    utils::{get_system_time_ms, get_system_time_ms_gstr},
};
use godot::prelude::*;

pub fn ping_all_peers(sync: &GR3DSync) {
    let peers = &sync.peers;
    let adapter = sync
        .network_adapter
        .as_ref()
        .expect("Network adapter not attached")
        .bind();

    if peers.is_empty() {
        return;
    }

    let local_time: GString = get_system_time_ms_gstr();

    for peer in peers {
        if peer.peer_id == adapter.get_unique_id() {
            log::error!("Cannot ping ourselves");
            continue;
        }

        adapter.send_ping(peer.peer_id, local_time.clone());
    }
}

pub fn return_ping(peer_id: i64, origin_time: GString, sync: &GR3DSync) {
    let adapter = sync
        .network_adapter
        .as_ref()
        .expect("Network adapter not attached")
        .bind();

    if peer_id == adapter.get_unique_id() {
        log::error!("Cannot ping back ourselves");
        return;
    }

    let local_time = get_system_time_ms_gstr();
    adapter.send_ping_back(peer_id, origin_time, local_time);
}

pub fn record_rtt(sync: &mut GR3DSync, peer_id: i64, local_time: u128, remote_time: u128) {
    let system_time = get_system_time_ms();

    let peer = sync
        .peers
        .iter_mut()
        .find(|peer| peer.peer_id == peer_id)
        .expect("Peer not found");

    peer.last_ping_received = system_time;
    peer.rtt = system_time - remote_time;

    let rf = remote_time as f64;
    let lf = local_time as f64;
    let rtt = peer.rtt as f64;
    peer.time_delta = rf - lf - (rtt / 2.0);
    sync.base_mut()
        .emit_signal("peer_pinged_back", &[Variant::from(peer_id)]);
}

pub fn record_all_advantages(sync: &mut GR3DSync, force_recalculate: bool) {
    for peer in &mut sync.peers {
        peer.record_advantage(sync.tick as u64, force_recalculate);
    }
}
