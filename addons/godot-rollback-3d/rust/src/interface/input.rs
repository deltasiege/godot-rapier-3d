use godot::prelude::*;

use crate::interface::GR3D;
use crate::network::PeerBuffers;
use crate::types::*;
use crate::utils::get_hash;

/// Return the input value for the given input key from either the local peer's input adapter or the
/// remote peer's buffer, depending on the peer_index of the provided GRUID.
pub fn get_input(gr3d: &mut GR3D, gruid: String, input_key: GString) -> Variant {
    let result = match try_get_input(gr3d, gruid.clone(), input_key.clone()) {
        Some(input) => input,
        None => Variant::nil(),
    };

    result
}

/// Option compatible version of get_input.
fn try_get_input(gr3d: &mut GR3D, gruid: String, input_key: GString) -> Option<Variant> {
    let tick = gr3d.world.time.tick.clone();
    let gruid = GRUID::try_from_string(&gruid)?;

    match gr3d.network.local_peer.is_local_gruid(gruid) {
        true => {
            // Local peer, get local node_state and then use it
            // to get input from the local adapter

            let node_state = gr3d.world.node_db.get_node_state(gruid);
            let adapter = gr3d.network.local_peer.get_adapter_mut()?;
            Some(get_input_from_adapter(
                adapter,
                input_key.clone(),
                node_state,
            ))
        }
        false => {
            // Remote peer, get input from the remote peer's buffer,
            // or fall back to default value from the local adapter
            let remote_input = gr3d.network.get_remote_input(
                gruid.get_peer_id(&gr3d.network)?,
                tick,
                input_key.clone(),
            );
            match remote_input {
                Some(input) => Some(input),
                None => {
                    let adapter = gr3d.network.local_peer.get_adapter_mut()?;
                    Some(get_default_from_adapter(adapter, input_key.clone()))
                }
            }
        }
    }
}

/// Store current local inputs in the given input buffer.
pub fn capture_current_inputs(
    buffers: &mut PeerBuffers,
    current_tick: Tick,
    adapter: &mut Option<Gd<GR3DInputAdapter>>,
) {
    match adapter {
        Some(adapter) => {
            let input_map = adapter.bind_mut().get_inputs();
            let serialized_inputs = adapter.bind_mut().get_ser_inputs();
            let input_hash = get_hash(&serialized_inputs);

            buffers.inputs.insert(current_tick, input_map);
            buffers.ser_inputs.insert(current_tick, serialized_inputs);
            buffers.input_hashes.insert(current_tick, input_hash);
        }
        None => {
            log::error!("capture_current_inputs failed: input adapter is not set");
        }
    }
}

fn get_input_from_adapter(
    adapter: &mut Gd<GR3DInputAdapter>,
    input_key: GString,
    node_state: Dictionary,
) -> Variant {
    adapter.call(
        "get_input",
        &[input_key.to_variant(), node_state.to_variant()],
    )
}

fn get_default_from_adapter(adapter: &mut Gd<GR3DInputAdapter>, input_key: GString) -> Variant {
    adapter.call("get_default", &[input_key.to_variant()])
}
