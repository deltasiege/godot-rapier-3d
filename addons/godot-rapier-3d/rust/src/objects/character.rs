use crate::objects::RapierCollider3D;
use crate::queue::CanDispatchActions;
use crate::utils::{isometry_to_transform, HasCUID2Field};
use godot::engine::{INode3D, Node3D};
use godot::obj::WithBaseField;
use godot::prelude::*;
use rapier3d::control::KinematicCharacterController;
use rapier3d::math::Translation;
use rapier3d::pipeline::QueryFilter;

use super::{Handle, HandleKind};

// rigidbody bake into character (only allow one per character)
// colliders as children ? or maybe only shape needed?
// when told to move, get children via godot and cast them to RapierCollider3D
// get collider handle(s) from RapierCollider3D
// pass via action:
// collider handle(s)
// character mass (self)

// later in queue
// pull collider(s) out of state using collider handle in action
// get &dyn Shape from Collider.shape(&self)
// get current position from Collider.position(&self)
// get mass from Collider.mass_properties(&self).mass + character mass

#[derive(GodotClass)]
#[class(base = Node3D)]
pub struct RapierCharacterBody3D {
    #[export]
    #[var(usage_flags = [EDITOR, STORAGE, READ_ONLY])]
    pub id: GString, // https://crates.io/crates/cuid2
    #[export]
    mass: f32,
    character_controller: KinematicCharacterController,
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for RapierCharacterBody3D {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            id: GString::from(crate::cuid2()),
            mass: 1.0,
            character_controller: KinematicCharacterController::default(),
            base,
        }
    }
}

#[godot_api]
impl RapierCharacterBody3D {
    #[func]
    fn move_and_slide(&mut self, velocity: Vector3, delta_time: f32) {
        let colliders = get_shallow_children_of_type::<RapierCollider3D>(&self.base());
        let handles = colliders
            .iter()
            .filter_map(|c| {
                let handle = c.bind().handle.clone();
                match handle.kind {
                    HandleKind::Invalid => None,
                    _ => Some(handle),
                }
            })
            .collect::<Vec<Handle>>();

        for handle in handles {
            let mut engine = crate::get_engine().unwrap();
            let mut bind = engine.bind_mut();
            let collider = bind.pipeline.get_collider(handle.clone()).unwrap();
            let collider_shape = collider.shape();
            let collider_pos = collider.position();
            // let mass = collider.mass_properties().mass() + self.mass;

            let result = self.character_controller.move_shape(
                delta_time,
                &bind.pipeline.state.rigid_body_set,
                &bind.pipeline.state.collider_set,
                &bind.pipeline.state.query_pipeline,
                collider_shape,
                collider_pos,
                crate::utils::vec_g2r(velocity),
                QueryFilter::default(),
                // QueryFilter::default().exclude_rigid_body(),
                |_| {},
            );

            let mut new_pos = collider_pos.clone();
            new_pos.append_translation_mut(&Translation::from(result.translation));

            bind.pipeline
                .get_collider_mut(handle)
                .unwrap()
                .set_position(new_pos);

            self.base_mut()
                .set_global_transform(isometry_to_transform(new_pos));

            // engine.bind_mut().action_queue.add_action(
            //     self.get_action(Actionable::MoveCharacter {
            //         collider_handle,
            //         shape,
            //         position,
            //         mass,
            //         velocity,
            //         delta_time,
            //     }),
            //     &QueueName::Sync,
            // );
        }
    }
}

fn get_shallow_children_of_type<T: WithBaseField<Base = Node3D> + Inherits<Node>>(
    node: &Node3D,
) -> Vec<Gd<T>> {
    let mut children = Vec::new();
    for i in 0..node.get_child_count() {
        let child = match node.get_child(i) {
            Some(child) => child,
            None => continue,
        };
        match child.try_cast::<T>() {
            Ok(c) => children.push(c),
            Err(_) => continue,
        }
    }
    children
}

impl HasCUID2Field for RapierCharacterBody3D {
    fn get_cuid2(&self) -> String {
        self.id.to_string()
    }

    fn set_cuid2(&mut self, cuid2: String) {
        self.id = GString::from(cuid2);
    }
}

impl CanDispatchActions for RapierCharacterBody3D {}
