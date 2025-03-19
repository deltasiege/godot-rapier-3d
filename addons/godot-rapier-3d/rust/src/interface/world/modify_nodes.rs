use godot::prelude::*;
use rapier3d::{
    control::{CharacterLength, KinematicCharacterController, PdController, PidController},
    math::UnitVector,
    prelude::{AxesMask, QueryFilter, RigidBodyHandle, RigidBodyVelocity},
};

use crate::{
    nodes::{Identifiable, RapierKinematicCharacter3D, RapierPIDCharacter3D, RapierRigidBody3D},
    utils::{uniform_rapier_vector, vector_to_rapier},
    World,
};

pub fn configure_nodes(nodes: Array<Gd<Node3D>>, _world: &mut World) {
    for node in nodes.iter_shared() {
        let class = node.get_class().to_string();

        match class.as_str() {
            "RapierKinematicCharacter3D" => {
                let mut casted = node.cast::<RapierKinematicCharacter3D>();
                let mut char = casted.bind_mut();

                char.controller = KinematicCharacterController {
                    up: UnitVector::new_normalize(vector_to_rapier(char.get_up_direction())),
                    offset: CharacterLength::Relative(char.get_safe_margin()),
                    slide: char.get_slide(),
                    autostep: None, // TODO
                    max_slope_climb_angle: char.get_floor_max_angle(),
                    min_slope_slide_angle: char.get_floor_min_slide_angle(),
                    snap_to_ground: Some(CharacterLength::Relative(char.get_floor_snap_length())),
                    normal_nudge_factor: char.get_normal_nudge_factor(),
                }
            }
            "RapierPIDCharacter3D" => {
                let mut casted = node.cast::<RapierPIDCharacter3D>();
                let mut char = casted.bind_mut();

                char.controller = PidController {
                    pd: PdController {
                        lin_kp: uniform_rapier_vector(char.lin_kp),
                        lin_kd: uniform_rapier_vector(char.lin_kd),
                        ang_kp: uniform_rapier_vector(char.ang_kp),
                        ang_kd: uniform_rapier_vector(char.ang_kd),
                        axes: AxesMask::all(),
                    },
                    lin_ki: uniform_rapier_vector(char.lin_ki),
                    ang_ki: uniform_rapier_vector(char.ang_ki),
                    ..PidController::default()
                };
            }
            "RapierArea3D"
            | "RapierRigidBody3D"
            | "RapierCollisionShape3D"
            | "RapierStaticBody3D" => {
                // TODO
            }
            _ => log::error!(
                "Trying to configure a '{}' node which is not a configurable node type",
                class
            ),
        }
    }
}

pub fn move_nodes(nodes: Array<Gd<Node3D>>, desired_movement: Vector3, world: &mut World) {
    for node in nodes.iter_shared() {
        let class = node.get_class().to_string();

        match class.as_str() {
            "RapierKinematicCharacter3D" => {
                let mut casted = node.cast::<RapierKinematicCharacter3D>();
                let mut char = casted.bind_mut();
                let handle = world
                    .physics
                    .lookup_table
                    .get_rapier_handle(&char.get_cuid());

                if let Some(raw) = handle {
                    let controller = &char.controller;
                    let handle = RigidBodyHandle::from_raw_parts(raw.0, raw.1);

                    if !world.physics.bodies.contains(handle) {
                        return;
                    }

                    let body = &world.physics.bodies[handle];
                    let collider = &world.physics.colliders[body.colliders()[0]];
                    let mass = body.mass();

                    let mut collisions = vec![];
                    let movement = controller.move_shape(
                        world.physics.integration_parameters.dt,
                        &world.physics.bodies,
                        &world.physics.colliders,
                        &world.physics.query_pipeline,
                        collider.shape(),
                        collider.position(),
                        vector_to_rapier(desired_movement),
                        QueryFilter::new().exclude_rigid_body(handle),
                        |c| collisions.push(c),
                    );

                    // Apply impulses to other rigidbodies that were contacted
                    controller.solve_character_collision_impulses(
                        world.physics.integration_parameters.dt,
                        &mut world.physics.bodies,
                        &world.physics.colliders,
                        &world.physics.query_pipeline,
                        collider.shape(),
                        mass,
                        &*collisions,
                        QueryFilter::new().exclude_rigid_body(handle),
                    );

                    let body = &mut world.physics.bodies[handle];

                    let pose = body.position();
                    body.set_next_kinematic_translation(
                        pose.translation.vector + movement.translation,
                    );

                    char.last_movement = Some(movement);
                    char.last_collisions = collisions;
                }
            }
            "RapierPIDCharacter3D" => {
                let mut casted = node.cast::<RapierPIDCharacter3D>();
                let uid = casted.bind().get_cuid();
                let handle = world.physics.lookup_table.get_rapier_handle(&uid);

                if let Some(raw) = handle {
                    let controller = &mut casted.bind_mut().controller;
                    let handle = RigidBodyHandle::from_raw_parts(raw.0, raw.1);
                    let body = &mut world.physics.bodies[handle];

                    let mut axes = AxesMask::ANG_X | AxesMask::ANG_Y | AxesMask::ANG_Z;
                    let mvmt = vector_to_rapier(desired_movement);

                    if mvmt.norm() != 0.0 {
                        axes |= if desired_movement.y == 0.0 {
                            AxesMask::LIN_X | AxesMask::LIN_Z
                        } else {
                            AxesMask::LIN_X | AxesMask::LIN_Z | AxesMask::LIN_Y
                        }
                    };

                    controller.set_axes(axes);

                    let corrective_vel = controller.rigid_body_correction(
                        world.physics.integration_parameters.dt,
                        body,
                        (body.translation() + mvmt).into(),
                        RigidBodyVelocity::zero(),
                    );

                    body.set_vels(*body.vels() + corrective_vel, true);
                }
            }
            "RapierRigidBody3D" => {
                let mut casted = node.cast::<RapierRigidBody3D>();
                let uid = casted.bind().get_cuid();
                let handle = world.physics.lookup_table.get_rapier_handle(&uid);

                if let Some(raw) = handle {
                    let controller = &mut casted.bind_mut().controller;
                    let handle = RigidBodyHandle::from_raw_parts(raw.0, raw.1);

                    if !world.physics.bodies.contains(handle) {
                        return;
                    }

                    let body = &mut world.physics.bodies[handle];

                    let corrective_vel = controller.rigid_body_correction(
                        world.physics.integration_parameters.dt,
                        body,
                        (body.translation() + vector_to_rapier(desired_movement)).into(),
                        RigidBodyVelocity::zero(),
                    );

                    body.set_vels(*body.vels() + corrective_vel, true);
                }
            }
            _ => log::error!("Cannot move node '{}'", class),
        }
    }
}
