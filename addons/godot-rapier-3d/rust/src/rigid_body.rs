use godot::builtin::Callable;
use godot::engine::notify::Node3DNotification;
use godot::engine::GDExtensionManager;
use godot::engine::INode3D;
use godot::engine::Node3D;
use godot::prelude::*;
use rapier3d::prelude::*;

#[derive(GodotClass)]
#[class(base = Node3D)]
pub struct RapierRigidBody3D {
    #[var]
    pub id: Array<Variant>, // RigidBodyHandle::into_raw_parts
    pub handle: RigidBodyHandle,
    #[export]
    body_type: RigidBodyType,
    #[export]
    pub additional_mass: Real,
    hot_reload_cb: Callable,
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for RapierRigidBody3D {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            id: Array::new(),
            handle: RigidBodyHandle::invalid(),
            body_type: RigidBodyType::Dynamic,
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

// unsafe impl Send for RapierRigidBody3D {}
// unsafe impl Sync for RapierRigidBody3D {}

#[godot_api]
impl RapierRigidBody3D {
    fn register(&mut self) {
        let mut engine = crate::get_engine!();
        let mut bind = engine.bind_mut();
        let handle = bind.pipeline.register_rigid_body(self);
        self.handle = handle;
        self.id = crate::utils::rb_handle_to_id(handle);
        let rigid_body = match bind.pipeline.get_rigid_body_mut(self.handle) {
            Some(rigid_body) => rigid_body,
            None => {
                godot_error!(
                    "RapierRigidBody3D on_enter_tree - could not find rigid body in pipeline after registering"
                );
                return;
            }
        };

        self.sync_transforms_to_godot(rigid_body, false);
        crate::debug!(bind; "RapierRigidBody3D registered {:?}", self.handle.clone());
    }

    fn unregister(&mut self) {
        let mut engine = crate::get_engine!();
        let mut bind = engine.bind_mut();
        bind.pipeline.unregister_rigid_body(self);
        crate::debug!(bind; "RapierRigidBody3D unregistered {:?}", self.handle.clone());
        self.handle = RigidBodyHandle::invalid();
        self.id = Array::new();
    }

    fn reregister(&mut self) {
        let mut engine = crate::get_engine!();
        let mut bind = engine.bind_mut();
        bind.pipeline.unregister_rigid_body(self);
        let handle = bind.pipeline.register_rigid_body(self);
        self.handle = handle;
        self.id = crate::utils::rb_handle_to_id(handle);
        let rigid_body = match bind.pipeline.get_rigid_body_mut(self.handle) {
            Some(collider) => collider,
            None => {
                crate::error!(bind; "RapierRigidBody3D reregister - could not find rigid body {:?} in pipeline", self.handle);
                return;
            }
        };
        self.sync_transforms_to_godot(rigid_body, false);
        crate::debug!(bind; "RapierRigidBody3D reregistered {:?}", handle);
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
        crate::debug!("RapierRigidBody3D _on_hot_reload {:?}", self.handle.clone());
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

    fn on_transform_changed(&mut self) {
        let mut engine = crate::get_engine!();
        let mut bind = engine.bind_mut();
        match bind.pipeline.get_rigid_body_mut(self.handle) {
            Some(rigid_body) => {
                self.sync_transforms_to_godot(rigid_body, false);
            }
            None => {
                godot_error!(
                    "RapierRigidBody3D on_transform_changed - could not find rigid body {:?} in pipeline",
                    self.handle
                );
                return;
            }
        }
    }

    // Changes rapier transforms to match godot transforms
    fn sync_transforms_to_godot(&mut self, rigid_body: &mut RigidBody, wakeup: bool) {
        if self.base().is_inside_tree() {
            let translation = self.base().get_global_position();
            let rotation = self.base().get_quaternion();
            let r_pos = crate::utils::pos_godot_to_rapier(translation);
            let r_rot = crate::utils::rot_godot_to_rapier(rotation);
            rigid_body.set_translation(r_pos, wakeup);
            rigid_body.set_rotation(r_rot, wakeup);
        }
    }

    pub fn build(&self) -> RigidBody {
        let rb = match self.body_type {
            RigidBodyType::Dynamic => RigidBodyBuilder::dynamic(),
            RigidBodyType::Fixed => RigidBodyBuilder::fixed(),
            RigidBodyType::KinematicPositionBased => RigidBodyBuilder::kinematic_position_based(),
            RigidBodyType::KinematicVelocityBased => RigidBodyBuilder::kinematic_velocity_based(),
        };
        rb.additional_mass(self.additional_mass).build()
    }

    #[func]
    pub fn print_colliders(&self) {
        let mut engine = crate::get_engine!();
        let mut bind = engine.bind_mut();
        match bind.pipeline.get_rigid_body_mut(self.handle) {
            Some(rigid_body) => {
                godot_print!("Colliders: {:?}", rigid_body.colliders());
            }
            None => {
                godot_error!(
                    "RapierRigidBody3D print_colliders - could not find rigid body {:?} in pipeline",
                    self.handle
                );
                return;
            }
        }
    }
}

#[derive(GodotConvert, Var, Export)]
#[godot(via = GString)]
// https://docs.rs/rapier3d/latest/rapier3d/dynamics/enum.RigidBodyType.html
pub enum RigidBodyType {
    Dynamic,
    Fixed,
    KinematicPositionBased,
    KinematicVelocityBased,
}
