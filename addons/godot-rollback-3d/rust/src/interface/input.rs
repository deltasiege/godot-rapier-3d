use godot::prelude::*;

use crate::adapters::GR3DInputAdapter;
use crate::interface::GR3D;
use crate::network::PeerBuffers;
use crate::types::*;
use crate::utils::get_hash;

/// Return the input value for the given input key from either the local peer's input adapter or the
/// remote peer's buffer, depending on the multiplayer authority of the given node.
pub fn get_input(gr3d: &mut GR3D, input_key: GString, node: Gd<Node>) -> Variant {
    let authority = node.get_multiplayer_authority() as PeerId;
    let tick = gr3d.world.time.tick.clone();

    match gr3d.network.local_peer.is_local_peer_id(authority) {
        true => {
            if let Some(adapter) = gr3d.network.local_peer.get_adapter_mut() {
                return get_input_from_adapter(adapter, input_key.clone()); // Local peer, get input from the local adapter
            }
        }
        false => {
            if let Some(input) = gr3d
                .network
                .get_remote_input(authority, tick, input_key.clone())
            {
                return input; // Remote peer, get input from the remote peer's buffer
            } else if let Some(adapter) = gr3d.network.local_peer.get_adapter_mut() {
                return get_default_from_adapter(adapter, input_key.clone()); // Fallback to default from the local adapter
            }
        }
    }

    Variant::nil()
}

fn get_input_from_adapter(adapter: &mut Gd<GR3DInputAdapter>, input_key: GString) -> Variant {
    adapter.call("get_input", &[input_key.to_variant()])
}

fn get_default_from_adapter(adapter: &mut Gd<GR3DInputAdapter>, input_key: GString) -> Variant {
    adapter.call("get_default", &[input_key.to_variant()])
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
