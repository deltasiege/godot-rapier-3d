use godot::classes::Engine;
use godot::obj::WithBaseField;
use godot::prelude::*;
use rapier3d::prelude::RigidBodyHandle;
use std::fmt;

use super::super::{
    pid_character::RapierPIDCharacter3D, RapierArea3D, RapierCollisionShape3D,
    RapierKinematicCharacter3D, RapierRigidBody3D, RapierStaticBody3D,
};
use super::identifiable::Identifiable;
use crate::nodes::generate_cuid;
use crate::world::Operation;
use crate::{interface::get_singleton, utils::isometry_to_transform};

pub trait IRapierObject: Identifiable + WithBaseField + GodotClass<Base = Node3D> {
    fn on_enter_tree(&mut self) {
        match Engine::singleton().is_editor_hint() {
            true => self.on_enter_editor_tree(),
            false => self.on_enter_runtime_tree(),
        }
    }

    fn on_enter_editor_tree(&mut self) {
        match self.has_cuid() {
            true => (),
            false => self.set_cuid(generate_cuid()),
        }
    }

    fn on_enter_runtime_tree(&mut self) {
        if let Some(mut singleton) = get_singleton() {
            singleton.call_deferred(
                "_ingest_local_action",
                &[
                    self.base().to_variant(),
                    Operation::AddNode.to_variant(),
                    Dictionary::new().to_variant(),
                ],
            );

            singleton.call_deferred(
                "_ingest_local_action",
                &[
                    self.base().to_variant(),
                    Operation::ConfigureNode.to_variant(),
                    Dictionary::new().to_variant(),
                ],
            );
        }
    }

    fn on_exit_tree(&mut self) {
        match Engine::singleton().is_editor_hint() {
            true => (),
            false => self.on_exit_runtime_tree(),
        }
    }

    fn on_exit_runtime_tree(&mut self) {
        if let Some(mut singleton) = get_singleton() {
            singleton.call_deferred(
                "_ingest_local_action",
                &[
                    self.base().to_variant(),
                    Operation::RemoveNode.to_variant(),
                    Dictionary::new().to_variant(),
                ],
            );
        }
    }

    // Sync Godot transform with Rapier transform
    fn sync(&mut self) {
        if let Some(singleton) = get_singleton() {
            if let Some(raw_handle) = self.get_handle_raw() {
                let physics = &singleton.bind().world.physics;
                let mut node = self.base_mut();
                let class = node.get_class();
                match class.to_string().as_str() {
                    "RapierRigidBody3D" | "RapierKinematicCharacter3D" | "RapierPIDCharacter3D" => {
                        let handle = RigidBodyHandle::from_raw_parts(raw_handle.0, raw_handle.1);
                        let dynamics = physics.islands.active_dynamic_bodies();
                        let kinematics = physics.islands.active_kinematic_bodies();
                        let active_bodies = [dynamics, kinematics].concat();
                        if !&active_bodies.contains(&handle) {
                            return;
                        }
                        let body = &physics.bodies[handle];

                        node.set_global_transform(isometry_to_transform(body.position()));
                    }
                    _ => {}
                }
            }
        }
    }
}

macro_rules! impl_irapier_object {
    ($t:ty) => {
        impl IRapierObject for $t {}

        impl fmt::Debug for $t {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.base().get_name())
            }
        }
    };
}

impl_irapier_object!(RapierArea3D);
impl_irapier_object!(RapierKinematicCharacter3D);
impl_irapier_object!(RapierCollisionShape3D);
impl_irapier_object!(RapierRigidBody3D);
impl_irapier_object!(RapierStaticBody3D);
impl_irapier_object!(RapierPIDCharacter3D);
