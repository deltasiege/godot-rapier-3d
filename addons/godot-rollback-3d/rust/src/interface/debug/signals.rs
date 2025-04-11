use godot::prelude::*;

use crate::interface::GR3D;

/// Log a debug message when any signal is emitted.
pub fn debug_all_signals(gr3d: &mut GR3D) {
    debug_signals(&mut gr3d.base_mut(), "GR3D");
    if let Some(adapter) = gr3d.network.get_adapter_mut() {
        debug_signals(adapter, "NetworkAdapter");
    }
}

/// Log all signals emitted by the given node.
fn debug_signals(node: &mut Gd<impl Inherits<Object>>, node_name: &str) {
    let upcast = node.upcast_mut::<Object>();

    for signal_name in upcast
        .get_signal_list()
        .iter_shared()
        .map(|dict| dict.get("name").unwrap().to::<String>())
    {
        if is_excluded_signal(&signal_name) {
            continue;
        }

        let name = node_name.to_string();
        let sig_name = signal_name.clone();
        let callable = Callable::from_local_fn(&signal_name, move |args| {
            log::log!(
                signal_name_to_log_level(&sig_name),
                "Signal emitted: [{:?}][{:?}]: {:?}",
                name,
                sig_name,
                truncate_by_arg_type(args, &SIGNAL_DEBUG_ARGS_TRUNCATE_TYPES),
            );
            Ok(Variant::nil())
        });

        upcast.connect(signal_name.as_str(), &callable);
    }
}

fn is_excluded_signal(signal_name: &str) -> bool {
    match signal_name {
        _ if signal_name.contains("ping") => true,
        _ if signal_name.contains("received_tick_data") => true,
        _ => false,
    }
}

fn signal_name_to_log_level(signal_name: &str) -> log::Level {
    match signal_name {
        _ => log::Level::Debug,
    }
}

const SIGNAL_DEBUG_ARGS_TRUNCATE_TYPES: [VariantType; 1] = [VariantType::PACKED_BYTE_ARRAY];

/// Displays argument type instead of the full value for certain types.
fn truncate_by_arg_type(args: &[&Variant], exclude_types: &[VariantType]) -> Vec<Variant> {
    args.iter()
        .cloned()
        .filter_map(|arg| {
            let arg_type = arg.get_type();
            if exclude_types.contains(&arg_type) {
                Some(format!("<{}>", arg_type.as_str()).to_variant())
            } else {
                Some(arg.clone())
            }
        })
        .collect()
}
