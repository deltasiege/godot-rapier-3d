use godot::obj::WithBaseField;
use godot::prelude::*;

use crate::impl_trait_for_all_nodes;
use crate::types::*;

pub trait Identifiable: WithBaseField + GodotClass<Base = Node3D> {
    fn get_gruid(&self) -> Option<GRUID> {
        get_gruid_from_metadata(self)
    }

    fn set_gruid(&mut self, gruid: GRUID) {
        set_gruid_in_metadata(self, gruid);
    }
}

/// Extracts the GRUID from the metadata of a node.
fn get_gruid_from_metadata(node: &impl Identifiable) -> Option<GRUID> {
    if !node.base().has_meta("gruid") {
        log::error!(
            "Node has no GRUID in metadata: {:?}",
            node.base().get_path()
        );
        return None;
    }

    match node.base().get_meta("gruid").try_to::<Array<i64>>() {
        Ok(arr) => {
            if arr.len() != 2 {
                log::error!(
                    "Invalid GRUID metadata on node: {:?}",
                    node.base().get_path()
                );
                return None;
            }
            Some((arr.at(0) as u8, arr.at(1) as u32))
        }
        Err(e) => {
            log::error!(
                "Error while parsing GRUID metadata on node '{}': {}",
                node.base().get_path(),
                e
            );
            None
        }
    }
}

/// Sets the GRUID in the metadata of a node.
fn set_gruid_in_metadata(node: &mut impl Identifiable, gruid: GRUID) {
    let arr = Array::from(&[gruid.0 as i64, gruid.1 as i64]).to_variant();
    node.base_mut().set_meta("gruid", &arr);
}

impl_trait_for_all_nodes!(Identifiable, {});
