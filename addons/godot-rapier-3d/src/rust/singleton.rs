// use crate::utils::transform_to_posrot;
// use godot::engine::notify::Node3DNotification;
use godot::engine::IObject;
use godot::engine::Object;
use godot::prelude::*;
// use nalgebra::Vector3 as NAVector3;
use rapier3d::prelude::*;
// use rapier3d::geometry::ShapeType

#[derive(GodotClass)]
#[class(base=Object)]
pub struct Rapier3DSingleton {
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    base: Base<Object>,
}

#[godot_api]
impl IObject for Rapier3DSingleton {
    fn init(base: Base<Object>) -> Self {
        Self {
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            base,
        }
    }
}

#[godot_api]
impl Rapier3DSingleton {
    pub fn say_hi(&self) {
        godot_print!("Hi from singleton!");
    }

    pub fn add_collider(&mut self, collider: Collider) -> ColliderHandle {
        godot_print!("Hi from the editor plugin!");
        self.collider_set.insert(collider);
    }
}
