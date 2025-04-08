use godot::prelude::*;

use crate::interface::GR3D;

pub fn dump_debug_data(gr3d: &GR3D) -> GString {
    GString::from(format!(
        "GR3D Debug Data:\n\n{:#?}\n\n{:#?}",
        gr3d.world, gr3d.network
    ))
}
