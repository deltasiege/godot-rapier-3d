use godot::classes::notify::Node3DNotification;
use godot::classes::{INode3D, Node3D};
use godot::prelude::*;
use rapier3d::control::{
    CharacterCollision, CharacterLength, EffectiveCharacterMovement, KinematicCharacterController,
};
use rapier3d::math::UnitVector;

use crate::nodes::common::*;
use crate::utils::{vector_to_godot, vector_to_rapier};

/*
    I had some issues with jittering when desired_movement is pushing into the floor.
    Maybe handle disabling of Y component of desired_movement when grounded?
    https://github.com/dimforge/rapier/issues/809
*/

#[derive(GodotClass)]
#[class(tool, base=Node3D)]
pub struct RollbackKinematicCharacter3D {
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

    // rapier specific settings
    // autostep: Option<CharacterAutostep> nice to have but expensive performance apparently
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for RollbackKinematicCharacter3D {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            floor_max_angle: 0.7853982,       // (45 degrees in radians)
            floor_min_slide_angle: 0.7853982, // (45 degrees in radians)
            floor_snap_length: 0.2,
            normal_nudge_factor: 0.0001,
            safe_margin: 0.01,
            up_direction: Vector3::UP,
            slide: true,
            last_movement: None,
            last_collisions: Vec::new(),
            base,
        }
    }

    fn enter_tree(&mut self) {
        self.on_enter_tree();
    }

    fn exit_tree(&mut self) {
        self.on_exit_tree();
    }

    fn on_notification(&mut self, what: Node3DNotification) {
        match what {
            Node3DNotification::PHYSICS_PROCESS => self.sync(),
            _ => {}
        }
    }
}

#[godot_api]
impl RollbackKinematicCharacter3D {
    pub fn get_controller(&self) -> KinematicCharacterController {
        KinematicCharacterController {
            up: UnitVector::new_normalize(vector_to_rapier(self.get_up_direction())),
            offset: CharacterLength::Relative(self.get_safe_margin()),
            slide: self.get_slide(),
            autostep: None, // TODO
            max_slope_climb_angle: self.get_floor_max_angle(),
            min_slope_slide_angle: self.get_floor_min_slide_angle(),
            snap_to_ground: Some(CharacterLength::Relative(self.get_floor_snap_length())),
            normal_nudge_factor: self.get_normal_nudge_factor(),
        }
    }

    #[func]
    fn get_real_velocity(&self) -> Vector3 {
        vector_to_godot(self.get_body_state().linvel)
    }

    #[func]
    fn get_real_angular_velocity(&self) -> Vector3 {
        vector_to_godot(self.get_body_state().angvel)
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
