use godot::classes::notify::Node3DNotification;
use godot::classes::{INode3D, Node3D};
use godot::prelude::*;
use rapier3d::control::{
    CharacterCollision, EffectiveCharacterMovement, KinematicCharacterController,
};

use super::common::{Controllable, Forceable};
use super::Identifiable;
use crate::nodes::IRapierObject;
use crate::utils::vector_to_godot;

/*
    I had some issues with jittering when desired_movement is pushing into the floor.
    Maybe handle disabling of Y component of desired_movement when grounded?
    https://github.com/dimforge/rapier/issues/809
*/

#[derive(GodotClass)]
#[class(tool, base=Node3D)]
pub struct RapierKinematicCharacter3D {
    #[var(usage_flags = [EDITOR, STORAGE, READ_ONLY])]
    pub handle: Array<u32>,

    // bool floor_block_on_wall = true
    // bool floor_constant_speed = false // IMPORTANT
    #[export]
    floor_max_angle: f32,
    #[export]
    floor_min_slide_angle: f32,
    #[export]
    floor_snap_length: f32,
    #[export]
    normal_nudge_factor: f32,
    // bool floor_stop_on_slope = true // unsure how to do
    // max_slides: i32, // can't do?
    // MotionMode motion_mode = 0 (grounded vs flying characters) // nice to have but tricky
    #[export]
    safe_margin: f32,
    #[export]
    up_direction: Vector3,

    #[export]
    slide: bool,

    pub last_movement: Option<EffectiveCharacterMovement>,
    pub last_collisions: Vec<CharacterCollision>,
    pub controller: KinematicCharacterController,

    // rapier specific settings
    // autostep: Option<CharacterAutostep> nice to have but expensive performance apparently
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for RapierKinematicCharacter3D {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            handle: Array::new(),
            floor_max_angle: 0.7853982,       // (45 degrees in radians)
            floor_min_slide_angle: 0.7853982, // (45 degrees in radians)
            floor_snap_length: 0.2,
            normal_nudge_factor: 0.0001,
            safe_margin: 0.01,
            up_direction: Vector3::UP,
            slide: true,
            last_movement: None,
            last_collisions: Vec::new(),
            controller: KinematicCharacterController::default(),
            base,
        }
    }

    fn enter_tree(&mut self) {
        self.on_enter_tree();
    }

    fn exit_tree(&mut self) {
        self.on_exit_tree();
    }

    fn physics_process(&mut self, _delta: f64) {
        self.sync();
    }

    fn on_notification(&mut self, what: Node3DNotification) {
        match what {
            Node3DNotification::PHYSICS_PROCESS => self.sync(),
            _ => {}
        }
    }
}

#[godot_api]
impl RapierKinematicCharacter3D {
    #[func]
    fn set_uid(&mut self, cuid: GString) {
        self.set_cuid(cuid);
    }

    #[func]
    fn match_rapier(&mut self) {
        self.sync()
    }

    #[func]
    fn move_by_amount(&self, amount: Vector3) {
        self.on_move_by_amount(amount);
    }

    #[func]
    fn teleport_to_position(&self, position: Vector3) {
        self.on_teleport_to_position(position);
    }

    #[func]
    fn get_real_velocity(&self) -> Vector3 {
        self.get_body_state().linvel
    }

    #[func]
    fn get_real_angular_velocity(&self) -> Vector3 {
        self.get_body_state().angvel
    }

    #[func]
    fn is_sleeping(&self) -> bool {
        self.get_body_state().sleeping
    }

    #[func]
    fn is_moving(&self) -> bool {
        self.get_body_state().moving
    }

    #[func]
    fn is_on_floor(&self) -> bool {
        match &self.last_movement {
            Some(movement) => movement.grounded,
            None => false,
        }
    }

    #[func]
    fn is_sliding_down_slope(&self) -> bool {
        match &self.last_movement {
            Some(movement) => movement.is_sliding_down_slope,
            None => false,
        }
    }

    #[func]
    fn get_slide_collision_count(&self) -> i32 {
        self.last_collisions.len() as i32
    }

    #[func]
    fn get_last_motion(&self) -> Vector3 {
        match &self.last_movement {
            Some(movement) => vector_to_godot(movement.translation),
            None => Vector3::ZERO,
        }
    }

    // void apply_floor_snap() // won't do - unimportant?
    // float get_floor_angle(up_direction: Vector3 = Vector3(0, 1, 0)) // IMPORTANT
    // Vector3 get_floor_normal() // IMPORTANT
    // KinematicCollision3D get_last_slide_collision() // useful but tricky
}
