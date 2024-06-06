use crate::objects::RapierCollider3D;
use crate::queue::{Actionable, CanDispatchActions};
use crate::utils::{
    get_shallow_children_of_type, isometry_to_transform, HasCUID2Field, HasHandleField,
};
use crate::{ObjectKind, PhysicsObject};
use godot::engine::notify::Node3DNotification;
use godot::engine::{INode3D, Node3D};
use godot::obj::WithBaseField;
use godot::prelude::*;
use nalgebra::RealField;
use rapier3d::control::{CharacterAutostep, CharacterLength, KinematicCharacterController};
use rapier3d::prelude::*;

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
    // My comment
    #[export]
    #[var(usage_flags = [EDITOR, STORAGE, READ_ONLY])]
    pub id: GString, // https://crates.io/crates/cuid2
    character_controller: KinematicCharacterController,
    #[export]
    character_type: CharacterType,
    #[export]
    additional_mass: Real,

    #[export]
    offset: Real,
    #[export]
    offset_reference: FrameOfReference,

    #[export]
    climb_steps: bool,
    #[export]
    max_step_height: Real, // Relative to character length
    #[export]
    min_step_width: Real, // Relative to character length
    #[export]
    step_reference: FrameOfReference,
    #[export]
    include_dynamic_bodies: bool,

    #[export]
    slide: bool,
    #[export]
    max_slope_climb_angle: Real,
    #[export]
    min_slope_slide_angle: Real,

    #[export]
    snap_to_ground: bool,
    #[export]
    snap_reference: FrameOfReference,
    #[export]
    max_snap_distance: Real,

    #[export]
    normal_nudge_factor: Real,

    handle: Handle,
    hot_reload_cb: Callable,
    colliders: Vec<Gd<RapierCollider3D>>,
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for RapierCharacterBody3D {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            id: GString::from(crate::cuid2()),
            character_controller: KinematicCharacterController::default(),
            character_type: CharacterType::default(),
            additional_mass: 0.0,
            offset: 0.01,
            offset_reference: FrameOfReference::RelativeToCharacterLength,
            climb_steps: true,
            max_step_height: 0.25,
            min_step_width: 0.5,
            step_reference: FrameOfReference::RelativeToCharacterLength,
            include_dynamic_bodies: true,
            slide: true,
            max_slope_climb_angle: Real::frac_pi_4(),
            min_slope_slide_angle: Real::frac_pi_4(),
            snap_to_ground: true,
            max_snap_distance: 0.2,
            snap_reference: FrameOfReference::RelativeToCharacterLength,
            normal_nudge_factor: 1.0e-4,
            handle: Handle::invalid(),
            hot_reload_cb: Callable::invalid(),
            colliders: Vec::new(),
            base,
        }
    }

    fn on_notification(&mut self, what: Node3DNotification) {
        match what {
            Node3DNotification::ENTER_TREE => self.on_enter_tree(),
            Node3DNotification::EXIT_TREE => self.on_exit_tree(),
            Node3DNotification::TRANSFORM_CHANGED => self.on_transform_changed(),
            _ => {}
        }
    }
}

#[godot_api]
impl RapierCharacterBody3D {
    fn update_character_controller(&mut self) {
        let offset = get_character_length(self.offset, &self.offset_reference);

        let max_height = get_character_length(self.max_step_height, &self.step_reference);
        let min_width = get_character_length(self.min_step_width, &self.step_reference);
        let autostep = match self.climb_steps {
            true => Some(CharacterAutostep {
                max_height,
                min_width,
                include_dynamic_bodies: self.include_dynamic_bodies,
            }),
            false => None,
        };

        let snap_to_ground = match self.snap_to_ground {
            true => Some(get_character_length(
                self.max_snap_distance,
                &self.snap_reference,
            )),
            false => None,
        };

        self.character_controller = KinematicCharacterController {
            up: Vector::y_axis(),
            offset,
            slide: true,
            autostep,
            max_slope_climb_angle: self.max_slope_climb_angle,
            min_slope_slide_angle: self.min_slope_slide_angle,
            snap_to_ground,
            normal_nudge_factor: self.normal_nudge_factor,
        };
    }

