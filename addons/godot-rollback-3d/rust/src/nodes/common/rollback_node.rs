use godot::classes::Engine;
use godot::obj::WithBaseField;
use godot::prelude::*;
use rapier3d::prelude::*;

use crate::impl_trait_for_all_nodes;
use crate::interface::get_gr3d;
use crate::types::RollbackNodeClass;
use crate::utils::isometry_to_transform;

use super::HasBlueprint;

pub trait RollbackNode: HasBlueprint + WithBaseField + GodotClass<Base = Node3D> {
    fn on_enter_tree(&mut self) {
        match Engine::singleton().is_editor_hint() {
            true => self.on_enter_editor_tree(),
            false => self.on_enter_runtime_tree(),
        }
    }

    fn on_enter_editor_tree(&mut self) {
        if let Some(mut gr3d) = get_gr3d() {
            // gr3d.call_deferred("_node_editor_enter", &[self.get_blueprint_variant()]);
        }
    }

    fn on_enter_runtime_tree(&mut self) {
        if let Some(mut gr3d) = get_gr3d() {
            // gr3d.call_deferred("_node_runtime_enter", &[self.get_blueprint_variant()]);
        }
    }

    fn on_exit_tree(&mut self) {
        match Engine::singleton().is_editor_hint() {
            true => self.on_exit_editor_tree(),
            false => self.on_exit_runtime_tree(),
        }
    }

    fn on_exit_editor_tree(&mut self) {
        if let Some(mut gr3d) = get_gr3d() {
            // gr3d.call_deferred("_node_editor_exit", &[self.get_blueprint_variant()]);
        }
    }

    fn on_exit_runtime_tree(&mut self) {
        if let Some(mut gr3d) = get_gr3d() {
            // gr3d.call_deferred("_node_runtime_exit", &[self.get_blueprint_variant()]);
        }
    }

    /// Sync Godot transform with Rapier transform
    fn sync(&mut self) {
        if let Some(gr3d) = get_gr3d() {
            if let Some(bp) = self.get_blueprint() {
                let physics = &gr3d.bind().world.physics;
                match bp.class {
                    RollbackNodeClass::RollbackRigidBody3D
                    | RollbackNodeClass::RollbackKinematicCharacter3D
                    | RollbackNodeClass::RollbackPIDCharacter3D => {
                        let handle =
                            RigidBodyHandle::from_raw_parts(bp.rapier_handle.0, bp.rapier_handle.1);
                        let dynamics = physics.islands.active_dynamic_bodies();
                        let kinematics = physics.islands.active_kinematic_bodies();
                        let active_bodies = [dynamics, kinematics].concat();
                        if !&active_bodies.contains(&handle) {
                            return;
                        }
                        let body = &physics.bodies[handle];

                        self.base_mut()
                            .set_global_transform(isometry_to_transform(body.position()));
                    }
                    _ => {}
                }
            }
        }
    }

    fn get_rollback_class(&self) -> Option<RollbackNodeClass> {
        match RollbackNodeClass::try_from(self.base().get_class()) {
            Ok(class) => Some(class),
            Err(e) => {
                log::error!(
                    "Failed to get rollback class for node '{}': {}",
                    self.base().get_path(),
                    e
                );
                None
            }
        }
    }
}

impl_trait_for_all_nodes!(RollbackNode, {});
