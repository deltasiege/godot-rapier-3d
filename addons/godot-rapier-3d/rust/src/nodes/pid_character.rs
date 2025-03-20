use godot::classes::notify::Node3DNotification;
use godot::classes::{INode3D, Node3D};
use godot::prelude::*;
use rapier3d::control::PidController;
use rapier3d::prelude::*;

use super::common::{Controllable, Forceable};
use super::Identifiable;
use crate::interface::{get_runtime, get_singleton, get_tree};
use crate::nodes::IRapierObject;
use crate::utils::vector_to_point;

#[derive(GodotClass)]
#[class(tool, base=Node3D)]
pub struct RapierPIDCharacter3D {
    #[var(usage_flags = [EDITOR, STORAGE, READ_ONLY])]
    pub handle: Array<u32>,

    // TODO why aren't descriptions working? https://github.com/godot-rust/gdext/issues/1059
    #[export]
    /// The Proportional gain applied to the instantaneous linear position errors.
    /// This is usually set to a multiple of the inverse of simulation step time
    /// (e.g. `60` if the delta-time is `1.0 / 60.0`).
    pub lin_kp: f32,
    #[export]
    /// The linear gain applied to the Integral part of the PID controller.
    pub lin_ki: f32,
    #[export]
    /// The Derivative gain applied to the instantaneous linear velocity errors.
    /// This is usually set to a value in `[0.0, 1.0]` where `0.0` implies no damping
    /// (no correction of velocity errors) and `1.0` implies complete damping (velocity errors
    /// are corrected in a single simulation step).
    pub lin_kd: f32,
    #[export]
    /// The Proportional gain applied to the instantaneous angular position errors.
    /// This is usually set to a multiple of the inverse of simulation step time
    /// (e.g. `60` if the delta-time is `1.0 / 60.0`).
    pub ang_kp: f32,
    #[export]
    /// The angular gain applied to the Integral part of the PID controller.
    pub ang_ki: f32,
    #[export]
    /// The Derivative gain applied to the instantaneous angular velocity errors.
    /// This is usually set to a value in `[0.0, 1.0]` where `0.0` implies no damping
    /// (no correction of velocity errors) and `1.0` implies complete damping (velocity errors
    /// are corrected in a single simulation step).
    pub ang_kd: f32,

    #[export]
    pub floor_check_ray_length: f32,

    pub controller: PidController,
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for RapierPIDCharacter3D {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            handle: Array::new(),
            lin_kp: 60.0,
            lin_ki: 1.0,
            lin_kd: 0.8,
            ang_kp: 60.0,
            ang_ki: 1.0,
            ang_kd: 0.8,
            floor_check_ray_length: 0.1,
            controller: PidController::default(),
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
impl RapierPIDCharacter3D {
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
        if let Some(singleton) = get_singleton() {
            if self.handle.len() < 2 {
                return false;
            }
            let bind = singleton.bind();
            let bodies = &bind.world.physics.bodies;
            let colliders = &bind.world.physics.colliders;
            let query_pipeline = &bind.world.physics.query_pipeline;

            let handle = RigidBodyHandle::from_raw_parts(self.handle.at(0), self.handle.at(1));
            let body = &bodies[handle];
            let collider = &colliders[body.colliders()[0]];
            let collider_height = collider.shape().compute_local_aabb().half_extents().y;
            let origin = body.translation() + vector![0.0, collider_height, 0.0];

            let ray = Ray::new(
                vector_to_point(&origin),
                vector![0.0, -1.0 * self.floor_check_ray_length, 0.0],
            );

            // Debug ray
            if let Some(mut runtime) = get_runtime(self) {
                if let Some(tree) = get_tree(self) {
                    if tree.is_debugging_collisions_hint() {
                        let origin_vec = Vector3 {
                            x: origin.x,
                            y: origin.y,
                            z: origin.z,
                        };
                        runtime.call(
                            "_draw_line",
                            &[
                                Vector3 {
                                    x: origin.x,
                                    y: origin.y,
                                    z: origin.z,
                                }
                                .to_variant(),
                                (origin_vec
                                    + Vector3 {
                                        x: 0.0,
                                        y: -self.floor_check_ray_length,
                                        z: 0.0,
                                    })
                                .to_variant(),
                            ],
                        );
                    }
                }
            }

            if let Some((_handle, _toi)) = query_pipeline.cast_ray(
                bodies,
                colliders,
                &ray,
                self.floor_check_ray_length,
                false,
                QueryFilter::new()
                    .exclude_rigid_body(handle)
                    .exclude_collider(body.colliders()[0]),
            ) {
                // let hit_point = ray.point_at(toi);
                // println!("Collider {:?} hit at point {}", handle, hit_point);
                return true;
            } else {
                return false;
            }
        } else {
            return false;
        }
    }
}
