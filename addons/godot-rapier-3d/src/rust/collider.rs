use crate::physics_pipeline::RapierPhysicsPipeline;
use crate::rigid_body::RapierRigidBody3D;
use godot::engine::notify::Node3DNotification;
use godot::engine::INode3D;
use godot::engine::Node3D;
use godot::prelude::*;
use nalgebra::geometry::OPoint;
use nalgebra::Vector3 as NAVector3;
use rapier3d::prelude::*;

#[derive(GodotClass)]
#[class(base=Node3D)]
pub struct RapierCollider3D {
    #[var]
    pub id: Array<Variant>, // ColliderHandle::into_raw_parts
    pub handle: ColliderHandle,
    #[export]
    pub shape: ShapeType,
    pub parent: Option<RigidBodyHandle>,
    notify_parent: bool,
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for RapierCollider3D {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            id: Array::new(),
            handle: ColliderHandle::invalid(),
            shape: ShapeType::Ball,
            parent: None,
            notify_parent: true,
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
        };
    }
}

#[godot_api]
impl RapierCollider3D {
    fn on_enter_tree(&mut self) {
        self.base_mut().set_notify_transform(true);
        let ston = crate::utils::get_singleton();
        if ston.is_some() {
            let mut singleton = ston.unwrap();
            let pipeline = &mut singleton.bind_mut().pipeline;
            let handle = pipeline.register_collider(self);
            self.handle = handle;
            self.id = crate::utils::collider_handle_to_id(handle);

            let collider = pipeline.get_collider_mut(self.handle);

            match collider {
                Some(collider) => {
                    self.sync_transforms_to_godot(collider);
                }
                None => {
                    godot_error!("RapierCollider3D::on_enter_tree - Could not find collider");
                    return;
                }
            }
        }
    }

    fn on_exit_tree(&mut self) {
        let ston = crate::utils::get_singleton();
        if ston.is_some() {
            ston.unwrap().bind_mut().pipeline.unregister_collider(self);
        }
    }

    fn on_parented(&mut self) {
        if !self.notify_parent {
            return;
        }
        self.notify_parent = false;
        let _res = self
            .base_mut()
            .try_call_deferred(StringName::from("_on_parented"), &[]); // Collider registering needs to be defferred so that RigidBodies are already registered
    }

    #[func]
    fn _on_parented(&mut self) {
        let ston = crate::utils::get_singleton();
        if ston.is_some() {
            let mut singleton = ston.unwrap();
            let pipeline = &mut singleton.bind_mut().pipeline;

            let parent = self.base().get_parent_node_3d();

            match parent {
                Some(parent) => {
                    let is_rb = parent.is_class(GString::from("RapierRigidBody3D"));

                    match is_rb {
                        true => {
                            let casted = parent.cast::<RapierRigidBody3D>();
                            let class = casted.bind();

                            if self.parent == Some(class.handle) {
                                return;
                            }

                            let rb_exists = pipeline.rigid_body_set.contains(class.handle);
                            if !rb_exists {
                                godot_error!("RapierCollider3D::on_parented - trying to parent to invalid rigid body {:?}", class.handle);
                                return;
                            }
                            self.parent = Some(class.handle);
                            pipeline.unregister_collider(self);
                            let handle = pipeline.register_collider_with_parent(self, class.handle);
                            self.handle = handle;
                            self.id = crate::utils::collider_handle_to_id(handle);
                        }
                        false => self.clear_parent(pipeline),
                    }
                }
                None => self.clear_parent(pipeline),
            }
        }
        self.notify_parent = true;
    }

    fn on_unparented(&mut self) {
        if !self.notify_parent {
            return;
        }
        let ston = crate::utils::get_singleton();
        if ston.is_some() {
            let mut singleton = ston.unwrap();
            let pipeline = &mut singleton.bind_mut().pipeline;
            self.clear_parent(pipeline);
        }
    }

    fn clear_parent(&mut self, pipeline: &mut RapierPhysicsPipeline) {
        self.parent = None;
        pipeline.set_collider_parent(self.handle, None);
        // TODO maybe need to remove collider from RB?
    }

    fn on_transform_changed(&mut self) {
        let ston = crate::utils::get_singleton();
        if ston.is_some() {
            let mut singleton = ston.unwrap();
            let pipeline = &mut singleton.bind_mut().pipeline;
            let collider = pipeline.get_collider_mut(self.handle);

            match collider {
                Some(collider) => {
                    self.sync_transforms_to_godot(collider);
                }
                None => {
                    godot_error!("Could not find collider {:?}", self.handle);
                    return;
                }
            }
        }
    }

    fn sync_transforms_to_godot(&mut self, collider: &mut Collider) {
        let translation = self.base().get_global_position();
        let rotation = self.base().get_quaternion();
        let r_pos = crate::utils::pos_godot_to_rapier(translation);
        let r_rot = crate::utils::rot_godot_to_rapier(rotation);
        collider.set_translation(r_pos);
        collider.set_rotation(r_rot);
    }

    pub fn build(&self) -> Collider {
        let shape = match self.shape {
            ShapeType::Ball => SharedShape::ball(0.5),
            ShapeType::Cuboid => SharedShape::cuboid(10.0, 1.0, 10.0),
            ShapeType::Capsule => {
                SharedShape::capsule(OPoint::origin(), OPoint::from(NAVector3::y()), 0.5)
            }
        };
        let collider = ColliderBuilder::new(shape).restitution(0.7).build();
        collider
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
