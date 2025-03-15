use super::super::{pid_character::RapierPIDCharacter3D, RapierKinematicCharacter3D};
use super::rapier_object::IRapierObject;
use crate::interface::get_runtime;
use godot::prelude::*;

pub trait Controllable: IRapierObject {
    fn on_move_by_amount(&self, amount: Vector3) {
        if let Some(mut runtime) = get_runtime(self) {
            runtime.call(
                "_move_node",
                &[self.to_gd().to_variant(), amount.to_variant()],
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
