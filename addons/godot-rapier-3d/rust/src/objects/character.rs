use crate::queue::{Actionable, CanDispatchActions, QueueName};
use crate::utils::{vec_g2r, HasCUID2Field, HasHandleField};
use crate::{ObjectKind, PhysicsObject};
use godot::classes::notify::Node3DNotification;
use godot::classes::{INode3D, Node3D};
use godot::prelude::*;
use nalgebra::RealField;
use rapier3d::control::{CharacterAutostep, CharacterLength, KinematicCharacterController};
use rapier3d::prelude::*;

use super::{Handle, HandleKind};

#[derive(GodotClass)]
#[class(base = Node3D)]
pub struct RapierCharacterBody3D {
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
    max_step_height: Real,
    #[export]
    min_step_width: Real,
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
            base,
        }
    }

    fn on_notification(&mut self, what: Node3DNotification) {
        match what {
            Node3DNotification::ENTER_TREE => self._on_enter_tree(),
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

    fn _on_enter_tree(&mut self) {
        self.update_character_controller();
        self.on_enter_tree();
    }

    #[func]
    fn move_character(&mut self, amount: Vector3, delta_time: f32) {
        if amount == Vector3::ZERO {
            return;
        }

        self.dispatch_action(
            Actionable::MoveCharacter {
                cuid2: self.get_cuid2(),
                controller: self.character_controller,
                amount: vec_g2r(amount),
                delta_time,
            },
            &QueueName::Sim,
        )
        .map_err(crate::handle_error)
        .ok();

        self.sync_g2r().map_err(crate::handle_error).ok();
    }
}

impl PhysicsObject for RapierCharacterBody3D {
    fn get_kind(&self) -> ObjectKind {
        ObjectKind::Character
    }

    fn get_hot_reload_cb(&self) -> &Callable {
        &self.hot_reload_cb
    }

    fn set_hot_reload_cb(&mut self, cb: Callable) {
        self.hot_reload_cb = cb;
    }

    fn build(&self) -> Actionable {
        let rb = RigidBodyBuilder::new(self.character_type.clone().into())
            .additional_mass(self.additional_mass)
            .build();
        Actionable::Character(rb)
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
