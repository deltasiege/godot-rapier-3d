use crate::queue::{Actionable, CanDispatchActions, QueueName};
use godot::engine::Shape3D;
use godot::obj::WithBaseField;
use godot::prelude::*;
use rapier3d::geometry::{ShapeType, SharedShape};

mod r#box;
mod capsule;
mod cylinder;
mod sphere;

pub use capsule::RapierCapsuleShape3D;
pub use cylinder::RapierCylinderShape3D;
pub use r#box::RapierBoxShape3D;
pub use sphere::RapierSphereShape3D;

// Implemented by collider.rs
pub trait HasColliderShapeField: WithBaseField<Base = Node3D> + CanDispatchActions {
    // Required
    fn get_shape(&self) -> Option<Gd<Shape3D>>;
    fn get_shape_mutated_cb(&self) -> Callable;
    fn set_shape_mutated_cb(&mut self, cb: Callable);

    // Provided
    fn shape_to_shared_shape(&self, shape: Gd<Shape3D>) -> Result<SharedShape, String> {
        let col_shape_type = ColliderShapeType::try_from(shape.get_class())?;
        let shared_shape = match col_shape_type {
            ColliderShapeType::Box => get_cast_shared_shape::<RapierBoxShape3D>(shape),
            ColliderShapeType::Capsule => get_cast_shared_shape::<RapierCapsuleShape3D>(shape),
            ColliderShapeType::Cylinder => get_cast_shared_shape::<RapierCylinderShape3D>(shape),
            ColliderShapeType::Sphere => get_cast_shared_shape::<RapierSphereShape3D>(shape),
        };
        Ok(shared_shape)
    }

    fn on_shape_change(&mut self, shape: Gd<Shape3D>) -> Result<(), String> {
        self.base_mut().update_gizmos();
        self.detach_shape_mutated(shape.clone())?;
        self.attach_shape_mutated(shape.clone())?;
        let shared_shape = self.shape_to_shared_shape(shape)?;
        self.push_shape_change(shared_shape)?;
        Ok(())
    }

    fn push_shape_change(&self, shape: SharedShape) -> Result<(), String> {
        let mut engine = crate::get_engine()?;
        engine.bind_mut().action_queue.add_action(
            self.get_action(Actionable::ColliderShape(shape)),
            &QueueName::Sync,
        );
        Ok(())
    }

    fn get_shared_shape(&self) -> Result<SharedShape, String> {
        match self.get_shape() {
            Some(shape) => self.shape_to_shared_shape(shape.clone()),
            None => Err("Collider must have a shape".to_string()),
        }
    }

    fn refresh_shape(&self) -> Result<(), String> {
        let shared_shape = self.get_shared_shape()?;
        self.push_shape_change(shared_shape)
    }

    fn attach_shape_mutated(&mut self, mut shape: Gd<Shape3D>) -> Result<(), String> {
        let cb: Callable =
            Callable::from_object_method(&self.base(), StringName::from("_on_shape_mutated"));
        let sig = Signal::from_object_signal(&shape, StringName::from("changed"));
        match sig.is_connected(cb.clone()) {
            true => {}
            false => {
                shape.connect(StringName::from("changed"), cb.clone());
                self.set_shape_mutated_cb(cb)
            }
        }
        Ok(())
    }

    fn detach_shape_mutated(&mut self, mut shape: Gd<Shape3D>) -> Result<(), String> {
        let sig = Signal::from_object_signal(&shape, StringName::from("changed"));
        let cb = self.get_shape_mutated_cb();
        if !self.get_shape_mutated_cb().is_null() && sig.is_connected(cb.clone()) {
            shape.disconnect(StringName::from("changed"), cb);
        };
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum ColliderShapeType {
    Box,
    Capsule,
    Cylinder,
    Sphere,
}

impl TryFrom<GString> for ColliderShapeType {
    type Error = String;

    fn try_from(shape: GString) -> Result<Self, Self::Error> {
        match shape.to_string().as_str() {
            "RapierBoxShape3D" => Ok(ColliderShapeType::Box),
            "RapierCapsuleShape3D" => Ok(ColliderShapeType::Capsule),
            "RapierCylinderShape3D" => Ok(ColliderShapeType::Cylinder),
            "RapierSphereShape3D" => Ok(ColliderShapeType::Sphere),
            _ => Err(format!("Unknown shape type {}", shape.to_string())),
        }
    }
}

impl From<ColliderShapeType> for ShapeType {
    fn from(shape: ColliderShapeType) -> Self {
        match shape {
            ColliderShapeType::Box => ShapeType::Cuboid,
            ColliderShapeType::Capsule => ShapeType::Capsule,
            ColliderShapeType::Cylinder => ShapeType::Cylinder,
            ColliderShapeType::Sphere => ShapeType::Ball,
        }
    }
}

// Implemented by all collision shapes
pub trait HasSharedShapeField {
    fn get_shared_shape(&self) -> SharedShape;
}

fn get_cast_shared_shape<T: HasSharedShapeField + Inherits<Shape3D> + WithBaseField>(
    shape: Gd<Shape3D>,
) -> SharedShape {
    let gr3d_shape = shape.cast::<T>();
    let bind = gr3d_shape.bind();
    bind.get_shared_shape()
}
