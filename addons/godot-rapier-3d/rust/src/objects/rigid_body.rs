use crate::queue::{Actionable, CanDispatchActions};
use crate::utils::{HasCUID2Field, HasHandleField};
use crate::{ObjectKind, PhysicsObject};
use godot::classes::notify::Node3DNotification;
use godot::classes::{INode3D, Node3D};
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
            body_type: RBType::default(),
            additional_mass: 0.0,
            hot_reload_cb: Callable::invalid(),
            base,
        }
    }

    fn on_notification(&mut self, what: Node3DNotification) {
        match what {
            Node3DNotification::ENTER_TREE => self.on_enter_tree(),
            Node3DNotification::EXIT_TREE => self.on_exit_tree(),
            Node3DNotification::TRANSFORM_CHANGED => self.on_transform_changed(),
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

    fn get_hot_reload_cb(&self) -> &Callable {
        &self.hot_reload_cb
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

impl HasCUID2Field for RapierRigidBody3D {
    fn get_cuid2(&self) -> String {
        self.id.to_string()
    }

    fn set_cuid2(&mut self, cuid2: String) {
        self.id = GString::from(cuid2);
    }
}

impl HasHandleField for RapierRigidBody3D {
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
}

impl CanDispatchActions for RapierRigidBody3D {}

#[derive(GodotConvert, Var, Export, Debug, Clone)]
#[godot(via = GString)]
// Mirror of https://docs.rs/rapier3d/latest/rapier3d/dynamics/enum.RigidBodyType.html
pub enum RBType {
    Dynamic,
    Fixed,
}

impl Default for RBType {
    fn default() -> Self {
        RBType::Dynamic
    }
}

impl Into<RigidBodyType> for RBType {
    fn into(self) -> RigidBodyType {
        match self {
            RBType::Dynamic => RigidBodyType::Dynamic,
            RBType::Fixed => RigidBodyType::Fixed,
        }
    }
}
