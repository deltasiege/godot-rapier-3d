use crate::queue::Actionable;
use crate::queue::QueueName;
use crate::ObjectKind;
use crate::PhysicsObject;
use godot::engine::notify::Node3DNotification;
use godot::engine::INode3D;
use godot::engine::Node3D;
use godot::prelude::*;
use rapier3d::prelude::*;

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
    #[var(get, set = set_shape)]
    pub shape: ShapeType,

    #[export]
    #[var(get, set = set_restitution)]
    pub restitution: f32,

    #[export]
    #[var(get, set = set_friction)]
    pub friction: f32,

    #[export]
    #[var(get, set = set_ball_radius)]
    pub ball_radius: f32,

    #[export]
    #[var(get, set = set_cuboid_half_extents)]
    pub cuboid_half_extents: Vector3,

    hot_reload_cb: Callable,
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for RapierCollider3D {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            id: GString::from(crate::cuid2()),
            handle: Handle::invalid(),
            parent: None,
            shape: ShapeType::Ball,
            restitution: 0.5,
            friction: 0.5,
            ball_radius: 0.5,
            cuboid_half_extents: Vector3::new(0.5, 0.5, 0.5),
            hot_reload_cb: Callable::invalid(),
            base,
        }
    }

    fn on_notification(&mut self, what: Node3DNotification) {
        match what {
            Node3DNotification::EnterTree => self.on_enter_tree(),
            Node3DNotification::ExitTree => self.on_exit_tree(),
            Node3DNotification::Parented => self.on_parented(),
            Node3DNotification::TransformChanged => self.on_transform_changed(),
            _ => {}
        }
    }
}

#[godot_api]
impl RapierCollider3D {
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

    // TODO This is gross - don't want a function for every single property
    // But don't want to define every single property like they are anyway (prefer Shape3Ds or Resources)
    // just wait until https://github.com/godot-rust/gdext/issues/440 is resolved
    #[func]
    fn set_shape(&mut self, shape: ShapeType) {
        self.shape = shape;
        self.base_mut().update_gizmos();
        // self.reregister().map_err(crate::handle_error).ok();
    }

    #[func]
    fn set_ball_radius(&mut self, radius: f32) {
        self.ball_radius = radius;
        self.base_mut().update_gizmos();
        // self.reregister().map_err(crate::handle_error).ok();
    }

    #[func]
    fn set_cuboid_half_extents(&mut self, half_extents: Vector3) {
        self.cuboid_half_extents = half_extents;
        self.base_mut().update_gizmos();
        // self.reregister().map_err(crate::handle_error).ok();
    }

    #[func]
    fn set_restitution(&mut self, restitution: f32) {
        self.restitution = restitution;
        // self.reregister().map_err(crate::handle_error).ok();
    }

    #[func]
    fn set_friction(&mut self, friction: f32) {
        self.friction = friction;
        // self.reregister().map_err(crate::handle_error).ok();
    }
}

impl PhysicsObject for RapierCollider3D {
    fn get_kind(&self) -> ObjectKind {
        ObjectKind::Collider
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

    fn get_hot_reload_cb(&self) -> Callable {
        self.hot_reload_cb.clone()
    }

    fn set_hot_reload_cb(&mut self, cb: Callable) {
        self.hot_reload_cb = cb;
    }

    fn build(&self) -> Actionable {
        let shape = match self.shape {
            ShapeType::Ball => SharedShape::ball(self.ball_radius),
            ShapeType::Cuboid => SharedShape::cuboid(
                self.cuboid_half_extents.x,
                self.cuboid_half_extents.y,
                self.cuboid_half_extents.z,
            ),
        };
        let collider = ColliderBuilder::new(shape)
            .restitution(self.restitution)
            .friction(self.friction)
            .build();
        Actionable::Collider(collider)
    }
}

#[derive(Debug, GodotConvert, Var, Export)]
#[godot(via = GString)]
// https://docs.rs/rapier3d/latest/rapier3d/geometry/enum.ShapeType.html
pub enum ShapeType {
    Ball,
    Cuboid,
}
