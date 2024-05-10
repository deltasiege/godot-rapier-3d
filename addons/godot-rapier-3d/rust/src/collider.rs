use crate::physics_pipeline::GR3DPhysicsPipeline;
use crate::rigid_body::RapierRigidBody3D;
use godot::engine::notify::Node3DNotification;
use godot::engine::GDExtensionManager;
use godot::engine::INode3D;
use godot::engine::Node3D;
use godot::prelude::*;
use rapier3d::prelude::*;

#[derive(GodotClass)]
#[class(base = Node3D)]
pub struct RapierCollider3D {
    #[var]
    pub id: Array<Variant>, // ColliderHandle::into_raw_parts
    pub handle: ColliderHandle,
    pub parent: Option<RigidBodyHandle>,

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

    notify_parent: bool,
    hot_reload_cb: Callable,
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for RapierCollider3D {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            id: Array::new(),
            handle: ColliderHandle::invalid(),
            parent: None,
            shape: ShapeType::Ball,
            restitution: 0.5,
            friction: 0.5,
            ball_radius: 0.5,
            cuboid_half_extents: Vector3::new(0.5, 0.5, 0.5),
            notify_parent: true,
            hot_reload_cb: Callable::invalid(),
            base,
        }
    }

    fn on_notification(&mut self, what: Node3DNotification) {
        match what {
            Node3DNotification::EnterTree => self.on_enter_tree(),
            Node3DNotification::ExitTree => self.on_exit_tree(),
            Node3DNotification::Parented => self.on_parented(),
            Node3DNotification::Unparented => self.on_unparented(),
            Node3DNotification::TransformChanged => self.on_transform_changed(),
            _ => {}
        }
    }
}

#[godot_api]
impl RapierCollider3D {
    fn register(&mut self) {
        let mut engine = crate::get_engine!();
        let mut bind = engine.bind_mut();
        let handle = bind.pipeline.register_collider(self);
        self.handle = handle;
        self.id = crate::utils::collider_handle_to_id(handle);
        let collider = match bind.pipeline.get_collider_mut(self.handle) {
            Some(collider) => collider,
            None => {
                crate::error!(bind; "RapierCollider3D register - could not find collider {:?} in pipeline", self.handle);
                return;
            }
        };
        self.sync_transforms_to_godot(collider);
        crate::debug!(bind; "RapierCollider3D registered {:?}", self.handle.clone());
    }

    fn unregister(&mut self) {
        let mut engine = crate::get_engine!();
        let mut bind = engine.bind_mut();
        bind.pipeline.unregister_collider(self);
        crate::debug!(bind; "RapierCollider3D unregistered {:?}", self.handle.clone());
        self.handle = ColliderHandle::invalid();
        self.id = Array::new();
    }

    fn reregister(&mut self) {
        let mut engine = crate::get_engine!();
        let mut bind = engine.bind_mut();
        bind.pipeline.unregister_collider(self);
        let handle = bind.pipeline.register_collider(self);
        self.handle = handle;
        self.id = crate::utils::collider_handle_to_id(handle);
        let collider = match bind.pipeline.get_collider_mut(self.handle) {
            Some(collider) => collider,
            None => {
                crate::error!(bind; "RapierCollider3D reregister - could not find collider {:?} in pipeline", self.handle);
                return;
            }
        };
        self.sync_transforms_to_godot(collider);
        crate::debug!(bind; "RapierCollider3D reregistered {:?}", handle);
    }

    fn attach_extensions_reloaded_signal(&mut self) {
        let sig = Signal::from_object_signal(
            &GDExtensionManager::singleton(),
            StringName::from("extensions_reloaded")
        );
        let cb = Callable::from_object_method(&self.to_gd(), StringName::from("_on_hot_reload"));
        let already_connected = sig.is_connected(cb.clone());
        if already_connected {
            return;
        }
        GDExtensionManager::singleton().connect(
            StringName::from("extensions_reloaded"),
            cb.clone()
        );
        self.hot_reload_cb = cb;
    }

    fn detach_extensions_reloaded_signal(&mut self) {
        if !self.hot_reload_cb.is_null() {
            GDExtensionManager::singleton().disconnect(
                StringName::from("extensions_reloaded"),
                self.hot_reload_cb.clone()
            );
        }
    }

    #[func]
    fn _on_hot_reload(&mut self) {
        crate::debug!("RapierCollider3D _on_hot_reload {:?}", self.handle.clone());
        self.base_mut().set_notify_transform(false);
        self.reregister();
        self.base_mut().set_notify_transform(true);
    }

    fn on_enter_tree(&mut self) {
        self.register();
        self.attach_extensions_reloaded_signal();
        self.base_mut().set_notify_transform(true);
    }

    fn on_exit_tree(&mut self) {
        self.base_mut().set_notify_transform(false);
        self.unregister();
        self.detach_extensions_reloaded_signal();
    }

