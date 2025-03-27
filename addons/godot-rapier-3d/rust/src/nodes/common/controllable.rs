use super::super::{pid_character::RapierPIDCharacter3D, RapierKinematicCharacter3D};
use crate::actions::Operation;
use crate::interface::get_net_singleton;
use crate::nodes::IRapierObject;
use godot::prelude::*;

pub trait Controllable: IRapierObject {
    fn on_move_by_amount(&self, amount: Vector3) {
        if amount == Vector3::ZERO {
            return;
        }

        if let Some(mut singleton) = get_net_singleton() {
            let mut dict = Dictionary::new();
            dict.set("movement", amount);

            singleton.call_deferred(
                "_ingest_local_action",
                &[
                    self.to_gd().to_variant(),
                    Operation::MoveNode.to_variant(),
                    dict.to_variant(),
                ],
            );
        }
    }

    fn on_teleport_to_position(&self, position: Vector3) {
        if let Some(mut singleton) = get_net_singleton() {
            let mut dict = Dictionary::new();
            dict.set("position", position);

            singleton.call_deferred(
                "_ingest_local_action",
                &[
                    self.to_gd().to_variant(),
                    Operation::MoveNode.to_variant(),
                    dict.to_variant(),
                ],
            );
        }
    }
}

macro_rules! impl_controllable {
    ($t:ty) => {
        impl Controllable for $t {}
    };
}

impl_controllable!(RapierKinematicCharacter3D);
impl_controllable!(RapierPIDCharacter3D);
