use godot::prelude::*;

use crate::interface::GR3D;
use crate::utils::{get_system_time_ms, get_system_time_ms_gstr};
use crate::Network;

/// Sends a ping to all remote peers in the network.
pub fn ping_all_peers(network: &Network) -> Result<(), ()> {
    let peers = &network.remote_peers;
    let adapter = network.get_adapter().ok_or(())?;
    if peers.is_empty() {
        return Ok(());
    }

    for peer in peers {
        if peer.metadata.id == adapter.bind().get_unique_id() {
            log::error!("Cannot ping ourselves");
            continue;
        }

        adapter
            .bind()
            .send_ping(peer.metadata.id, get_system_time_ms_gstr());
    }

    Ok(())
}

/// Responds to a ping from a remote peer.
pub fn return_ping(network: &Network, peer_id: i64, origin_ts: GString) -> Result<(), ()> {
    let adapter = network.get_adapter().ok_or(())?;

    if peer_id == adapter.bind().get_unique_id() {
        log::error!("Cannot ping back ourselves");
        return Err(());
    }

    let local_time = get_system_time_ms_gstr();
    adapter
        .bind()
        .send_ping_back(peer_id, origin_ts, local_time);

    Ok(())
}

/// Records the round-trip time (RTT) of a peer once return_ping response is received.
pub fn record_rtt(
    gr3d: &mut GR3D,
    peer_id: i64,
    origin_ts: u128,
    remote_ts: u128,
) -> Result<(), ()> {
    let current_local_ts = get_system_time_ms();

    let peer = gr3d.network.get_remote_peer_mut(peer_id).ok_or(())?;
    peer.last_ping_received = current_local_ts;
    peer.rtt = current_local_ts - remote_ts;

    let rf = remote_ts as f64;
    let lf = origin_ts as f64;
    let rtt = peer.rtt as f64;
    peer.time_delta = rf - lf - (rtt / 2.0);

    gr3d.base_mut()
        .emit_signal("peer_pinged_back", &[peer_id.to_variant()]);

    Ok(())
}

/// Records the advantages of all remote peers in the network.
pub fn record_all_advantages(gr3d: &mut GR3D, force_recalculate: bool) {
    for peer in &mut gr3d.network.remote_peers {
        peer.record_advantage(gr3d.world.time.tick, force_recalculate);
    }
}