    fn on_parented(&mut self) {
        if !self.notify_parent {
            return;
        }
        self.notify_parent = false;
        let _res = self.base_mut().try_call_deferred(StringName::from("_on_parented"), &[]); // Collider registering needs to be defferred so that RigidBodies are already registered
    }

    #[func]
    fn _on_parented(&mut self) {
        let mut engine = crate::get_engine!();
        let mut bind = engine.bind_mut();
        let parent = match self.base().get_parent_node_3d() {
            Some(parent) => parent,
            None => {
                self.clear_parent(&mut bind.pipeline);
                return;
            }
        };
        let casted = match parent.is_class(GString::from("RapierRigidBody3D")) {
            true => parent.cast::<RapierRigidBody3D>(),
            false => {
                // Parented to something that is not a rigid body
                self.clear_parent(&mut bind.pipeline);
                self.notify_parent = true;
                return;
            }
        };
        let class = casted.bind();

        if self.parent == Some(class.handle) {
            // Parenting to already attached parent - do nothing
            self.notify_parent = true;
            return;
        }

        if !bind.pipeline.state.rigid_body_set.contains(class.handle) {
            // Trying to parent to rigid body that doesn't exist
            godot_error!(
                "RapierCollider3D _on_parented - could not find rigid body {:?} in pipeline",
                class.handle
            );
            self.notify_parent = true;
            return;
        }

        bind.pipeline.unregister_collider(self);
        let handle = bind.pipeline.register_collider_with_parent(self, class.handle);
        self.parent = Some(class.handle);
        self.handle = handle;
        self.id = crate::utils::collider_handle_to_id(handle);
        self.notify_parent = true;
    }

    fn on_unparented(&mut self) {
        if !self.notify_parent {
            return;
        }
        let mut engine = crate::get_engine!();
        let mut bind = engine.bind_mut();
        self.clear_parent(&mut bind.pipeline);
    }

    fn clear_parent(&mut self, pipeline: &mut GR3DPhysicsPipeline) {
        self.parent = None;
        pipeline.set_collider_parent(self.handle, None);
    }

    fn on_transform_changed(&mut self) {
        let mut engine = crate::get_engine!();
        let mut bind = engine.bind_mut();
        match bind.pipeline.get_collider_mut(self.handle) {
            Some(collider) => {
                self.sync_transforms_to_godot(collider);
            }
            None => {
                godot_error!(
                    "RapierCollider3D on_transform_changed - could not find collider {:?} in pipeline",
                    self.handle
                );
                return;
            }
        }
    }

    // Changes rapier transforms to match godot transforms
    fn sync_transforms_to_godot(&mut self, collider: &mut Collider) {
        if self.base().is_inside_tree() {
            let translation = self.base().get_global_position();
            let rotation = self.base().get_global_transform().basis.to_quat();
            let r_pos = crate::utils::pos_godot_to_rapier(translation);
            let r_rot = crate::utils::rot_godot_to_rapier(rotation);
            collider.set_translation(r_pos);
            collider.set_rotation(r_rot);
        }
    }

    pub fn build(&self) -> Collider {
        let shape = match self.shape {
            ShapeType::Ball => SharedShape::ball(self.ball_radius),
            ShapeType::Cuboid =>
                SharedShape::cuboid(
                    self.cuboid_half_extents.x,
                    self.cuboid_half_extents.y,
                    self.cuboid_half_extents.z
                ),
        };
        let collider = ColliderBuilder::new(shape)
            .restitution(self.restitution)
            .friction(self.friction)
            .build();
        collider
    }

    // This is gross - don't want a function for every single property
    // But don't want to define every single property like they are anyway (prefer Shape3Ds or Resources)
    // just wait until https://github.com/godot-rust/gdext/issues/440 is resolved
    #[func]
    fn set_shape(&mut self, shape: ShapeType) {
        self.shape = shape;
        self.base_mut().update_gizmos();
        self.reregister();
    }

    #[func]
    fn set_ball_radius(&mut self, radius: f32) {
        self.ball_radius = radius;
        self.base_mut().update_gizmos();
        self.reregister();
    }

    #[func]
    fn set_cuboid_half_extents(&mut self, half_extents: Vector3) {
        self.cuboid_half_extents = half_extents;
        self.base_mut().update_gizmos();
        self.reregister();
    }

    #[func]
    fn set_restitution(&mut self, restitution: f32) {
        self.restitution = restitution;
        self.reregister();
    }

    #[func]
    fn set_friction(&mut self, friction: f32) {
        self.friction = friction;
        self.reregister();
    }
}

#[derive(Debug, GodotConvert, Var, Export)]
#[godot(via = GString)]
// https://docs.rs/rapier3d/latest/rapier3d/geometry/enum.ShapeType.html
pub enum ShapeType {
    Ball,
    Cuboid,
}
