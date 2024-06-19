use crate::engine::get_engine;
use crate::queue::{Actionable, CanDispatchActions, QueueName};
use crate::utils::{isometry_to_transform, transform_to_isometry, HasCUID2Field, HasHandleField};
use crate::LookupIdentifier;
use godot::builtin::Transform3D;
use godot::engine::{GDExtensionManager, Node3D};
use godot::obj::WithBaseField;
use godot::prelude::*;
use rapier3d::math::{Isometry, Real};

mod character;
mod collider;
mod collider_shape;
mod handle;
mod object_kind;
mod rigid_body;

pub use character::RapierCharacterBody3D;
pub use collider::RapierCollider3D;
pub use handle::{Handle, HandleKind};
pub use object_kind::ObjectKind;
pub use rigid_body::RapierRigidBody3D;

// Implemented by all classes that inherit a Godot node and have an underlying Rapier object
pub trait PhysicsObject:
    WithBaseField<Base = Node3D> + CanDispatchActions + HasCUID2Field + HasHandleField
{
    // Required
    fn get_kind(&self) -> ObjectKind;
    fn get_hot_reload_cb(&self) -> Callable;
    fn set_hot_reload_cb(&mut self, cb: Callable);
    fn build(&self) -> Actionable;

    // Provided
    fn is_registered(&self) -> Result<bool, String> {
        let cuid2 = self.get_cuid2();
        let engine = get_engine()?;
        let lookups = &engine.bind().lookups;
        Ok(lookups
            .get(self.get_kind(), LookupIdentifier::ID, &cuid2)
            .is_some())
    }

    fn attach_extensions_reloaded(&mut self) {
        let sig = Signal::from_object_signal(
            &GDExtensionManager::singleton(),
            StringName::from("extensions_reloaded"),
        );
        let cb = Callable::from_object_method(&self.base(), StringName::from("_on_hot_reload"));
        let already_connected = sig.is_connected(cb.clone());
        if already_connected {
            return;
        }
        GDExtensionManager::singleton()
            .connect(StringName::from("extensions_reloaded"), cb.clone());
        self.set_hot_reload_cb(cb);
    }

    fn detach_extensions_reloaded(&mut self) {
        if !self.get_hot_reload_cb().is_null() {
            GDExtensionManager::singleton().disconnect(
                StringName::from("extensions_reloaded"),
                self.get_hot_reload_cb(),
            );
        }
    }

    fn on_hot_reload(&mut self) {
        log::debug!("Hot reloading {:?}", self.get_cuid2());
        self.base_mut().set_notify_transform(false);
        self.register().map_err(crate::handle_error).ok();
        self.sync_r2g().map_err(crate::handle_error).ok();
    }

    fn on_enter_tree(&mut self) {
        self.register().map_err(crate::handle_error).ok();
        self.attach_extensions_reloaded();
        self.sync_r2g().map_err(crate::handle_error).ok();
    }

    fn on_exit_tree(&mut self) {
        self.base_mut().set_notify_transform(false);
        self.unregister().map_err(crate::handle_error).ok();
        self.detach_extensions_reloaded();
    }

    fn on_transform_changed(&mut self) {
        // Change rapier transform to match godot transform
        self.sync_r2g().map_err(crate::handle_error).ok();
    }

    fn get_rapier_position(&self) -> Result<Isometry<Real>, String> {
        let handle = self.get_handle();
        let engine = get_engine()?;
        let bind = engine.bind();
        bind.pipeline.get_object_position(handle)
    }

    fn get_node_transform(&self) -> Result<Transform3D, String> {
        let node = self.base();
        match node.is_inside_tree() {
            true => Ok(node.get_global_transform()),
            false => Err(format!("Node '{:?}' is not inside tree", node.get_name())),
        }
    }

    fn set_node_transform(&mut self, transform: Transform3D) -> Result<(), String> {
        let mut node = self.base_mut();
        match node.is_inside_tree() {
            true => {
                node.set_global_transform(transform);
                Ok(())
            }
            false => Err(format!("Node '{:?}' is not inside tree", node.get_name())),
        }
    }

    fn register(&mut self) -> Result<(), String> {
        log::debug!("Registering {} {:?}", self.get_kind(), self.get_cuid2());
        let action = self.get_action(self.build());
        let mut engine = get_engine()?;
        let mut bind = engine.bind_mut();
        let queue = bind
            .action_queue
            .queues
            .get_mut(&QueueName::Insert)
            .expect("QueueName::Insert not found");
        queue.push(action);
        Ok(())
    }

    fn unregister(&mut self) -> Result<(), String> {
        log::debug!("Unregistering {} {:?}", self.get_kind(), self.get_cuid2());
        let action = self.get_action(Actionable::from(self.get_handle()));
        self.set_handle(Handle::invalid());
        let mut engine = get_engine()?;
        let mut bind = engine.bind_mut();
        let queue = bind
            .action_queue
            .queues
            .get_mut(&QueueName::Remove)
            .expect("QueueName::Remove not found");
        queue.push(action);
        Ok(())
    }

    fn reregister(&mut self) -> Result<(), String> {
        self.unregister()?;
        self.register()
    }

    fn sync_r2g(&mut self) -> Result<(), String> {
        log::trace!("Syncing r2g for {} {:?}", self.get_kind(), self.get_cuid2());
        let node_pos = self.get_godot_isometry()?;
        let action = self.get_action(Actionable::NodePos(self.get_kind(), node_pos));
        let mut engine = get_engine()?;
        let mut bind = engine.bind_mut();
        bind.action_queue.add_action(action, &QueueName::Sync);
        Ok(())
    }

    // Changes godot transforms to match rapier transforms
    fn sync_g2r(&mut self) -> Result<(), String> {
        log::trace!(
            "Syncing g2r for {:?} {:?}",
            self.get_kind(),
            self.get_cuid2()
        );
        let transform = self.get_rapier_transform()?;
        self.set_node_transform(transform)?;
        Ok(())
    }

    // Returns the current position and rotation of the godot node as a Rapier Isometry
    fn get_godot_isometry(&self) -> Result<Isometry<Real>, String> {
        let transform = self.get_node_transform()?;
        Ok(transform_to_isometry(transform))
    }

    // Returns the current position and rotation of the internal Rapier object as a Godot Transform
    fn get_rapier_transform(&self) -> Result<Transform3D, String> {
        let isometry = self.get_rapier_position()?;
        Ok(isometry_to_transform(isometry))
    }
}
