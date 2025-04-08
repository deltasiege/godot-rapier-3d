use godot::prelude::*;

pub fn disconnect_outgoing_signals(node: &mut Gd<impl Inherits<Object>>) {
    let upcast = node.upcast_mut::<Object>();

    for signal_name in upcast
        .get_signal_list()
        .iter_shared()
        .map(|dict| dict.get("name").unwrap().to::<String>())
    {
        for dict in upcast
            .get_signal_connection_list(&signal_name)
            .iter_shared()
        {
            let sig = dict.get("signal").unwrap().to::<Signal>();
            let callable = dict.get("callable").unwrap().to::<Callable>();
            upcast.disconnect(&sig.name(), &callable);
        }
    }
}

pub fn disconnect_incoming_signals(node: &mut Gd<impl Inherits<Object>>) {
    let upcast = node.upcast_mut::<Object>();

    for dict in upcast.get_incoming_connections().iter_shared() {
        let sig = dict.get("signal").unwrap().to::<Signal>();
        let callable = dict.get("callable").unwrap().to::<Callable>();
        upcast.disconnect(&sig.name(), &callable);
    }
}
