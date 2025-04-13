use crate::nodes::*;
use godot::prelude::*;

pub trait Controllable: HasNodeData + RollbackNode {
    fn on_move_by_amount(&self, amount: Vector3) {
        if amount == Vector3::ZERO {
            return;
        }

        // TODO - move my rapier object by checking blueprint for handle ? or do I need queuing ?
        // I dont think so, inputs must be replayed in deterministic order instead
    }

    fn on_teleport_to_position(&self, position: Vector3) {
        // TODO - teleport my rapier object
    }
}

impl Controllable for RollbackKinematicCharacter3D {}
impl Controllable for RollbackPIDCharacter3D {}
