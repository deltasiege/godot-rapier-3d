use crate::utils::transform_to_posrot;
use godot::engine::notify::Node3DNotification;
use godot::engine::INode3D;
use godot::engine::Node3D;
use godot::prelude::*;
use nalgebra::Vector3 as NAVector3;
use rapier3d::prelude::*;
// use rapier3d::geometry::ShapeType

#[derive(GodotClass)]
#[class(base=Node3D)]
pub struct RapierCollider3D {
    #[var]
    pub id: Array<u32>, // ColliderHandle::into_raw_parts
    pub handle: ColliderHandle,
    pub collider: Collider,
    #[export]
    pub shape: ShapeType,
    pub parent: Option<RigidBodyHandle>,
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for RapierCollider3D {
    fn init(base: Base<Node3D>) -> Self {
        godot_print!("RapierCollider3D::init()");
        // ColliderBuilder::ball(0.5).restitution(0.7).build()
        Self {
            id: Array::new(),
            handle: ColliderHandle::invalid(),
            collider: ColliderBuilder::ball(0.5).restitution(0.7).build(),
            shape: ShapeType::Ball,
            parent: None,
            base,
        }
    }

    fn on_notification(&mut self, what: Node3DNotification) {
        match what {
            Node3DNotification::EnterTree => self.on_enter_tree(),
            Node3DNotification::ExitTree => self.on_exit_tree(),
            Node3DNotification::TransformChanged => self.on_transform_changed(),
            _ => {}
        };
    }
}

#[godot_api]
impl RapierCollider3D {
    fn on_enter_tree(&mut self) {
        self.base_mut().set_notify_transform(true);
    }

    fn on_exit_tree(&mut self) {
        godot_print!("RapierCollider3D::exit_tree()");
        // TODO remove self from physics pipeline collider_set
    }

    fn on_transform_changed(&mut self) {
        let transform = self.base().get_global_transform();
        let (pos, rot) = transform_to_posrot(transform);
        self.set_rapier_translation(pos);
        self.set_rapier_rotation(rot);
    }

    fn set_rapier_translation(&mut self, translation: NAVector3<Real>) {
        self.collider.set_translation(translation);
    }

    fn set_rapier_rotation(&mut self, rotation: Rotation<Real>) {
        self.collider.set_rotation(rotation);
    }
}

#[derive(GodotConvert, Var, Export)]
#[godot(via = GString)]
// https://docs.rs/rapier3d/latest/rapier3d/geometry/enum.ShapeType.html
pub enum ShapeType {
    Ball,
    Cuboid,
    Capsule,
}
