use godot::classes::Engine;
use godot::obj::WithBaseField;
use godot::prelude::*;

use crate::impl_trait_for_all_nodes;
use crate::interface::get_gr3d;
use crate::nodes::{HasBlueprint, HasNodeData};
use crate::types::RollbackNodeClass;
use crate::utils::isometry_to_transform;

pub trait RollbackNode:
    HasNodeData + HasBlueprint + WithBaseField + GodotClass<Base = Node3D>
{
    fn on_enter_tree(&mut self) {
        match Engine::singleton().is_editor_hint() {
            true => self.on_enter_editor_tree(),
            false => self.on_enter_runtime_tree(),
        }
    }

    fn on_enter_editor_tree(&mut self) {
        if let Some(mut gr3d) = get_gr3d() {
            gr3d.call_deferred("_node_editor_enter", &[self.get_blueprint_variant()]);
        }
    }

    fn on_enter_runtime_tree(&mut self) {
        if let Some(mut gr3d) = get_gr3d() {
            gr3d.call_deferred("_node_runtime_enter", &[self.get_blueprint_variant()]);
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
            gr3d.call_deferred("_node_editor_exit", &[self.get_blueprint_variant()]);
        }
    }

    fn on_exit_runtime_tree(&mut self) {
        if let Some(mut gr3d) = get_gr3d() {
            gr3d.call_deferred("_node_runtime_exit", &[self.get_blueprint_variant()]);
        }
    }

    /// Sync Godot transform with Rapier transform. Returns silently if:
    /// - node_data not available
    /// - the node is not a rigid body
    /// - the node is not active in the Rapier physics world
    fn sync(&mut self) {
        self.try_sync();
    }

    fn try_sync(&mut self) -> Option<()> {
        let gr3d = get_gr3d()?;
        let node_data = self.get_node_data()?;
        let physics = &gr3d.bind().world.physics;
        let handle = node_data.get_rapier_handle().left()?;

        let dynamics = physics.islands.active_dynamic_bodies();
        let kinematics = physics.islands.active_kinematic_bodies();
        let active_bodies = [dynamics, kinematics].concat();
        if !&active_bodies.contains(&handle) {
            return Some(());
        }
        let body = &physics.bodies[handle];

        self.base_mut()
            .set_global_transform(isometry_to_transform(body.position()));

        Some(())
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
