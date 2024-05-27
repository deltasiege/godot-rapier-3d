use crate::queue::Actionable;
use crate::{ObjectKind, PhysicsObject};
use godot::engine::notify::Node3DNotification;
use godot::engine::{INode3D, Node3D};
use godot::prelude::*;
use rapier3d::dynamics::{RigidBodyBuilder, RigidBodyType};
use rapier3d::math::Real;

use super::{Handle, HandleKind};

#[derive(GodotClass, Debug)]
#[class(base = Node3D)]
pub struct RapierRigidBody3D {
    #[export]
    #[var(usage_flags = [EDITOR, STORAGE, READ_ONLY])]
    pub id: GString, // https://crates.io/crates/cuid2
    pub handle: Handle,
    #[export]
    body_type: RBType,
    #[export]
    pub additional_mass: Real,
    hot_reload_cb: Callable,
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for RapierRigidBody3D {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            id: GString::from(crate::cuid2()),
            handle: Handle::invalid(),
            body_type: RBType::Dynamic,
            additional_mass: 0.0,
            hot_reload_cb: Callable::invalid(),
            base,
        }
    }

    fn on_notification(&mut self, what: Node3DNotification) {
        match what {
            Node3DNotification::EnterTree => self.on_enter_tree(),
            Node3DNotification::ExitTree => self.on_exit_tree(),
            Node3DNotification::TransformChanged => self.on_transform_changed(),
            _ => {}
        }
    }
}

#[godot_api]
impl RapierRigidBody3D {
    #[func]
    fn _on_hot_reload(&mut self) {
        self.on_hot_reload();
    }
}

impl PhysicsObject for RapierRigidBody3D {
    fn get_kind(&self) -> ObjectKind {
        ObjectKind::RigidBody
    }

    fn get_cuid2(&self) -> String {
        self.id.to_string()
    }

    fn set_cuid2(&mut self, cuid2: String) {
        self.id = GString::from(cuid2);
    }

    fn get_handle(&self) -> Handle {
        self.handle.clone()
    }

    fn set_handle(&mut self, handle: Handle) {
        match handle.kind {
            HandleKind::RigidBodyHandle | HandleKind::Invalid => {
                self.handle = handle;
            }
            _ => log::error!(
                "[{}] Invalid handle kind '{:?}'",
                self.id.to_string(),
                handle.kind
            ),
        }
    }

    fn get_hot_reload_cb(&self) -> Callable {
        self.hot_reload_cb.clone()
    }

    fn set_hot_reload_cb(&mut self, cb: Callable) {
        self.hot_reload_cb = cb;
    }

    fn build(&self) -> Actionable {
        let rb = RigidBodyBuilder::new(self.body_type.clone().into())
            .additional_mass(self.additional_mass)
            .build();
        Actionable::RigidBody(rb)
    }
}

#[derive(GodotConvert, Var, Export, Debug, Clone)]
#[godot(via = GString)]
// Mirror of https://docs.rs/rapier3d/latest/rapier3d/dynamics/enum.RigidBodyType.html
pub enum RBType {
    Dynamic,
    Fixed,
    KinematicPositionBased,
    KinematicVelocityBased,
}

impl Into<RigidBodyType> for RBType {
    fn into(self) -> RigidBodyType {
        match self {
            RBType::Dynamic => RigidBodyType::Dynamic,
            RBType::Fixed => RigidBodyType::Fixed,
            RBType::KinematicPositionBased => RigidBodyType::KinematicPositionBased,
            RBType::KinematicVelocityBased => RigidBodyType::KinematicVelocityBased,
        }
    }
}