    fn update_colliders(&mut self) {
        self.colliders = get_shallow_children_of_type::<RapierCollider3D>(&self.base());
        // TODO instead get registered collider children in rapier
    }

    fn on_enter_tree(&mut self) {
        // TODO register rigidbody
        self.update_character_controller();
        self.update_colliders();
    }

    fn on_exit_tree(&mut self) {
        // TODO unregister rigidbody
        self.colliders = Vec::new();
    }

    fn on_transform_changed(&mut self) {
        // TODO sync rigidbody in rapier
    }

    #[func]
    fn move_and_slide(&mut self, velocity: Vector3, delta_time: f32) {
        let handles = self
            .colliders
            .iter()
            .filter_map(|c| {
                let handle = c.bind().handle.clone();
                match handle.kind {
                    HandleKind::Invalid => None,
                    _ => Some(handle),
                }
            })
            .collect::<Vec<Handle>>();

        // handles come from state instead

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

            godot_print!("autostep: {:?}", self.character_controller.autostep);
            godot_print!("old_pos: {:?}", collider_pos.translation.vector);
            godot_print!("new_pos: {:?}", new_pos.translation.vector);

            // bind.pipeline
            //     .get_collider_mut(handle)
            //     .unwrap()
            //     .set_position(new_pos);

            // !@ set rigidbody instead

            self.base_mut()
                .set_global_transform(isometry_to_transform(new_pos));

            // TODO - use action queue instead
            // - use rigidbody (i think required for collision?)
            // - hopping why?

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

impl PhysicsObject for RapierCharacterBody3D {
    fn get_kind(&self) -> ObjectKind {
        ObjectKind::Character
    }

    fn get_hot_reload_cb(&self) -> Callable {
        self.hot_reload_cb.clone()
    }

    fn set_hot_reload_cb(&mut self, cb: Callable) {
        self.hot_reload_cb = cb;
    }

    fn build(&self) -> Actionable {
        let rb = RigidBodyBuilder::new(self.character_type.clone().into())
            .additional_mass(self.additional_mass)
            .build();
        Actionable::RigidBody(rb)
    }
}

#[derive(GodotConvert, Var, Export, Debug, Clone)]
#[godot(via = GString)]
pub enum CharacterType {
    KinematicVelocityBased,
    KinematicPositionBased,
}

impl Default for CharacterType {
    fn default() -> Self {
        CharacterType::KinematicVelocityBased
    }
}

impl Into<RigidBodyType> for CharacterType {
    fn into(self) -> RigidBodyType {
        match self {
            CharacterType::KinematicPositionBased => RigidBodyType::KinematicPositionBased,
            CharacterType::KinematicVelocityBased => RigidBodyType::KinematicVelocityBased,
        }
    }
}

#[derive(GodotConvert, Var, Export, Debug, Clone)]
#[godot(via = GString)]
enum FrameOfReference {
    RelativeToCharacterLength,
    Absolute,
}

// TODO can this be implemented using From somehow?
fn get_character_length(value: Real, reference: &FrameOfReference) -> CharacterLength {
    match reference {
        &FrameOfReference::RelativeToCharacterLength => CharacterLength::Relative(value),
        &FrameOfReference::Absolute => CharacterLength::Absolute(value),
    }
}

impl HasCUID2Field for RapierCharacterBody3D {
    fn get_cuid2(&self) -> String {
        self.id.to_string()
    }

    fn set_cuid2(&mut self, cuid2: String) {
        self.id = GString::from(cuid2);
    }
}

impl HasHandleField for RapierCharacterBody3D {
    fn get_handle(&self) -> Handle {
        self.handle.clone()
    }

    fn set_handle(&mut self, handle: Handle) {
        match handle.kind {
            HandleKind::RigidBodyHandle | HandleKind::Invalid => {
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

impl CanDispatchActions for RapierCharacterBody3D {}
