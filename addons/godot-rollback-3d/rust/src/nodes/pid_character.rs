use godot::classes::notify::Node3DNotification;
use godot::classes::{INode3D, Node3D};
use godot::prelude::*;
use rapier3d::prelude::*;
use serde::{Deserialize, Serialize};

use crate::interface::*;
use crate::nodes::common::*;
use crate::utils::{vector_to_godot, vector_to_point};

#[derive(GodotClass)]
#[class(tool, base=Node3D)]
/// Description of the RollbackPIDCharacter3D class.
pub struct RollbackPIDCharacter3D {
    #[export]
    /// The Proportional gain applied to the instantaneous linear position errors.
    /// This is usually set to a multiple of the inverse of simulation step time
    /// (e.g. `60` if the delta-time is `1.0 / 60.0`).
    pub kp: f32,
    #[export]
    /// The linear gain applied to the Integral part of the PID controller.
    pub ki: f32,
    #[export]
    /// The Derivative gain applied to the instantaneous linear velocity errors.
    /// This is usually set to a value in `[0.0, 1.0]` where `0.0` implies no damping
    /// (no correction of velocity errors) and `1.0` implies complete damping (velocity errors
    /// are corrected in a single simulation step).
    pub kd: f32,

    #[export]
    pub floor_check_ray_length: f32,

    base: Base<Node3D>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PDControllerSettings {
    pub kp: f32,
    pub ki: f32,
    pub kd: f32,
}

#[godot_api]
impl INode3D for RollbackPIDCharacter3D {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            kp: 60.0,
            ki: 1.0,
            kd: 0.8,
            floor_check_ray_length: 0.1,
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
impl RollbackPIDCharacter3D {
    pub fn get_controller_settings(&self) -> PDControllerSettings {
        PDControllerSettings {
            kp: self.kp,
            ki: self.ki,
            kd: self.kd,
        }
    }

    #[func]
    fn move_by_amount(&self, amount: Vector3) {
        self.on_move_by_amount(amount);
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
        self.try_is_on_floor().unwrap_or(false)
    }

    fn try_is_on_floor(&self) -> Option<bool> {
        if let Some(gr3d) = get_gr3d() {
            let handle = self.get_node_data()?.get_rapier_handle().left()?;

            let bind = gr3d.bind();
            let bodies = &bind.world.physics.bodies;
            let colliders = &bind.world.physics.colliders;
            let query_pipeline = &bind.world.physics.query_pipeline;

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
                if let Some(tree) = self.base().get_tree() {
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
                return Some(true);
            } else {
                return Some(false);
            }
        } else {
            return Some(false);
        }
    }
}
