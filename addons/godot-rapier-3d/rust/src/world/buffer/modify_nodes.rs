use godot::prelude::*;
use rapier3d::{
    control::{CharacterLength, KinematicCharacterController, PdController, PidController},
    math::UnitVector,
    prelude::{AxesMask, QueryFilter, RigidBodyHandle, RigidBodyVelocity},
};

use crate::{
    nodes::{
        IRapierObject, Identifiable, RapierKinematicCharacter3D, RapierPIDCharacter3D,
        RapierRigidBody3D,
    },
    utils::{uniform_rapier_vector, vector_to_rapier},
    world::state::PhysicsState,
};

pub fn configure_node(node: Gd<Node3D>) {
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
        "RapierArea3D" | "RapierRigidBody3D" | "RapierCollisionShape3D" | "RapierStaticBody3D" => {
            // TODO
        }
        _ => log::error!(
            "Trying to configure a '{}' node which is not a configurable node type",
            class
        ),
    }
}

pub fn teleport_node(node: Gd<Node3D>, position: Vector3, physics: &mut PhysicsState) {
    let class = node.get_class().to_string();
    match class.as_str() {
        "RapierKinematicCharacter3D" => {
            teleport_rb(
                &node.cast::<RapierKinematicCharacter3D>(),
                physics,
                position,
            );
        }
        "RapierPIDCharacter3D" => {
            teleport_rb(&node.cast::<RapierPIDCharacter3D>(), physics, position);
        }
        "RapierRigidBody3D" => {
            teleport_rb(&node.cast::<RapierRigidBody3D>(), physics, position);
        }
        _ => log::error!("Cannot teleport node '{}'", class),
    }
}

fn teleport_rb(node: &Gd<impl IRapierObject>, physics: &mut PhysicsState, position: Vector3) {
    if let Some(raw_handle) = physics
        .lookup_table
        .get_rapier_handle(&node.bind().get_cuid())
    {
        let handle = RigidBodyHandle::from_raw_parts(raw_handle.0, raw_handle.1);
        let body = &mut physics.bodies[handle];
        godot_print!("Teleporting {} to {:?}", node, position);
        body.set_next_kinematic_translation(vector_to_rapier(position));
    }
}

pub fn move_node(node: Gd<Node3D>, desired_movement: Vector3, physics: &mut PhysicsState) {
    let class = node.get_class().to_string();

    match class.as_str() {
        "RapierKinematicCharacter3D" => {
            let mut casted = node.cast::<RapierKinematicCharacter3D>();
            let mut char = casted.bind_mut();
            let handle = physics.lookup_table.get_rapier_handle(&char.get_cuid());

            if let Some(raw) = handle {
                let controller = &char.controller;
                let handle = RigidBodyHandle::from_raw_parts(raw.0, raw.1);

                if !physics.bodies.contains(handle) {
                    return;
                }

                let body = &physics.bodies[handle];
                let collider = &physics.colliders[body.colliders()[0]];
                let mass = body.mass();

                let mut collisions = vec![];
                let movement = controller.move_shape(
                    physics.integration_parameters.dt,
                    &physics.bodies,
                    &physics.colliders,
                    &physics.query_pipeline,
                    collider.shape(),
                    collider.position(),
                    vector_to_rapier(desired_movement),
                    QueryFilter::new().exclude_rigid_body(handle),
                    |c| collisions.push(c),
                );

                // Apply impulses to other rigidbodies that were contacted
                controller.solve_character_collision_impulses(
                    physics.integration_parameters.dt,
                    &mut physics.bodies,
                    &physics.colliders,
                    &physics.query_pipeline,
                    collider.shape(),
                    mass,
                    &*collisions,
                    QueryFilter::new().exclude_rigid_body(handle),
                );

                let body = &mut physics.bodies[handle];

                let pose = body.position();
                body.set_next_kinematic_translation(pose.translation.vector + movement.translation);

                char.last_movement = Some(movement);
                char.last_collisions = collisions;
            }
        }
        "RapierPIDCharacter3D" => {
            let mut casted = node.cast::<RapierPIDCharacter3D>();
            let uid = casted.bind().get_cuid();
            let handle = physics.lookup_table.get_rapier_handle(&uid);

            if let Some(raw) = handle {
                let controller = &mut casted.bind_mut().controller;
                let handle = RigidBodyHandle::from_raw_parts(raw.0, raw.1);
                let body = &mut physics.bodies[handle];

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
                    physics.integration_parameters.dt,
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
            let handle = physics.lookup_table.get_rapier_handle(&uid);

            if let Some(raw) = handle {
                let controller = &mut casted.bind_mut().controller;
                let handle = RigidBodyHandle::from_raw_parts(raw.0, raw.1);

                if !physics.bodies.contains(handle) {
                    return;
                }

                let body = &mut physics.bodies[handle];

                let corrective_vel = controller.rigid_body_correction(
                    physics.integration_parameters.dt,
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
