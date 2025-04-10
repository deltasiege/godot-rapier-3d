use godot::classes::Engine;
use godot::prelude::*;

use crate::interface::GR3D;
use crate::network::*;
use crate::types::*;

/// Network host notifies all other peers to start syncing and informs them of their peer index.
pub fn start_sync(gr3d: &mut GR3D) -> Result<(), ()> {
    let network = &mut gr3d.network;
    validate_net_state(network, "start_sync", false, true)?;
    let adapter = network.adapter.as_ref().unwrap();

    network.host_starting = true;
    let highest_peer_rtt = network
        .remote_peers
        .iter()
        .map(|peer| peer.rtt)
        .max()
        .unwrap_or(0);

    let peer_map = get_peer_map(network)?;

    for peer in network.remote_peers.iter() {
        adapter
            .bind()
            .send_remote_start(peer.metadata.id, peer_map.clone());
    }

    log::debug!("Delaying host start by {:?}", highest_peer_rtt);
    std::thread::sleep(std::time::Duration::from_millis(highest_peer_rtt as u64)); // TODO this may cause Godot to hang

    network.host_starting = false;
    on_received_remote_start(gr3d, peer_map.clone())?;
    Ok(())
}

/// Network host notifies all other peers to stop syncing.
pub fn stop_sync(gr3d: &mut GR3D) -> Result<(), ()> {
    let network = &mut gr3d.network;
    validate_net_state(network, "stop_sync", true, true)?;
    let adapter = network.adapter.as_ref().unwrap();

    for peer in &network.remote_peers {
        adapter.bind().send_remote_stop(peer.metadata.id);
    }

    on_received_remote_stop(gr3d)?;
    Ok(())
}

/// Reset all local data, define local peer and emit sync_started signal.
pub fn on_received_remote_start(gr3d: &mut GR3D, peer_map: PeerMap) -> Result<(), ()> {
    let world = &mut gr3d.world;
    let network = &mut gr3d.network;
    validate_net_state(network, "on_received_remote_start", false, false)?;

    let adapter = network.adapter.as_ref().unwrap();
    adapter.bind().on_sync_start();
    gr3d.logger.set_peer_id(adapter.bind().get_unique_id());

    network.started = true;

    let _ = apply_peer_map_to_network(network, peer_map.clone());

    world.reset();
    world.physics.integration_parameters.dt =
        1.0 / (Engine::singleton().get_physics_ticks_per_second() as f32);

    let pm_var = peer_map.to_variant();
    gr3d.base_mut().emit_signal("peer_map_changed", &[pm_var]);
    gr3d.base_mut().emit_signal("sync_started", &[]);

    Ok(())
}

/// Reset all local and remote peer data, remove local peer and emit sync_stopped signal.
pub fn on_received_remote_stop(gr3d: &mut GR3D) -> Result<(), ()> {
    let world = &mut gr3d.world;
    let network = &mut gr3d.network;
    validate_net_state(network, "on_received_remote_stop", true, false)?;

    let adapter = network.adapter.as_ref().unwrap();
    adapter.bind().on_sync_stop();

    network.started = false;
    network.host_starting = false;
    network.local_peer = None;

    network.remote_peers.iter_mut().for_each(|peer| {
        peer.reset();
    });

    world.reset();

    gr3d.base_mut().emit_signal("sync_stopped", &[]);
    Ok(())
}

/// Checks that network adapter is attached, the host if required, and that started/starting matches the desired state.
fn validate_net_state(
    network: &Network,
    func_name: &str,
    desired_state: bool,
    require_host: bool,
) -> Result<(), ()> {
    let current_state = network.started || network.host_starting;
    if !desired_state == current_state {
        let reason = match current_state {
            true => "already",
            false => "not",
        };

        log::error!("{} failed: sync is {} started", func_name, reason);
        return Err(());
    }

    match network.get_adapter() {
        Some(adapter) => {
            if !adapter.bind().is_network_host() && require_host {
                log::error!("{} should only be called on the network host", func_name);
                return Err(());
            }
        }
        None => {
            log::error!("{} failed: no NetworkAdapter is attached", func_name);
            return Err(());
        }
    };

    Ok(())
}
