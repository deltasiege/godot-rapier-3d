use super::super::{pid_character::RapierPIDCharacter3D, RapierKinematicCharacter3D};
use super::rapier_object::IRapierObject;
use crate::interface::{get_singleton, Operation};
use godot::prelude::*;

pub trait Controllable: IRapierObject {
    fn on_move_by_amount(&self, amount: Vector3) {
        if let Some(mut singleton) = get_singleton() {
            let mut dict = Dictionary::new();
            dict.set("movement", amount);

            singleton.call_deferred(
                "_ingest_action",
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
