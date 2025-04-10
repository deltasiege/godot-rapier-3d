use godot::prelude::*;

use crate::interface::GR3D;
use crate::utils::vector_to_godot;
use crate::{Network, World};

/// Returns a string with all GR3D debug information.
pub fn get_debug_string(gr3d: &GR3D) -> GString {
    GString::from(format!(
        "GR3D Debug Data:\n\n{:#?}\n\n{:#?}",
        gr3d.world, gr3d.network
    ))
}

/// Returns a godot dictionary with all GR3D debug information.
pub fn get_debug_dictionary(gr3d: &GR3D) -> Dictionary {
    let mut dict = Dictionary::new();
    dict.set("world", world_dictionary(&gr3d.world));
    dict.set("network", network_dictionary(&gr3d.network));
    dict
}

/// Log a debug message when any signal is emitted.
pub fn debug_all_signals(gr3d: &mut GR3D) {
    debug_signals(&mut gr3d.base_mut(), "GR3D");
    if let Some(adapter) = gr3d.network.get_adapter_mut() {
        debug_signals(adapter, "NetworkAdapter");
    }
}

fn debug_signals(node: &mut Gd<impl Inherits<Object>>, node_name: &str) {
    let upcast = node.upcast_mut::<Object>();

    for signal_name in upcast
        .get_signal_list()
        .iter_shared()
        .map(|dict| dict.get("name").unwrap().to::<String>())
    {
        let name = node_name.to_string();
        let sig_name = signal_name.clone();
        let callable = Callable::from_local_fn(&signal_name, move |args| {
            log::debug!("Signal emitted: [{:?}][{:?}]: {:?}", name, sig_name, args);
            Ok(Variant::nil())
        });

        upcast.connect(signal_name.as_str(), &callable);
    }
}

fn world_dictionary(world: &World) -> Dictionary {
    let mut dict = Dictionary::new();
    let mut time = Dictionary::new();
    let mut phx = Dictionary::new();
    let mut nodes = Dictionary::new();

    time.set("tick", world.time.tick as i64);
    time.set("seconds", world.time.secs as f64);

    phx.set("bodies", world.physics.bodies.len() as i64);
    phx.set("colliders", world.physics.colliders.len() as i64);
    phx.set("impulse_joints", world.physics.impulse_joints.len() as i64);
    let mb_joints = world.physics.multibody_joints.iter().count();
    phx.set("multibody_joints", mb_joints as i64);
    phx.set("gravity", vector_to_godot(world.physics.gravity));
    let tick_interval = world.physics.integration_parameters.dt as f64;
    phx.set("tick_interval", tick_interval);

    nodes.set("total_nodes", world.node_db.nodes.len() as i64);

    dict.set("time", time);
    dict.set("physics", phx);
    dict.set("nodes", nodes);
    dict
}

fn network_dictionary(network: &Network) -> Dictionary {
    let mut dict = Dictionary::new();
    let mut local_peer = Dictionary::new();
    let mut remote_peers = Dictionary::new();
    let mut peer_map = Dictionary::new();

    dict.set("started", network.started);
    dict.set("host_starting", network.host_starting);
    dict.set("adapter", class_or_none(&network.adapter));

    if let Some(lp) = &network.local_peer {
        local_peer.set("id", lp.metadata.id);
        local_peer.set("idx", lp.metadata.idx.unwrap_or(-1));
        local_peer.set("is_spectator", lp.metadata.is_spectator);
        local_peer.set("input_adapter", class_or_none(&lp.input_adapter));

        let mut buffers = Dictionary::new();
        buffers.set("inputs", lp.buffers.inputs.len() as i64);
        buffers.set("ser_inputs", lp.buffers.ser_inputs.len() as i64);
        buffers.set("input_hashes", lp.buffers.input_hashes.len() as i64);
        buffers.set("world_snapshots", lp.world_snapshots.len() as i64);
        buffers.set("world_hashes", lp.buffers.world_hashes.len() as i64);
    }

    for peer in network.remote_peers.iter() {
        let mut pd = Dictionary::new();
        pd.set("id", peer.metadata.id);
        pd.set("idx", peer.metadata.idx.unwrap_or(-1));
        pd.set("is_spectator", peer.metadata.is_spectator);
        pd.set("rtt", peer.rtt as i64);
        pd.set("last_ping_received", peer.last_ping_received as i64);
        pd.set("time_delta", peer.time_delta);
        let lrtr = peer.latest_remote_tick_received;
        pd.set("latest_remote_tick_received", lrtr as i64);
        let lltr = peer.latest_local_tick_requested;
        pd.set("latest_local_tick_requested", lltr as i64);
        pd.set("remote_lag", peer.remote_lag);
        pd.set("local_lag", peer.local_lag);
        pd.set("calculated_advantage", peer.calculated_advantage);
        pd.set("advantage_list", peer.advantage_list.to_variant());

        let mut buffers = Dictionary::new();
        buffers.set("inputs", peer.buffers.inputs.len() as i64);
        buffers.set("ser_inputs", peer.buffers.ser_inputs.len() as i64);
        buffers.set("input_hashes", peer.buffers.input_hashes.len() as i64);
        buffers.set("world_hashes", peer.buffers.world_hashes.len() as i64);

        remote_peers.set(format!("Remote peer: {}", peer.metadata.id), pd);
    }

    for (idx, peer) in network.remote_peers.iter().enumerate() {
        peer_map.set(idx as i64, peer.metadata.id);
    }

    dict.set("local_peer", local_peer);
    dict.set("remote_peers", remote_peers);
    dict.set("peer_map", peer_map);

    dict
}

fn class_or_none(node: &Option<Gd<impl Inherits<Object>>>) -> GString {
    match node {
        Some(ref node) => node.upcast_ref().get_class(),
        None => GString::from("None"),
    }
}
