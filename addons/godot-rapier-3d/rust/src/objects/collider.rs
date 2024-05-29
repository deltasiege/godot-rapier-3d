use crate::queue::{Actionable, CanDispatchActions, QueueName};
use crate::utils::{HasCUID2Field, HasHandleField};
use crate::{ObjectKind, PhysicsObject};
use godot::engine::notify::Node3DNotification;
use godot::engine::{INode3D, Node3D, Shape3D};
use godot::prelude::*;
use rapier3d::prelude::*;

use super::collider_shape::{HasColliderShapeField, RapierSphereShape3D};
use super::Handle;
use super::HandleKind;
use super::RapierRigidBody3D;

#[derive(GodotClass)]
#[class(base = Node3D)]
pub struct RapierCollider3D {
    #[export]
    #[var(usage_flags = [EDITOR, STORAGE, READ_ONLY])]
    pub id: GString, // https://crates.io/crates/cuid2
    pub handle: Handle,
    pub parent: Option<Handle>,

    #[export]
    #[var(get, set = _on_shape_set)]
    pub shape: Option<Gd<Shape3D>>,

    #[export]
    pub restitution: f32,

    #[export]
    pub friction: f32,

    hot_reload_cb: Callable,
    shape_mutated_cb: Callable,
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for RapierCollider3D {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            id: GString::from(crate::cuid2()),
            handle: Handle::invalid(),
            parent: None,
            shape: None,
            restitution: 0.5,
            friction: 0.5,
            hot_reload_cb: Callable::invalid(),
            shape_mutated_cb: Callable::invalid(),
            base,
        }
    }

    fn on_notification(&mut self, what: Node3DNotification) {
        match what {
            Node3DNotification::EnterTree => self._on_enter_tree(),
            Node3DNotification::ExitTree => self._on_exit_tree(),
            Node3DNotification::Parented => self.on_parented(),
            Node3DNotification::TransformChanged => self.on_transform_changed(),
            _ => {}
        }
    }
}

#[godot_api]
impl RapierCollider3D {
    fn _on_enter_tree(&mut self) {
        if self.shape.is_none() {
            let shape_resource = RapierSphereShape3D::new_gd();
            self.shape = Some(shape_resource.upcast());
        }
        self.attach_shape_mutated(self.shape.clone().unwrap())
            .map_err(crate::handle_error)
            .ok();
        self.on_enter_tree();
    }

    fn _on_exit_tree(&mut self) {
        self.detach_shape_mutated(self.shape.clone().unwrap())
            .map_err(crate::handle_error)
            .ok();
        self.on_exit_tree();
    }

    #[func]
    fn _on_hot_reload(&mut self) {
        self.on_hot_reload();
    }

    fn on_parented(&mut self) {
        self.push_parent_action().map_err(crate::handle_error).ok();
    }

    fn push_parent_action(&mut self) -> Result<(), String> {
        let parent_id = match self.get_parent_rigid_body_node() {
            Some(rb) => Some(rb.bind().id.to_string()),
            None => None,
        };
        log::trace!(
            "Parenting collider '{}' to rigid body '{:?}'",
            self.id.to_string(),
            parent_id,
        );

        let action = self.get_action(Actionable::ColliderIDWithParentID(
            self.id.to_string(),
            parent_id,
        ));

        let mut engine = crate::get_engine()?;
        let mut bind = engine.bind_mut();
        let queue = bind
            .action_queue
            .queues
            .get_mut(&QueueName::Parent)
            .expect("QueueName::Parent not found");
        queue.push(action);
        Ok(())
    }

    fn get_parent_rigid_body_node(&self) -> Option<Gd<RapierRigidBody3D>> {
        match self.base().get_parent_node_3d() {
            Some(parent) => match parent.is_class(GString::from("RapierRigidBody3D")) {
                true => Some(parent.cast::<RapierRigidBody3D>()),
                false => None,
            },
            None => None,
        }
    }

    #[func]
    fn _on_shape_set(&mut self, shape: Option<Gd<Shape3D>>) {
        match shape {
            Some(shape) => {
                self.on_shape_change(shape.clone())
                    .map_err(crate::handle_error)
                    .ok();
                self.shape = Some(shape);
            }
            None => {
                log::error!("Collider must have a shape")
            }
        }
    }

    #[func]
    fn _on_shape_mutated(&self) {
        godot_print!("ON SHAPE MUTATED");
        self.refresh_shape().map_err(crate::handle_error).ok();
    }
}

impl HasColliderShapeField for RapierCollider3D {
    fn get_shape(&self) -> Option<Gd<Shape3D>> {
        self.shape.clone()
    }

    fn get_shape_mutated_cb(&self) -> Callable {
        self.shape_mutated_cb.clone()
    }

    fn set_shape_mutated_cb(&mut self, cb: Callable) {
        self.shape_mutated_cb = cb;
    }
}

impl HasCUID2Field for RapierCollider3D {
    fn get_cuid2(&self) -> String {
        self.id.to_string()
    }

    fn set_cuid2(&mut self, cuid2: String) {
        self.id = GString::from(cuid2);
    }
}

impl HasHandleField for RapierCollider3D {
    fn get_handle(&self) -> Handle {
        self.handle.clone()
    }

    fn set_handle(&mut self, handle: Handle) {
        match handle.kind {
            HandleKind::ColliderHandle | HandleKind::Invalid => {
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

impl CanDispatchActions for RapierCollider3D {}

impl PhysicsObject for RapierCollider3D {
    fn get_kind(&self) -> ObjectKind {
        ObjectKind::Collider
    }

    fn get_hot_reload_cb(&self) -> Callable {
        self.hot_reload_cb.clone()
    }

    fn set_hot_reload_cb(&mut self, cb: Callable) {
        self.hot_reload_cb = cb;
    }

    fn build(&self) -> Actionable {
        match self.get_shared_shape() {
            Ok(shared_shape) => {
                let collider = ColliderBuilder::new(shared_shape)
                    .restitution(self.restitution)
                    .friction(self.friction)
                    .build();
                Actionable::Collider(collider)
            }
            Err(e) => {
                log::error!("Failed to build collider {}: {}", self.id.to_string(), e);
                Actionable::Invalid
            }
        }
    }
}
